//! Binary AST transfer format used by the JavaScript FFI.
//!
//! Internal AST references are tagged store-local IDs. The public JavaScript
//! decoder still consumes one dense node array, so serialization remaps IDs and
//! adds one virtual ClassBody record per class. No internal owner or ClassBody
//! node is allocated.

use std::{
    mem::{offset_of, size_of},
    slice,
};

use crate::{
    Ast, AstId, Comment, CommentKind, CommentPosition, CommonAstNode, ExtraData, NodeKind,
    NodePayload, Span, TypedNodeRange,
};

const HEADER_SIZE: usize = 36;
const SPAN_SIZE: usize = 8;
const NODE_KIND_SIZE: usize = 1;
const NODE_PAYLOAD_SIZE: usize = 16;
const EXTRA_DATA_SIZE: usize = 8;
const COMMENT_SIZE: usize = 16;
const FLAG_TS: u32 = 1 << 0;
const FLAG_SOURCE_TYPE_SHIFT: u32 = 1;
const CLASS_COMPAT_EXTRA_COUNT: usize = 5;
const CLASS_COMPAT_NODE_COUNT: usize = 1;

const _: () = {
    assert!(cfg!(target_endian = "little"));
    assert!(size_of::<Span>() == SPAN_SIZE);
    assert!(size_of::<NodeKind>() == NODE_KIND_SIZE);
    assert!(size_of::<NodePayload>() == NODE_PAYLOAD_SIZE);
    assert!(size_of::<ExtraData>() == EXTRA_DATA_SIZE);
    assert!(size_of::<AstId>() == 4);
    assert!(size_of::<CommentKind>() == 2);
    assert!(size_of::<CommentPosition>() == 2);
    assert!(size_of::<Comment>() == COMMENT_SIZE);
    assert!(offset_of!(Comment, span) == 0);
    assert!(offset_of!(Comment, attached_to) == 8);
    assert!(offset_of!(Comment, kind) == 12);
    assert!(offset_of!(Comment, position) == 14);
};

#[inline]
fn wire_node_count(tree: &Ast<'_>) -> usize {
    tree.node_count() + tree.class_count() * CLASS_COMPAT_NODE_COUNT
}

#[inline]
fn wire_extra_count(tree: &Ast<'_>) -> usize {
    tree.node_edge_count() + tree.extra_data_count() + tree.class_count() * CLASS_COMPAT_EXTRA_COUNT
}

/// Return the exact number of bytes required to serialize tree.
pub fn buffer_size(tree: &Ast<'_>) -> usize {
    let nodes = wire_node_count(tree);
    HEADER_SIZE
        + nodes * SPAN_SIZE
        + align_to_four(nodes * NODE_KIND_SIZE)
        + nodes * NODE_PAYLOAD_SIZE
        + wire_extra_count(tree) * EXTRA_DATA_SIZE
        + align_to_four(tree.strings.extra.len())
        + tree.comments().len() * COMMENT_SIZE
}

/// Serialize an AST into buf and return the required number of bytes.
///
/// When buf is too small, including when it is empty, this returns the
/// required size without writing a partial payload.
pub fn serialize_into(tree: &Ast<'_>, buf: &mut [u8]) -> usize {
    let required = buffer_size(tree);
    if buf.len() < required {
        return required;
    }

    let wire_map = WireMap::new(tree);
    let node_count = u32::try_from(wire_node_count(tree)).expect("AST wire node count exceeds u32");
    let node_edge_count =
        u32::try_from(tree.node_edge_count()).expect("AST node-edge count exceeds u32");
    let extra_count =
        u32::try_from(wire_extra_count(tree)).expect("AST wire extra-data count exceeds u32");
    let string_pool_len =
        u32::try_from(tree.strings.extra.len()).expect("AST string pool exceeds u32");
    let source_len = u32::try_from(tree.source().len()).expect("AST source length exceeds u32");
    let comment_count = u32::try_from(tree.comments().len()).expect("comment count exceeds u32");

    let mut pos = 0;
    for value in [
        node_count,
        extra_count,
        string_pool_len,
        source_len,
        comment_count,
        0, // attached_comment_count
        0, // diag_count
        wire_map.node_id(tree.root_program().ast_id()),
        (if tree.is_ts() { FLAG_TS } else { 0 })
            | ((tree.source_type() as u32) << FLAG_SOURCE_TYPE_SHIFT),
    ] {
        write_u32(buf, &mut pos, value);
    }

    write_node_spans(buf, &mut pos, &tree.nodes.data);
    write_node_spans(buf, &mut pos, &tree.expressions.data);
    write_node_spans(buf, &mut pos, &tree.statements.data);
    write_node_spans(buf, &mut pos, &tree.name_patterns.data);
    write_node_spans(buf, &mut pos, &tree.class_elements.data);
    write_node_spans(buf, &mut pos, &tree.type_jsx.data);
    write_node_spans(buf, &mut pos, &tree.program_aux.data);
    let function_spans = unsafe {
        slice::from_raw_parts(
            tree.functions.span.as_ptr().cast::<u8>(),
            tree.function_count() * SPAN_SIZE,
        )
    };
    buf[pos..pos + function_spans.len()].copy_from_slice(function_spans);
    pos += function_spans.len();
    let class_spans = unsafe {
        slice::from_raw_parts(
            tree.classes.span.as_ptr().cast::<u8>(),
            tree.class_count() * SPAN_SIZE,
        )
    };
    buf[pos..pos + class_spans.len()].copy_from_slice(class_spans);
    pos += class_spans.len();
    let classes = unsafe { slice::from_raw_parts(tree.classes.data.as_ptr(), tree.class_count()) };
    for class in classes {
        write_span(buf, &mut pos, class.body_span);
    }

    write_node_kinds(buf, &mut pos, &tree.nodes.data);
    write_node_kinds(buf, &mut pos, &tree.expressions.data);
    write_node_kinds(buf, &mut pos, &tree.statements.data);
    write_node_kinds(buf, &mut pos, &tree.name_patterns.data);
    write_node_kinds(buf, &mut pos, &tree.class_elements.data);
    write_node_kinds(buf, &mut pos, &tree.type_jsx.data);
    write_node_kinds(buf, &mut pos, &tree.program_aux.data);
    for _ in 0..tree.function_count() {
        buf[pos] = wire_kind(NodeKind::Function);
        pos += 1;
    }
    for _ in 0..tree.class_count() {
        buf[pos] = wire_kind(NodeKind::Class);
        pos += 1;
    }
    for _ in 0..tree.class_count() {
        buf[pos] = NodeKind::Class as u8 + 1;
        pos += CLASS_COMPAT_NODE_COUNT;
    }
    pad_to_four(buf, &mut pos);

    let mut compatibility_extra = tree.extra_data.as_slice().to_vec();
    let mut mapped_extra = vec![false; compatibility_extra.len()];
    let class_extra_base = tree.node_edge_count() + tree.extra_data_count();
    write_node_payloads(
        &wire_map,
        buf,
        &mut pos,
        &tree.nodes.data,
        &mut compatibility_extra,
        &mut mapped_extra,
        node_edge_count,
    );
    write_node_payloads(
        &wire_map,
        buf,
        &mut pos,
        &tree.expressions.data,
        &mut compatibility_extra,
        &mut mapped_extra,
        node_edge_count,
    );
    write_node_payloads(
        &wire_map,
        buf,
        &mut pos,
        &tree.statements.data,
        &mut compatibility_extra,
        &mut mapped_extra,
        node_edge_count,
    );
    write_node_payloads(
        &wire_map,
        buf,
        &mut pos,
        &tree.name_patterns.data,
        &mut compatibility_extra,
        &mut mapped_extra,
        node_edge_count,
    );
    write_node_payloads(
        &wire_map,
        buf,
        &mut pos,
        &tree.class_elements.data,
        &mut compatibility_extra,
        &mut mapped_extra,
        node_edge_count,
    );
    write_node_payloads(
        &wire_map,
        buf,
        &mut pos,
        &tree.type_jsx.data,
        &mut compatibility_extra,
        &mut mapped_extra,
        node_edge_count,
    );
    write_node_payloads(
        &wire_map,
        buf,
        &mut pos,
        &tree.program_aux.data,
        &mut compatibility_extra,
        &mut mapped_extra,
        node_edge_count,
    );
    let function_payloads =
        unsafe { slice::from_raw_parts(tree.functions.data.as_ptr(), tree.function_count()) };
    for &payload in function_payloads {
        let mut payload = payload;
        NodeKind::Function.map_ast_ids(
            &mut payload,
            &mut compatibility_extra,
            &mut mapped_extra,
            |id| AstId::from_raw(wire_map.node_id(id)),
        );
        let extra_id = (payload.0 >> 96) as u32;
        let relocated = extra_id
            .checked_add(node_edge_count)
            .expect("AST compatibility extra-data index exceeds u32");
        payload.0 = (payload.0 & ((1u128 << 96) - 1)) | ((relocated as u128) << 96);
        write_payload(buf, &mut pos, payload);
    }
    for (storage_index, class) in classes.iter().enumerate() {
        let extra_id = u32::try_from(class_extra_base + storage_index * CLASS_COMPAT_EXTRA_COUNT)
            .expect("class compatibility extra-data index exceeds u32");
        let id = class
            .id
            .map_or(u32::MAX, |node| wire_map.node_id(node.ast_id()));
        let type_parameters = class
            .type_parameters
            .map_or(u32::MAX, |node| wire_map.node_id(node.ast_id()));
        write_payload(
            buf,
            &mut pos,
            NodePayload(
                (class.class_type as u128)
                    | ((class.r#abstract as u128) << 8)
                    | ((class.declare as u128) << 16)
                    | ((id as u128) << 24)
                    | ((type_parameters as u128) << 56)
                    | ((extra_id as u128) << 96),
            ),
        );
    }
    for class in classes {
        write_payload(buf, &mut pos, NodePayload(range_word(class.body) as u128));
    }

    for edge in tree.node_edges.as_slice() {
        write_u32(buf, &mut pos, wire_map.node_id(*edge));
        write_u32(buf, &mut pos, 0);
    }
    for extra in &compatibility_extra {
        write_u64(buf, &mut pos, unsafe { extra.other });
    }
    for (index, class) in classes.iter().enumerate() {
        for word in [
            range_word(class.decorators),
            optional_wire_node(&wire_map, class.super_class),
            optional_wire_node(&wire_map, class.super_type_arguments),
            range_word(class.implements),
            wire_map.class_body_id(index) as u64,
        ] {
            write_u64(buf, &mut pos, word);
        }
    }

    let string_pool = &tree.strings.extra;
    buf[pos..pos + string_pool.len()].copy_from_slice(string_pool);
    pos += string_pool.len();
    pad_to_four(buf, &mut pos);

    let comments = unsafe {
        slice::from_raw_parts(
            tree.comments().as_ptr().cast::<u8>(),
            tree.comments().len() * COMMENT_SIZE,
        )
    };
    buf[pos..pos + comments.len()].copy_from_slice(comments);
    pos += comments.len();

    debug_assert_eq!(pos, required);
    required
}

struct WireMap {
    common: Vec<u32>,
    expressions: Vec<u32>,
    statements: Vec<u32>,
    name_patterns: Vec<u32>,
    class_elements: Vec<u32>,
    type_jsx: Vec<u32>,
    program_aux: Vec<u32>,
    function_base: u32,
    class_base: u32,
    class_body_base: u32,
}

impl WireMap {
    fn new(tree: &Ast<'_>) -> Self {
        fn map_store(nodes: &crate::NodeData<'_>, next: &mut u32) -> Vec<u32> {
            let kinds = unsafe { slice::from_raw_parts(nodes.kind.as_ptr(), nodes.len()) };
            kinds
                .iter()
                .map(|&kind| {
                    if kind == NodeKind::Hole {
                        u32::MAX
                    } else {
                        let id = *next;
                        *next = next
                            .checked_add(1)
                            .expect("AST wire node count exceeds u32");
                        id
                    }
                })
                .collect()
        }

        let mut next = 0;
        let common = map_store(&tree.nodes.data, &mut next);
        let expressions = map_store(&tree.expressions.data, &mut next);
        let statements = map_store(&tree.statements.data, &mut next);
        let name_patterns = map_store(&tree.name_patterns.data, &mut next);
        let class_elements = map_store(&tree.class_elements.data, &mut next);
        let type_jsx = map_store(&tree.type_jsx.data, &mut next);
        let program_aux = map_store(&tree.program_aux.data, &mut next);
        let function_base = next;
        next = next
            .checked_add(tree.function_count() as u32)
            .expect("AST wire node count exceeds u32");
        let class_base = next;
        next = next
            .checked_add(tree.class_count() as u32)
            .expect("AST wire node count exceeds u32");
        Self {
            common,
            expressions,
            statements,
            name_patterns,
            class_elements,
            type_jsx,
            program_aux,
            function_base,
            class_base,
            class_body_base: next,
        }
    }

    #[inline]
    fn node_id(&self, id: AstId) -> u32 {
        if id.is_none() {
            return u32::MAX;
        }
        let wire = match id.store_tag() {
            crate::StoreTag::Common => self.common[id.index()],
            crate::StoreTag::Expression => self.expressions[id.index()],
            crate::StoreTag::Statement => self.statements[id.index()],
            crate::StoreTag::NamePattern => self.name_patterns[id.index()],
            crate::StoreTag::ClassElement => self.class_elements[id.index()],
            crate::StoreTag::TypeJsx => self.type_jsx[id.index()],
            crate::StoreTag::ProgramAux => self.program_aux[id.index()],
            crate::StoreTag::Function => self.function_base + id.index() as u32,
            crate::StoreTag::Class => self.class_base + id.index() as u32,
        };
        assert_ne!(
            wire,
            u32::MAX,
            "AST edge references a hole: store={:?}, index={}",
            id.store_tag(),
            id.index()
        );
        wire
    }

    #[inline]
    fn class_body_id(&self, index: usize) -> u32 {
        self.class_body_base + index as u32
    }
}

#[inline]
fn write_node_spans(buf: &mut [u8], pos: &mut usize, nodes: &crate::NodeData<'_>) {
    let spans = unsafe { slice::from_raw_parts(nodes.span.as_ptr(), nodes.len()) };
    let kinds = unsafe { slice::from_raw_parts(nodes.kind.as_ptr(), nodes.len()) };
    for (&span, &kind) in spans.iter().zip(kinds) {
        if kind != NodeKind::Hole {
            write_span(buf, pos, span);
        }
    }
}

#[inline]
fn write_node_kinds(buf: &mut [u8], pos: &mut usize, nodes: &crate::NodeData<'_>) {
    let kinds = unsafe { slice::from_raw_parts(nodes.kind.as_ptr(), nodes.len()) };
    for &kind in kinds {
        if kind != NodeKind::Hole {
            buf[*pos] = wire_kind(kind);
            *pos += 1;
        }
    }
}

fn write_node_payloads(
    wire_map: &WireMap,
    buf: &mut [u8],
    pos: &mut usize,
    nodes: &crate::NodeData<'_>,
    compatibility_extra: &mut [ExtraData],
    mapped_extra: &mut [bool],
    node_edge_count: u32,
) {
    let kinds = unsafe { slice::from_raw_parts(nodes.kind.as_ptr(), nodes.len()) };
    let payloads = unsafe { slice::from_raw_parts(nodes.data.as_ptr(), nodes.len()) };
    for (&kind, &payload) in kinds.iter().zip(payloads) {
        if kind == NodeKind::Hole {
            continue;
        }
        let mut payload = payload;
        kind.map_ast_ids(&mut payload, compatibility_extra, mapped_extra, |id| {
            AstId::from_raw(wire_map.node_id(id))
        });
        if kind.uses_extra_data() {
            let extra_id = (payload.0 >> 96) as u32;
            let relocated = extra_id
                .checked_add(node_edge_count)
                .expect("AST compatibility extra-data index exceeds u32");
            payload.0 = (payload.0 & ((1u128 << 96) - 1)) | ((relocated as u128) << 96);
        }
        write_payload(buf, pos, payload);
    }
}

#[inline]
fn wire_kind(kind: NodeKind) -> u8 {
    let raw = kind as u8;
    if raw > NodeKind::Class as u8 {
        raw.checked_add(1).expect("wire NodeKind exceeds u8")
    } else {
        raw
    }
}

#[inline]
fn range_word<T>(range: TypedNodeRange<T>) -> u64 {
    range.start.index() as u64 | ((range.end.index() as u64) << 32)
}

#[inline]
fn optional_wire_node<T: CommonAstNode>(wire_map: &WireMap, node: Option<T>) -> u64 {
    node.map_or(u32::MAX as u64, |node| {
        wire_map.node_id(node.ast_id()) as u64
    })
}

#[inline]
const fn align_to_four(len: usize) -> usize {
    (len + 3) & !3
}

#[inline]
fn pad_to_four(buf: &mut [u8], pos: &mut usize) {
    let aligned = align_to_four(*pos);
    buf[*pos..aligned].fill(0);
    *pos = aligned;
}

#[inline]
fn write_span(buf: &mut [u8], pos: &mut usize, span: Span) {
    write_u32(buf, pos, span.start);
    write_u32(buf, pos, span.end);
}

#[inline]
fn write_payload(buf: &mut [u8], pos: &mut usize, payload: NodePayload) {
    buf[*pos..*pos + NODE_PAYLOAD_SIZE].copy_from_slice(&payload.0.to_le_bytes());
    *pos += NODE_PAYLOAD_SIZE;
}

#[inline]
fn write_u32(buf: &mut [u8], pos: &mut usize, value: u32) {
    buf[*pos..*pos + size_of::<u32>()].copy_from_slice(&value.to_le_bytes());
    *pos += size_of::<u32>();
}

#[inline]
fn write_u64(buf: &mut [u8], pos: &mut usize, value: u64) {
    buf[*pos..*pos + size_of::<u64>()].copy_from_slice(&value.to_le_bytes());
    *pos += size_of::<u64>();
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;

    use super::*;
    use crate::{
        ArrowFunctionBody, ArrowFunctionExpression, DUMMY_SP, Directive, FormalParameterItem,
        FormalParameterKind, NodeId, Stmt,
    };

    #[test]
    fn shared_extra_ast_ids_are_remapped_once() {
        let allocator = Allocator::new();
        let mut tree = Ast::new("", &allocator);
        let name = tree.add_utf8("value");
        let expression = tree.expr_identifier_reference(DUMMY_SP, name);
        let items = tree.add_typed_sub_range(std::iter::empty::<FormalParameterItem>());
        let params = tree.formal_parameters(
            DUMMY_SP,
            items,
            None,
            FormalParameterKind::ArrowFormalParameters,
        );
        let arrow = tree.arrow_function_expression(
            DUMMY_SP,
            true,
            false,
            None,
            params,
            None,
            ArrowFunctionBody::Expr(expression),
        );

        let payload = unsafe { *tree.node_payload_unchecked(arrow.ast_id()) };
        let _: NodeId<ArrowFunctionExpression> =
            tree.add_node(DUMMY_SP, NodeKind::ArrowFunctionExpression, payload);

        let directives = tree.add_typed_sub_range(std::iter::empty::<Directive>());
        let body = tree.add_typed_sub_range(std::iter::empty::<Stmt>());
        let program = tree.program(DUMMY_SP, directives, body, None);
        tree.set_root_program(program);

        let required = buffer_size(&tree);
        let mut buffer = vec![0; required];
        assert_eq!(serialize_into(&tree, &mut buffer), required);
    }
}
