use crate::{AstNode, NodeId};

/// Converts AST list elements to and from the compact list-data table.
///
/// AST lists only contain node handles (or optional node handles), so each
/// element fits in one `u32`. Optional handles use `u32::MAX` as `None`.
pub trait ListDataCompact: Sized {
    fn to_list_data(self) -> u32;

    /// # Safety
    ///
    /// `data` must have been written for `Self` and, unless it is the optional
    /// sentinel, must identify a node accepted by `Self`.
    unsafe fn from_list_data(data: u32) -> Self;
}

impl<T: AstNode> ListDataCompact for T {
    #[inline]
    fn to_list_data(self) -> u32 {
        self.node_id().index() as u32
    }

    #[inline]
    unsafe fn from_list_data(data: u32) -> Self {
        debug_assert_ne!(data, u32::MAX);
        unsafe { Self::from_node_id_unchecked(NodeId::from_raw_unchecked(data)) }
    }
}

impl<T: AstNode> ListDataCompact for Option<T> {
    #[inline]
    fn to_list_data(self) -> u32 {
        self.map_or(u32::MAX, |node| node.node_id().index() as u32)
    }

    #[inline]
    unsafe fn from_list_data(data: u32) -> Self {
        if data == u32::MAX {
            None
        } else {
            Some(unsafe { T::from_node_id_unchecked(NodeId::from_raw_unchecked(data)) })
        }
    }
}
