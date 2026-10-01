//! Program syntax emits formatting instructions independently of printing.

use swc_next_ecma_ast::{Ast, GetSpan, NodeKind};

use crate::{FormatError, document::Writer};

pub(crate) fn format_program<'ast>(
    ast: &'ast Ast<'_>,
    writer: &mut Writer<'ast>,
) -> Result<(), FormatError> {
    let program = ast.root_program();
    if !program.body(ast).is_empty()
        || !program.directives(ast).is_empty()
        || !ast.comments().is_empty()
    {
        return Err(FormatError::unsupported(
            "program",
            Some(NodeKind::Program),
            program.span(ast),
        ));
    }

    if ast.source().starts_with('\u{feff}') {
        writer.text("\u{feff}");
    }
    if let Some(hashbang) = program.hashbang(ast) {
        writer.text("#!");
        writer.text(ast.get_utf8(hashbang.value(ast)).trim_end());
        writer.hard_line();
    }
    Ok(())
}
