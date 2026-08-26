//! Binary AST transfer format used by the JavaScript FFI.
//!
//! Internal AST references are untagged common-node IDs. A common Class owner
//! stores the dense index of its object in Ast::classes. The public JavaScript
//! decoder still consumes the original 36-byte-header format, so serialization
//! adds one virtual ClassBody record per class. No internal ClassBody node is
//! allocated.

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
        tree.root_program().node_id().raw(),
        (if tree.is_ts() { FLAG_TS } else { 0 })
            | ((tree.source_type() as u32) << FLAG_SOURCE_TYPE_SHIFT),
    ] {
        write_u32(buf, &mut pos, value);
    }

    let spans = unsafe {
        slice::from_raw_parts(
            tree.nodes.span.as_ptr().cast::<u8>(),
            tree.node_count() * SPAN_SIZE,
        )
    };
    buf[pos..pos + spans.len()].copy_from_slice(spans);
    pos += spans.len();
    for class in &tree.classes {
        write_span(buf, &mut pos, class.body_span);
    }

    for index in 0..tree.node_count() {
        let kind = unsafe { *tree.nodes.kind.as_ptr().add(index) };
        buf[pos] = wire_kind(kind);
        pos += 1;
    }
    for _ in 0..tree.class_count() {
        buf[pos] = NodeKind::Class as u8 + 1;
        pos += CLASS_COMPAT_NODE_COUNT;
    }
    pad_to_four(buf, &mut pos);

    let mut compatibility_extra = tree.extra_data.as_slice().to_vec();
    let class_extra_base = tree.node_edge_count() + tree.extra_data_count();
    for index in 0..tree.node_count() {
        let kind = unsafe { *tree.nodes.kind.as_ptr().add(index) };
        let mut payload = unsafe { *tree.nodes.data.as_ptr().add(index) };
        if kind == NodeKind::Class {
            let storage_index = payload.0 as u32 as usize;
            let class = &tree.classes[storage_index];
            let extra_id =
                u32::try_from(class_extra_base + storage_index * CLASS_COMPAT_EXTRA_COUNT)
                    .expect("class compatibility extra-data index exceeds u32");
            let id = class
                .id
                .map_or(u32::MAX, |node| wire_node_id(node.ast_id()));
            let type_parameters = class
                .type_parameters
                .map_or(u32::MAX, |node| wire_node_id(node.ast_id()));
            payload = NodePayload(
                (class.class_type as u128)
                    | ((class.r#abstract as u128) << 8)
                    | ((class.declare as u128) << 16)
                    | ((id as u128) << 24)
                    | ((type_parameters as u128) << 56)
                    | ((extra_id as u128) << 96),
            );
        } else {
            kind.map_ast_ids(&mut payload, &mut compatibility_extra, |id| {
                AstId::from_raw(wire_node_id(id))
            });
            if kind.uses_extra_data() {
                let extra_id = (payload.0 >> 96) as u32;
                let relocated = extra_id
                    .checked_add(node_edge_count)
                    .expect("AST compatibility extra-data index exceeds u32");
                payload.0 = (payload.0 & ((1u128 << 96) - 1)) | ((relocated as u128) << 96);
            }
        }
        write_payload(buf, &mut pos, payload);
    }
    for class in &tree.classes {
        write_payload(buf, &mut pos, NodePayload(range_word(class.body) as u128));
    }

    for edge in tree.node_edges.as_slice() {
        write_u32(buf, &mut pos, wire_node_id(*edge));
        write_u32(buf, &mut pos, 0);
    }
    for extra in &compatibility_extra {
        write_u64(buf, &mut pos, unsafe { extra.other });
    }
    for (index, class) in tree.classes.iter().enumerate() {
        for word in [
            range_word(class.decorators),
            optional_wire_node(class.super_class),
            optional_wire_node(class.super_type_arguments),
            range_word(class.implements),
            wire_class_body_id(tree, index) as u64,
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

#[inline]
fn wire_class_body_id(tree: &Ast<'_>, index: usize) -> u32 {
    u32::try_from(tree.node_count() + index).expect("class body wire node index exceeds u32")
}

#[inline]
fn wire_node_id(id: AstId) -> u32 {
    id.raw()
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
fn optional_wire_node<T: CommonAstNode>(node: Option<T>) -> u64 {
    node.map_or(u32::MAX as u64, |node| wire_node_id(node.ast_id()) as u64)
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
