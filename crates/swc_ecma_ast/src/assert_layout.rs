use std::mem::{align_of, size_of};

use crate::{
    AstUtf8, AstWtf8, Expr, ExtraData, ExtraDataId, NodeId, NodePayload, Program, Stmt, SubRange,
};

const _: () = {
    assert!(size_of::<crate::node_id::NodeExtraDataId<'static, ()>>() == 4);
    assert!(size_of::<crate::node_id::RawNodeId>() == 4);
    assert!(size_of::<crate::node_id::RawSubRange>() == 8);
    assert!(size_of::<Option<NodeId<'static>>>() == 4);
    assert!(size_of::<NodeId<'static>>() == size_of::<u32>());
    assert!(align_of::<NodeId<'static>>() == align_of::<u32>());
    assert!(size_of::<ExtraDataId>() == size_of::<u32>());
    assert!(align_of::<ExtraDataId>() == align_of::<u32>());
    assert!(size_of::<SubRange<'static, ()>>() == 8);
    assert!(align_of::<SubRange<'static, ()>>() == align_of::<u32>());
    assert!(size_of::<NodePayload>() == 16);
    assert!(align_of::<NodePayload>() == align_of::<u128>());
    assert!(size_of::<ExtraData>() == 8);
    assert!(size_of::<AstUtf8<'static>>() == 8);
    assert!(size_of::<AstWtf8<'static>>() == 8);
    assert!(align_of::<AstUtf8<'static>>() == align_of::<u32>());
    assert!(align_of::<AstWtf8<'static>>() == align_of::<u32>());
    assert!(size_of::<Program<'static>>() == size_of::<NodeId<'static>>());
    assert!(size_of::<Stmt<'static>>() == size_of::<NodeId<'static>>());
    assert!(size_of::<Expr<'static>>() == size_of::<NodeId<'static>>());
};
