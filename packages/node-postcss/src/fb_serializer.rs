/// FlatBuffers serialiser for the PostCSS AST.
///
/// Traverses the tree bottom-up (FlatBuffers requires all child offsets to
/// be created before the parent table is started) and returns the finished
/// byte buffer.
#[allow(dead_code, unused_imports)]
use flatbuffers::FlatBufferBuilder;
use postcss_rs_parser::{AtRule, Data, Node, Raws};

#[path = "css_ast_generated.rs"]
mod gen;
pub use gen::css_ast as fb;

/// Serialise a `Node` tree into a FlatBuffer.
///
/// Returns the completed buffer as a `Vec<u8>`.  The buffer is
/// self-contained: it starts at byte 0 and the root object is at the
/// offset stored at the last 4 bytes (standard FlatBuffers framing).
pub fn to_flatbuffer(root: &Node) -> Vec<u8> {
    // Pre-allocate ~5× the typical JSON size.  FlatBuffers buffers are
    // usually smaller than JSON but we reuse the same heuristic.
    let mut fbb = FlatBufferBuilder::with_capacity(512);
    let root_offset = build_node(&mut fbb, root);
    fbb.finish(root_offset, None);
    fbb.finished_data().to_vec()
}

/// Recursively build a `Node` table, bottom-up.
fn build_node<'bldr>(
    fbb: &mut FlatBufferBuilder<'bldr>,
    node: &Node,
) -> flatbuffers::WIPOffset<fb::Node<'bldr>> {
    // ── 1. Build children first (must be created before the parent table) ──
    let nodes_vec = node.nodes.as_ref().map(|children| {
        let offsets: Vec<_> = children.iter().map(|c| build_node(fbb, c)).collect();
        fbb.create_vector(&offsets)
    });

    // ── 2. Create all string offsets (must precede start_table) ──────────
    let raws = &node.raws;

    let r_before        = raw_str(fbb, &raws.before);
    let r_between       = raw_str(fbb, &raws.between);
    let r_after         = raw_str(fbb, &raws.after);
    let r_after_name    = raw_str(fbb, &raws.after_name);
    let r_left          = raw_str(fbb, &raws.left);
    let r_right         = raw_str(fbb, &raws.right);
    let r_important     = raw_str(fbb, &raws.important);
    let r_own_semicolon = raw_str(fbb, &raws.own_semicolon);
    let r_value_value   = raws.value.value.as_deref().map(|s| fbb.create_string(s));
    let r_value_raw     = raws.value.raw.as_deref().map(|s| fbb.create_string(s));
    let r_selector_value = raws.selector.value.as_deref().map(|s| fbb.create_string(s));
    let r_selector_raw   = raws.selector.raw.as_deref().map(|s| fbb.create_string(s));

    // Type-specific strings
    let (selector_off, prop_off, value_off, name_off, params_off, text_off) =
        match &node.data {
            Data::Rule(r) => (
                Some(fbb.create_string(&r.selector)),
                None, None, None, None, None,
            ),
            Data::Declaration(d) => (
                None,
                Some(fbb.create_string(&d.prop)),
                Some(fbb.create_string(&d.value)),
                None, None, None,
            ),
            Data::AtRule(a) => (
                None, None, None,
                Some(fbb.create_string(&a.name)),
                a.params.as_deref().map(|s| fbb.create_string(s)),
                None,
            ),
            Data::Comment(c) => (
                None, None, None, None, None,
                Some(fbb.create_string(&c.text)),
            ),
            Data::Root => (None, None, None, None, None, None),
        };

    // ── 3. Build the table ────────────────────────────────────────────────
    let mut b = fb::NodeBuilder::new(fbb);

    // Node type
    let t: i8 = match &node.data {
        Data::Root => 0,
        Data::Rule(_) => 1,
        Data::Declaration(_) => 2,
        Data::AtRule(_) => 3,
        Data::Comment(_) => 4,
    };
    b.add_t(t);

    // Source start (always present)
    let src = &node.source;
    b.add_ss_off(src.start.offset as u32);
    b.add_ss_line(src.start.line as u32);
    b.add_ss_col(src.start.column as u32);

    // Source end (optional)
    if let Some(end) = &src.end {
        b.add_has_end(true);
        b.add_se_off(end.offset as u32);
        b.add_se_line(end.line as u32);
        b.add_se_col(end.column as u32);
    }

    // Raws
    if let Some(v) = r_before        { b.add_r_before(v); }
    if let Some(v) = r_between       { b.add_r_between(v); }
    if let Some(v) = r_after         { b.add_r_after(v); }
    if let Some(v) = r_after_name    { b.add_r_after_name(v); }
    if let Some(semi) = *raws.semicolon {
        b.add_r_semicolon(if semi { 1 } else { 0 });
    }
    if let Some(v) = r_left          { b.add_r_left(v); }
    if let Some(v) = r_right         { b.add_r_right(v); }
    if let Some(v) = r_important     { b.add_r_important(v); }
    if let Some(v) = r_own_semicolon { b.add_r_own_semicolon(v); }
    if let Some(v) = r_value_value   { b.add_r_value_value(v); }
    if let Some(v) = r_value_raw     { b.add_r_value_raw(v); }
    if let Some(v) = r_selector_value { b.add_r_selector_value(v); }
    if let Some(v) = r_selector_raw  { b.add_r_selector_raw(v); }

    // Type-specific
    if let Some(v) = selector_off    { b.add_selector(v); }
    if let Some(v) = prop_off        { b.add_prop(v); }
    if let Some(v) = value_off       { b.add_value(v); }
    if let Data::Declaration(d) = &node.data {
        if d.important == Some(true) { b.add_important(true); }
    }
    if let Some(v) = name_off        { b.add_name(v); }
    if let Some(v) = params_off      { b.add_params(v); }
    if let Some(v) = text_off        { b.add_text(v); }

    // Children
    if let Some(v) = nodes_vec       { b.add_nodes(v); }

    b.finish()
}

/// Helper: create a string offset from an optional `Raw` wrapper,
/// returning `None` when the raw is absent (skipped).
#[inline]
fn raw_str<'b>(
    fbb: &mut FlatBufferBuilder<'b>,
    raw: &postcss_rs_parser::Raw,
) -> Option<flatbuffers::WIPOffset<&'b str>> {
    raw.as_deref().map(|s| fbb.create_string(s))
}
