/// Direct NAPI V8 object construction — no serialization format.
///
/// Uses raw napi_sys calls to avoid lifetime/type complications with the
/// higher-level napi-rs wrappers, while still giving us full NAPI access.
use napi::{sys, Env, JsObject, NapiRaw, NapiValue, Result};
use napi::check_status;
use postcss_rs_parser::{Data, Node, Raws};
use std::ptr;

/// Raw NAPI handles to the five JS factory functions.
/// `napi_value` is just `*mut c_void` — Copy and lifetime-free.
#[derive(Clone, Copy)]
struct Factories {
    make_root:    sys::napi_value,
    make_rule:    sys::napi_value,
    make_decl:    sys::napi_value,
    make_atrule:  sys::napi_value,
    make_comment: sys::napi_value,
}

// ── low-level helpers ──────────────────────────────────────────────────────────

pub struct Builder {
    env: sys::napi_env,
    global: sys::napi_value,
}

impl Builder {
    pub fn new(env: &Env) -> Result<Self> {
        let raw_env = unsafe { env.raw() };
        let mut global = ptr::null_mut();
        check_status!(
            unsafe { sys::napi_get_global(raw_env, &mut global) },
            "napi_get_global failed"
        )?;
        Ok(Self { env: raw_env, global })
    }

    #[inline]
    unsafe fn get_named(&self, obj: sys::napi_value, name: &[u8]) -> Result<sys::napi_value> {
        let mut key = ptr::null_mut();
        check_status!(
            sys::napi_create_string_utf8(
                self.env,
                name.as_ptr() as *const _,
                name.len() as isize,
                &mut key,
            ),
            "create key failed"
        )?;
        let mut out = ptr::null_mut();
        check_status!(
            sys::napi_get_property(self.env, obj, key, &mut out),
            "get_property failed"
        )?;
        Ok(out)
    }

    #[inline]
    unsafe fn set_named(&self, obj: sys::napi_value, name: &[u8], val: sys::napi_value) -> Result<()> {
        let mut key = ptr::null_mut();
        check_status!(
            sys::napi_create_string_utf8(
                self.env,
                name.as_ptr() as *const _,
                name.len() as isize,
                &mut key,
            ),
            "create key failed"
        )?;
        check_status!(
            sys::napi_set_property(self.env, obj, key, val),
            "set_property failed"
        )?;
        Ok(())
    }

    #[inline]
    unsafe fn call0(&self, func: sys::napi_value) -> Result<sys::napi_value> {
        let mut ret = ptr::null_mut();
        check_status!(
            sys::napi_call_function(self.env, self.global, func, 0, ptr::null(), &mut ret),
            "napi_call_function failed"
        )?;
        Ok(ret)
    }

    #[inline]
    unsafe fn create_obj(&self) -> Result<sys::napi_value> {
        let mut obj = ptr::null_mut();
        check_status!(sys::napi_create_object(self.env, &mut obj), "napi_create_object failed")?;
        Ok(obj)
    }

    #[inline]
    unsafe fn create_str(&self, s: &str) -> Result<sys::napi_value> {
        let mut v = ptr::null_mut();
        check_status!(
            sys::napi_create_string_utf8(self.env, s.as_ptr() as *const _, s.len() as isize, &mut v),
            "napi_create_string_utf8 failed"
        )?;
        Ok(v)
    }

    #[inline]
    unsafe fn create_u32(&self, n: u32) -> Result<sys::napi_value> {
        let mut v = ptr::null_mut();
        check_status!(sys::napi_create_uint32(self.env, n, &mut v), "napi_create_uint32 failed")?;
        Ok(v)
    }

    #[inline]
    unsafe fn create_bool(&self, b: bool) -> Result<sys::napi_value> {
        let mut v = ptr::null_mut();
        check_status!(
            sys::napi_get_boolean(self.env, b, &mut v),
            "napi_get_boolean failed"
        )?;
        Ok(v)
    }

    #[inline]
    unsafe fn set_element(&self, arr: sys::napi_value, i: u32, val: sys::napi_value) -> Result<()> {
        check_status!(
            sys::napi_set_element(self.env, arr, i, val),
            "napi_set_element failed"
        )?;
        Ok(())
    }

    unsafe fn build_raws(&self, raws: &Raws) -> Result<sys::napi_value> {
        let obj = self.create_obj()?;

        macro_rules! set_str {
            ($raw:expr, $name:literal) => {
                if let Some(v) = $raw.as_deref() {
                    let s = self.create_str(v)?;
                    self.set_named(obj, $name, s)?;
                }
            };
        }
        set_str!(raws.before,       b"before");
        set_str!(raws.between,      b"between");
        set_str!(raws.after,        b"after");
        set_str!(raws.after_name,   b"afterName");
        if let Some(v) = *raws.semicolon {
            let b = self.create_bool(v)?;
            self.set_named(obj, b"semicolon", b)?;
        }
        set_str!(raws.left,         b"left");
        set_str!(raws.right,        b"right");
        set_str!(raws.important,    b"important");
        set_str!(raws.own_semicolon, b"ownSemicolon");

        if raws.value.value.is_some() || raws.value.raw.is_some() {
            let rv = self.create_obj()?;
            if let Some(v) = raws.value.value.as_deref() { let s = self.create_str(v)?; self.set_named(rv, b"value", s)?; }
            if let Some(v) = raws.value.raw.as_deref()   { let s = self.create_str(v)?; self.set_named(rv, b"raw",   s)?; }
            self.set_named(obj, b"value", rv)?;
        }
        if raws.selector.value.is_some() || raws.selector.raw.is_some() {
            let rs = self.create_obj()?;
            if let Some(v) = raws.selector.value.as_deref() { let s = self.create_str(v)?; self.set_named(rs, b"value", s)?; }
            if let Some(v) = raws.selector.raw.as_deref()   { let s = self.create_str(v)?; self.set_named(rs, b"raw",   s)?; }
            self.set_named(obj, b"selector", rs)?;
        }

        Ok(obj)
    }

    pub unsafe fn build_node(
        &self,
        node:      &Node,
        factories: &Factories,
        input:     sys::napi_value,
        parent:    Option<sys::napi_value>,
    ) -> Result<sys::napi_value> {
        // ── 1. Factory call → proto-correct object ──────────────────────────
        let factory = match &node.data {
            Data::Root           => factories.make_root,
            Data::Rule(_)        => factories.make_rule,
            Data::Declaration(_) => factories.make_decl,
            Data::AtRule(_)      => factories.make_atrule,
            Data::Comment(_)     => factories.make_comment,
        };
        let js_node = self.call0(factory)?;

        // ── 2. Type-specific properties ─────────────────────────────────────
        match &node.data {
            Data::Root => {}
            Data::Rule(r) => {
                let s = self.create_str(&r.selector)?;
                self.set_named(js_node, b"selector", s)?;
            }
            Data::Declaration(d) => {
                let p = self.create_str(&d.prop)?;
                self.set_named(js_node, b"prop", p)?;
                let v = self.create_str(&d.value)?;
                self.set_named(js_node, b"value", v)?;
                if d.important == Some(true) {
                    let t = self.create_bool(true)?;
                    self.set_named(js_node, b"important", t)?;
                }
            }
            Data::AtRule(a) => {
                let n = self.create_str(&a.name)?;
                self.set_named(js_node, b"name", n)?;
                let p = self.create_str(a.params.as_deref().unwrap_or(""))?;
                self.set_named(js_node, b"params", p)?;
            }
            Data::Comment(c) => {
                let t = self.create_str(&c.text)?;
                self.set_named(js_node, b"text", t)?;
            }
        }

        // ── 3. Raws ─────────────────────────────────────────────────────────
        let raws = self.build_raws(&node.raws)?;
        self.set_named(js_node, b"raws", raws)?;

        // ── 4. Source ────────────────────────────────────────────────────────
        let source = self.create_obj()?;
        let start  = self.create_obj()?;
        self.set_named(start, b"offset", self.create_u32(node.source.start.offset as u32)?)?;
        self.set_named(start, b"line",   self.create_u32(node.source.start.line   as u32)?)?;
        self.set_named(start, b"column", self.create_u32(node.source.start.column as u32)?)?;
        self.set_named(source, b"start", start)?;
        if let Some(end) = &node.source.end {
            let end_obj = self.create_obj()?;
            self.set_named(end_obj, b"offset", self.create_u32(end.offset as u32)?)?;
            self.set_named(end_obj, b"line",   self.create_u32(end.line   as u32)?)?;
            self.set_named(end_obj, b"column", self.create_u32(end.column as u32)?)?;
            self.set_named(source, b"end", end_obj)?;
        }
        self.set_named(source, b"input", input)?;
        self.set_named(js_node, b"source", source)?;

        // ── 5. Parent ────────────────────────────────────────────────────────
        if let Some(p) = parent {
            self.set_named(js_node, b"parent", p)?;
        }

        // ── 6. Children ──────────────────────────────────────────────────────
        if let Some(children) = &node.nodes {
            let n = children.len() as u32;
            let mut arr = ptr::null_mut();
            check_status!(
                sys::napi_create_array_with_length(self.env, n as usize, &mut arr),
                "napi_create_array_with_length failed"
            )?;
            for (i, child) in children.iter().enumerate() {
                let child_js = self.build_node(child, factories, input, Some(js_node))?;
                self.set_element(arr, i as u32, child_js)?;
            }
            self.set_named(js_node, b"nodes", arr)?;
        }

        Ok(js_node)
    }
}

/// Entry point called from the #[napi] function.
pub fn build(env: &Env, css: &str, ctx: &JsObject) -> Result<JsObject> {
    let builder = Builder::new(env)?;
    let ctx_raw = unsafe { ctx.raw() };

    let factories = unsafe {
        Factories {
            make_root:    builder.get_named(ctx_raw, b"makeRoot")?,
            make_rule:    builder.get_named(ctx_raw, b"makeRule")?,
            make_decl:    builder.get_named(ctx_raw, b"makeDecl")?,
            make_atrule:  builder.get_named(ctx_raw, b"makeAtRule")?,
            make_comment: builder.get_named(ctx_raw, b"makeComment")?,
        }
    };
    let input = unsafe { builder.get_named(ctx_raw, b"input")? };

    let mut parser = postcss_rs_parser::Parser::new(css).unwrap();
    let node = parser.parse();

    let raw_result = unsafe { builder.build_node(&node, &factories, input, None)? };

    // Wrap the raw napi_value back into a JsObject for the #[napi] return type.
    Ok(unsafe { JsObject::from_raw_unchecked(builder.env, raw_result) })
}
