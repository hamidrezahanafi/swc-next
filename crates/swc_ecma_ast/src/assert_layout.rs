use crate::{Expr, ExtraData, ListDataId, NodeId, NodePayload, Program, Stmt, SubRange};

const _: () = {
    assert!(size_of::<NodePayload>() == 16);
    assert!(align_of::<NodePayload>() == align_of::<u128>());
    assert!(size_of::<ExtraData>() == 8);
    assert!(size_of::<ListDataId>() == 4);
    assert!(size_of::<SubRange>() == 8);
    assert!(size_of::<Program>() == size_of::<NodeId>());
    assert!(size_of::<Stmt>() == size_of::<NodeId>());
    assert!(size_of::<Expr>() == size_of::<NodeId>());
};
