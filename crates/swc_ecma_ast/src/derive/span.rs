use crate::DUMMY_SP;

pub trait GetSpan<'id>: Copy {
    fn span(self, ast: &crate::Ast<'_, 'id>) -> crate::Span;

    #[inline]
    fn span_lo(self, ast: &crate::Ast<'_, 'id>) -> u32 {
        self.span(ast).start
    }

    #[inline]
    fn span_hi(self, ast: &crate::Ast<'_, 'id>) -> u32 {
        self.span(ast).end
    }
}

pub trait SetSpan<'id>: Copy {
    fn set_span(self, ast: &mut crate::Ast<'_, 'id>, span: crate::Span);
}

impl<'id, S> GetSpan<'id> for Option<S>
where
    S: GetSpan<'id>,
{
    #[inline]
    fn span(self, ast: &crate::Ast<'_, 'id>) -> crate::Span {
        match self {
            Some(s) => s.span(ast),
            None => DUMMY_SP,
        }
    }
}
