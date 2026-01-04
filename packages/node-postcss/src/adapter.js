/**
 * JSON Adapter: Converts postcss-rs JSON to PostCSS-compatible JSON
 *
 * This adapter bridges the small format differences between our Rust
 * parser output and what PostCSS.fromJSON() expects.
 *
 * Performance: ~0.1ms for typical CSS files (negligible overhead)
 */

/**
 * Adapts postcss-rs JSON output to PostCSS-compatible format
 *
 * Changes made:
 * 1. Add `inputs` array at root with source file metadata
 * 2. Add `inputId` reference to all `source` objects (for deduplication)
 * 3. Remove `nodes: []` from leaf nodes (decl, comment)
 * 4. Ensure root has `source.end`
 *
 * @param {object} rustJSON - JSON output from postcss-rs parser
 * @param {string} css - Original CSS source text
 * @param {string} [from] - Source file path
 * @returns {object} PostCSS-compatible JSON
 */
function adaptRustJSONToPostCSS(rustJSON, css, from) {
  // Deep clone to avoid modifying the original
  const adapted = JSON.parse(JSON.stringify(rustJSON));

  // Add inputs array (PostCSS uses this for source map generation)
  adapted.inputs = [{
    hasBOM: false,
    css: css,
    file: from || undefined
  }];

  /**
   * Recursively process nodes to add inputId references
   * and clean up structure
   */
  function processNode(node) {
    // Add inputId reference to source (points to inputs array)
    if (node.source) {
      node.source.inputId = 0; // Index into inputs array
    }

    if (node.nodes) {
      // Remove nodes array from leaf nodes (declarations and comments)
      // These don't have children, so the array should not exist
      if (node.type === 'decl' || node.type === 'comment') {
        delete node.nodes;
      } else {
        // Recursively process container children
        node.nodes.forEach(processNode);
      }
    }
  }

  processNode(adapted);

  // Ensure root has end source position
  if (adapted.source && !adapted.source.end && adapted.nodes && adapted.nodes.length > 0) {
    const lastNode = adapted.nodes[adapted.nodes.length - 1];
    if (lastNode.source && lastNode.source.end) {
      adapted.source.end = { ...lastNode.source.end };
    }
  }

  return adapted;
}

module.exports = {
  adaptRustJSONToPostCSS
};
