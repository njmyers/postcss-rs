/**
 * JSON Adapter: Converts postcss-rs JSON to PostCSS-compatible JSON
 *
 * This adapter bridges the small format differences between our Rust
 * parser output and what PostCSS.fromJSON() expects.
 *
 * Performance: ~0.1ms for typical CSS files (negligible overhead)
 */

interface SourcePosition {
  line: number;
  column: number;
  offset?: number;
}

interface Source {
  start?: SourcePosition;
  end?: SourcePosition;
  inputId?: number;
}

interface Input {
  hasBOM: boolean;
  css: string;
  file?: string;
}

interface ASTNode {
  type: string;
  source?: Source;
  nodes?: ASTNode[];
}

interface RootNode extends ASTNode {
  type: 'root';
  inputs?: Input[];
}

/**
 * Adapts postcss-rs JSON output to PostCSS-compatible format
 *
 * Note: The Rust parser now automatically:
 * - Skips the `nodes` field for leaf nodes (declarations, comments)
 * - Includes `inputId: 0` in all `source` objects
 * - Includes `inputs` array at root with source file metadata
 *
 * This function now only ensures root has `source.end` if missing.
 *
 * @param rustJSON - JSON output from postcss-rs parser
 * @param _css - Original CSS source text (no longer needed, kept for compatibility)
 * @param _from - Source file path (no longer needed, kept for compatibility)
 * @returns PostCSS-compatible JSON
 */
export function adaptRustJSONToPostCSS(
  rustJSON: RootNode,
  _css: string,
  _from?: string
): RootNode {
  // Ensure root has end source position
  if (rustJSON.source && !rustJSON.source.end && rustJSON.nodes && rustJSON.nodes.length > 0) {
    const lastNode = rustJSON.nodes[rustJSON.nodes.length - 1];
    if (lastNode.source && lastNode.source.end) {
      rustJSON.source.end = { ...lastNode.source.end };
    }
  }

  return rustJSON;
}
