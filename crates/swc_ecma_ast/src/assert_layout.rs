use crate::{
    CallExpression, Expr, ExpressionStatement, ExtraData, NodeId, NodePayload, Program, Stmt,
};

const _: () = {
    assert!(size_of::<NodePayload>() == 16);
    assert!(align_of::<NodePayload>() == align_of::<u128>());
    assert!(size_of::<ExtraData>() == 8);
    assert!(size_of::<Program>() == size_of::<NodeId>());
    assert!(size_of::<Stmt>() == size_of::<NodeId>());
    assert!(size_of::<Expr>() == size_of::<NodeId>());
    assert!(size_of::<NodeId<CallExpression>>() == 4);
    assert!(size_of::<NodeId<ExpressionStatement>>() == 4);
};
