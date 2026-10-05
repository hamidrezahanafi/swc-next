//! Ambient prefixes retain the real keyword and the actual following declaration owner.

use swc_next_ecma_ast::{BindingIdentifier, Decl, DeclData, GetSpan, Span, VariableDeclarator};

use super::Formatter;
use crate::formatter::comments::CommentBoundary;
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
enum DeclarationLeading {
    Name(BindingIdentifier),
    Variable(VariableDeclarator),
}

impl DeclarationLeading {
    fn span(self, formatter: &Formatter<'_, '_>) -> Span {
        match self {
            Self::Name(node) => node.span(formatter.ast),
            Self::Variable(node) => node.span(formatter.ast),
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::formatter) struct DeclarationKeyword {
    pub(in crate::formatter) span: Span,
    incoming: Option<DeclarationLeading>,
}

impl DeclarationKeyword {
    pub(in crate::formatter) fn suppresses_name(self, name: BindingIdentifier) -> bool {
        matches!(self.incoming, Some(DeclarationLeading::Name(owner)) if owner == name)
    }

    pub(in crate::formatter) fn suppresses_variable(self, node: VariableDeclarator) -> bool {
        matches!(self.incoming, Some(DeclarationLeading::Variable(owner)) if owner == node)
    }
}

enum DeclarationPrefix<'ast> {
    Source(Span),
    Formatted(CommentBoundary<'ast>),
}

pub(super) struct AmbientDeclarationHead<'ast> {
    pub(super) keyword: DeclarationKeyword,
    prefix: DeclarationPrefix<'ast>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn prepare_ambient_declaration_keyword(
        &self,
        declaration: Decl,
    ) -> Result<AmbientDeclarationHead<'ast>, FormatError> {
        let (keyword, following) = match self.ast.decl_data(declaration) {
            DeclData::TsTypeAliasDeclaration(node) => {
                ("type", DeclarationLeading::Name(node.id(self.ast)))
            }
            DeclData::TsInterfaceDeclaration(node) => {
                ("interface", DeclarationLeading::Name(node.id(self.ast)))
            }
            DeclData::VariableDeclaration(node) => {
                let Some(first) = node.declarators(self.ast).first() else {
                    return Err(self.unsupported("empty ambient variable declaration", declaration));
                };
                (
                    node.kind(self.ast).as_str(),
                    DeclarationLeading::Variable(self.ast.get_node_in_sub_range(first)),
                )
            }
            _ => return Err(self.unsupported("ambient declaration prefix", declaration)),
        };
        let start = declaration.span(self.ast).start + "declare".len() as u32;
        let following_span = following.span(self);
        let gap = Span::new(start, following_span.start);
        let comments = self.comments.in_range(gap);
        let keyword_start = self
            .find_token(gap, comments, keyword)
            .ok_or_else(|| self.unsupported("ambient declaration keyword", declaration))?;
        let prefix_span = Span::new(start, keyword_start);
        let prefix_comments = self.comments.in_range(prefix_span);
        let suppressed =
            self.has_suppression && self.content_mode(prefix_comments) == ContentMode::Verbatim;
        let prefix = if suppressed {
            DeclarationPrefix::Source(prefix_span)
        } else {
            DeclarationPrefix::Formatted(self.prepare_comment_boundary(
                prefix_comments,
                start,
                keyword_start,
            ))
        };
        Ok(AmbientDeclarationHead {
            keyword: DeclarationKeyword {
                span: Span::new(keyword_start, keyword_start + keyword.len() as u32),
                incoming: suppressed.then_some(following),
            },
            prefix,
        })
    }

    pub(super) fn format_ambient_declaration_prefix(
        &mut self,
        head: &AmbientDeclarationHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        writer.text("declare");
        match &head.prefix {
            DeclarationPrefix::Source(span) => self.format_verbatim(*span, writer),
            DeclarationPrefix::Formatted(boundary) => {
                self.format_comment_boundary(boundary, writer)?;
                boundary
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
                Ok(())
            }
        }
    }
}
