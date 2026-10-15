use crate::Span;
use crate::{Ast, AstUtf8};

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommentKind {
    Line = 0,
    Block = 1,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommentPosition {
    Leading = 0,
    Trailing = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Comment {
    pub span: Span,
    pub attached_to: u32,
    pub kind: CommentKind,
    pub position: CommentPosition,
}

impl Comment {
    pub fn new(span: Span, kind: CommentKind) -> Self {
        Self {
            span,
            kind,
            attached_to: 0,
            position: CommentPosition::Trailing,
        }
    }

    pub fn value<'id>(&self, ast: &Ast<'_, 'id>) -> AstUtf8<'id> {
        let source = ast.source().as_bytes();
        let start = self.span.start;
        let head = match source[start as usize] {
            b'<' => 4,
            b'-' => 3,
            _ => 2,
        };
        let tail = if self.kind == CommentKind::Block {
            2
        } else {
            0
        };

        ast.source_slice(self.span.start + head, self.span.end - tail)
    }
}
