use crate::{AstNode, AstUtf8, AstWtf8, SubRange};

use super::{NodeId, RawNodeId, sealed::ExtraDataValue as SealedExtraDataValue};

oxc_index::define_index_type! {
    #[repr(transparent)]
    pub(crate) struct ExtraDataId = u32;
    MAX_INDEX = u32::MAX as usize;
}

/// Unbranded list bounds used only inside AST storage.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct RawSubRange {
    pub(crate) start: ExtraDataId,
    pub(crate) end: ExtraDataId,
}

/// Compact values stored in the flattening AST's extra-data table.
///
/// The AST schema supplies the type information, so the table stores only its
/// 64-bit representation without a runtime discriminant. The low and high
/// words are stored in that order, each in native byte order.
#[repr(C)]
#[derive(Clone, Copy)]
pub union ExtraData {
    node_id: RawNodeId,
    sub_range: RawSubRange,
    alignment: u64,
}

impl ExtraData {
    #[inline]
    pub(crate) fn from_raw(raw: u64) -> Self {
        Self {
            sub_range: RawSubRange {
                start: ExtraDataId::from_raw(raw as u32),
                end: ExtraDataId::from_raw((raw >> 32) as u32),
            },
        }
    }

    #[inline]
    pub(crate) fn raw(self) -> u64 {
        // SAFETY: Both range fields accept every u32 bit pattern, and all
        // constructors initialize the full eight-byte slot.
        let range = unsafe { self.sub_range };
        (range.start.index() as u64) | ((range.end.index() as u64) << 32)
    }
}

/// A value that has an eight-byte representation in the AST extra-data table.
/// Implementations are restricted to the storage types supported by this crate.
pub trait ExtraDataValue<'id>: Sized + SealedExtraDataValue {
    fn into_extra_data(self) -> ExtraData;

    /// # Safety
    /// `data` must have been written for `Self`.
    /// Any node, string, range, or slot handles must refer to the AST
    /// branded by `'id` and satisfy their encoding and range invariants.
    unsafe fn from_extra_data(data: &ExtraData) -> Self;
}

impl<'id, T: AstNode<'id>> SealedExtraDataValue for T {}

impl<'id, T: AstNode<'id>> ExtraDataValue<'id> for T {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.node_id().index() as u64)
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        unsafe { Self::from_node_id_unchecked(NodeId::from_raw(data.node_id)) }
    }
}

impl<'id, T: AstNode<'id>> SealedExtraDataValue for Option<T> {}

impl<'id, T: AstNode<'id>> ExtraDataValue<'id> for Option<T> {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.map_or(u64::from(u32::MAX), |node| node.node_id().index() as u64))
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        let raw = unsafe { data.node_id }.index() as u32;
        (raw != u32::MAX).then(|| unsafe { T::from_node_id_unchecked(NodeId::from_raw_index(raw)) })
    }
}

impl<'id, T> SealedExtraDataValue for SubRange<'id, T> {}

impl<'id, T: ExtraDataValue<'id>> ExtraDataValue<'id> for SubRange<'id, T> {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData {
            sub_range: self.raw(),
        }
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        let range = unsafe { data.sub_range };
        Self::new(range.start, range.end)
    }
}

impl<'id> SealedExtraDataValue for AstUtf8<'id> {}

impl<'id> ExtraDataValue<'id> for AstUtf8<'id> {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.raw())
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        Self::from_raw(data.raw())
    }
}

impl<'id> SealedExtraDataValue for Option<AstUtf8<'id>> {}

impl<'id> ExtraDataValue<'id> for Option<AstUtf8<'id>> {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        // `AstUtf8` ranges always satisfy `start <= end`, so `(u32::MAX, 0)`
        // is available as an unambiguous `None` sentinel.
        ExtraData::from_raw(self.map_or(u64::from(u32::MAX), AstUtf8::raw))
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        let raw = data.raw();
        (raw != u64::from(u32::MAX)).then(|| AstUtf8::from_raw(raw))
    }
}

impl SealedExtraDataValue for f64 {}

impl<'id> ExtraDataValue<'id> for f64 {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.to_bits())
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        Self::from_bits(data.raw())
    }
}

impl<'id> SealedExtraDataValue for AstWtf8<'id> {}

impl<'id> ExtraDataValue<'id> for AstWtf8<'id> {
    #[inline]
    fn into_extra_data(self) -> ExtraData {
        ExtraData::from_raw(self.raw())
    }

    #[inline]
    unsafe fn from_extra_data(data: &ExtraData) -> Self {
        Self::from_raw(data.raw())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Ast, DUMMY_SP, ExtraData, ExtraDataValue, Program, SubRange, make_ast_guard};
    use swc_next_allocator::Allocator;

    #[test]
    fn canonical_storage() {
        assert_eq!(
            ExtraData::from_raw(0x0123_4567_89ab_cdef).raw(),
            0x0123_4567_89ab_cdef
        );
        assert_eq!(ExtraData::from_raw(u64::MAX).raw(), u64::MAX);

        let data = None::<Program<'_>>.into_extra_data();
        assert_eq!(data.raw(), u64::from(u32::MAX));
        assert!(unsafe { Option::<Program<'_>>::from_extra_data(&data) }.is_none());

        let allocator = Allocator::new();
        make_ast_guard!(guard);
        let mut ast = Ast::new("", &allocator, guard);
        ast.numeric_literal(DUMMY_SP, 1.0, None);
        let node = ast.program(DUMMY_SP, SubRange::empty(), SubRange::empty(), None);
        let data = node.into_extra_data();
        assert_eq!(data.raw(), 1);
        // SAFETY: This slot stores the live program from this AST.
        let decoded = unsafe { Program::from_extra_data(&data) };
        assert_eq!(decoded.node_id(), node.node_id());

        ast.add_sub_range([1.0]);
        let range = ast.add_sub_range([node]);
        let data = range.into_extra_data();
        assert_eq!(
            data.raw(),
            (range.start.index() as u64) | ((range.end.index() as u64) << 32)
        );
        // SAFETY: This slot stores this AST's range of programs.
        let decoded = unsafe { SubRange::<Program<'_>>::from_extra_data(&data) };
        assert_eq!(decoded.get_node(&ast, 0).unwrap().node_id(), node.node_id());
    }
}
