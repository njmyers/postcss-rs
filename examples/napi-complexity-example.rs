/**
 * NAPI Rust Implementation Example
 *
 * This demonstrates the complexity required to implement PostCSS
 * node classes using NAPI Rust bindings.
 */

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::{Arc, Mutex, Weak};
use serde::{Serialize, Deserialize};

// Core Rust node structure (already exists)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    #[serde(flatten)]
    pub data: Data,
    pub nodes: Vec<Node>,
    pub source: Option<Source>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Data {
    #[serde(rename = "root")]
    Root,
    #[serde(rename = "rule")]
    Rule { selector: String },
    #[serde(rename = "decl")]
    Declaration { prop: String, value: String, important: bool },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub start: Position,
    pub end: Position,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

// NAPI wrapper with shared ownership - COMPLEX!
#[napi]
pub struct Declaration {
    // Need Arc for multiple JS references to same Rust data
    // Need Mutex for interior mutability from JS
    inner: Arc<Mutex<Node>>,

    // Need Weak to avoid circular references with parent
    parent: Weak<Mutex<Node>>,
}

#[napi]
impl Declaration {
    /// Get the property name
    ///
    /// COMPLEXITY ISSUES:
    /// 1. Must acquire lock (can fail)
    /// 2. Must check type (runtime type safety)
    /// 3. Must clone string (can't return reference across FFI)
    /// 4. FFI overhead (~200ns per call)
    #[napi(getter)]
    pub fn prop(&self) -> Result<String> {
        // Acquire lock - can fail if poisoned
        let node = self.inner.lock()
            .map_err(|_| Error::from_reason("Failed to acquire lock"))?;

        // Check type at runtime
        if let Data::Declaration { ref prop, .. } = node.data {
            // Must clone because we can't return a reference
            Ok(prop.clone())
        } else {
            Err(Error::from_reason("Not a declaration node"))
        }
    }

    /// Get the property value
    #[napi(getter)]
    pub fn value(&self) -> Result<String> {
        let node = self.inner.lock()
            .map_err(|_| Error::from_reason("Failed to acquire lock"))?;

        if let Data::Declaration { ref value, .. } = node.data {
            Ok(value.clone())
        } else {
            Err(Error::from_reason("Not a declaration node"))
        }
    }

    /// Set the property value
    ///
    /// MORE COMPLEXITY:
    /// 1. Mutable lock acquisition
    /// 2. Type checking
    /// 3. No easy way to notify JS that value changed
    #[napi(setter)]
    pub fn set_value(&mut self, new_value: String) -> Result<()> {
        let mut node = self.inner.lock()
            .map_err(|_| Error::from_reason("Failed to acquire lock"))?;

        if let Data::Declaration { ref mut value, .. } = node.data {
            *value = new_value;
            Ok(())
        } else {
            Err(Error::from_reason("Not a declaration node"))
        }
    }

    /// Get parent node
    ///
    /// MAJOR COMPLEXITY:
    /// 1. Upgrade Weak reference (can fail if parent dropped)
    /// 2. Must create NEW NAPI wrapper each time
    /// 3. Different JS object each call: node.parent !== node.parent
    /// 4. Breaks identity checks and Map/Set usage
    #[napi(getter)]
    pub fn parent(&self, env: Env) -> Result<Option<Container>> {
        // Try to upgrade weak reference
        let parent_arc = match self.parent.upgrade() {
            Some(arc) => arc,
            None => return Ok(None), // Parent was dropped
        };

        // Create a new NAPI wrapper - different object each time!
        let parent_wrapper = Container {
            inner: parent_arc.clone(),
            parent: Weak::new(), // Don't know grandparent
        };

        Ok(Some(parent_wrapper))
    }

    /// Remove this node from its parent
    ///
    /// SYNCHRONIZATION NIGHTMARE:
    /// 1. Must find self in parent's children
    /// 2. Must update parent's Vec
    /// 3. What if JS still holds references?
    /// 4. Iterator invalidation issues
    #[napi]
    pub fn remove(&mut self) -> Result<Self> {
        // Get parent
        let parent_arc = self.parent.upgrade()
            .ok_or_else(|| Error::from_reason("No parent"))?;

        // Lock both parent and self
        let mut parent_node = parent_arc.lock()
            .map_err(|_| Error::from_reason("Failed to lock parent"))?;

        let self_node = self.inner.lock()
            .map_err(|_| Error::from_reason("Failed to lock self"))?;

        // Find self in parent's children by pointer equality
        let self_idx = parent_node.nodes.iter().position(|n| {
            // Can't use Arc::ptr_eq because nodes are not Arc in parent
            // Have to compare by value... expensive!
            std::ptr::eq(n as *const Node, &*self_node as *const Node)
        }).ok_or_else(|| Error::from_reason("Not found in parent"))?;

        // Remove from parent
        parent_node.nodes.remove(self_idx);

        // Clear parent reference
        self.parent = Weak::new();

        Ok(self.clone()) // Must clone because self is &mut
    }

    /// Clone this node
    ///
    /// COMPLEXITY:
    /// 1. Deep clone all Rust data
    /// 2. Create new Arc<Mutex>
    /// 3. Recursively clone children
    #[napi]
    pub fn clone_node(&self) -> Result<Declaration> {
        let node = self.inner.lock()
            .map_err(|_| Error::from_reason("Failed to acquire lock"))?;

        // Deep clone the node
        let cloned = node.clone();

        // Wrap in new Arc<Mutex>
        Ok(Declaration {
            inner: Arc::new(Mutex::new(cloned)),
            parent: Weak::new(),
        })
    }
}

/// Container type (Rule, AtRule, Root)
#[napi]
pub struct Container {
    inner: Arc<Mutex<Node>>,
    parent: Weak<Mutex<Node>>,
}

#[napi]
impl Container {
    /// Walk all declarations
    ///
    /// CALLBACK HELL:
    /// 1. Must convert every child to NAPI wrapper
    /// 2. Must call JS function from Rust
    /// 3. Changes in callback don't sync back
    /// 4. Must clone nodes to pass to callback
    #[napi]
    pub fn walk_decls(&self, env: Env, callback: JsFunction) -> Result<()> {
        let node = self.inner.lock()
            .map_err(|_| Error::from_reason("Failed to acquire lock"))?;

        // Recursively walk tree
        self.walk_decls_internal(&node, &env, &callback)?;

        Ok(())
    }

    fn walk_decls_internal(&self, node: &Node, env: &Env, callback: &JsFunction) -> Result<()> {
        for (i, child) in node.nodes.iter().enumerate() {
            match child.data {
                Data::Declaration { .. } => {
                    // Create NAPI wrapper for this child
                    // Problem: We don't have an Arc for the child!
                    // It's stored by value in parent's Vec<Node>
                    // Must clone to create Arc!
                    let child_arc = Arc::new(Mutex::new(child.clone()));

                    let decl_wrapper = Declaration {
                        inner: child_arc,
                        parent: Arc::downgrade(&self.inner),
                    };

                    // Convert to JS object
                    let js_decl = decl_wrapper.into_instance(*env)?;

                    // Call callback
                    let result = callback.call(None, &[js_decl])?;

                    // Problem: If callback modified decl_wrapper,
                    // changes are in the cloned Arc, not in parent's Vec!
                    // Need complex synchronization...

                    // Check if callback returned false
                    if let Some(b) = result.coerce_to_bool() {
                        if !b.get_value()? {
                            return Ok(()); // Stop iteration
                        }
                    }
                },
                _ => {
                    // Recurse into containers
                    self.walk_decls_internal(child, env, callback)?;
                }
            }
        }

        Ok(())
    }

    /// Insert before a child node
    ///
    /// EXTREME COMPLEXITY:
    /// 1. Find old node in children
    /// 2. Convert JS object to Rust node
    /// 3. Insert into Vec
    /// 4. Update parent references
    /// 5. Invalidates any existing NAPI wrappers for children
    #[napi]
    pub fn insert_before(&mut self, env: Env, old_node: JsObject, new_node: JsObject) -> Result<()> {
        // This requires:
        // 1. Deserialize new_node from JS object
        // 2. Find old_node in children (how? compare by what?)
        // 3. Insert into Vec
        // 4. Update all NAPI wrapper references

        // This is extremely complex and error-prone
        unimplemented!("Too complex for example")
    }
}

// Compare this to JavaScript implementation:
//
// class Declaration extends Node {
//     type = 'decl';
//     prop: string;
//     value: string;
//     parent?: Container;
//
//     remove(): this {
//         if (this.parent) {
//             const idx = this.parent.nodes.indexOf(this);
//             this.parent.nodes.splice(idx, 1);
//             this.parent = undefined;
//         }
//         return this;
//     }
// }
//
// class Container extends Node {
//     nodes: ChildNode[] = [];
//
//     walkDecls(callback: (decl: Declaration) => void | false): void | false {
//         for (const node of this.nodes) {
//             if (node.type === 'decl') {
//                 if (callback(node) === false) return false;
//             }
//             if (node.nodes) {
//                 if ((node as Container).walkDecls(callback) === false) {
//                     return false;
//                 }
//             }
//         }
//     }
//
//     insertBefore(oldNode: ChildNode, newNode: ChildNode): this {
//         const idx = this.nodes.indexOf(oldNode);
//         newNode.parent = this;
//         this.nodes.splice(idx, 0, newNode);
//         return this;
//     }
// }
//
// The JS version is:
// - 10x less code
// - No locks
// - No Arc/Weak complexity
// - No cloning
// - Object identity works correctly
// - 30x faster property access
// - Natural JavaScript semantics
