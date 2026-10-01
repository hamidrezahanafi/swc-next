use swc_next_allocator::Allocator;

use super::{Doc, FormatElement, Writer};
use crate::{FormatOptions, IndentStyle, LineEnding};

fn nested_document(options: FormatOptions) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("{");
    writer.indent(|writer| {
        writer.hard_line();
        writer.text("{");
        writer.indent(|writer| {
            writer.hard_line();
            writer.text("value;");
        });
        writer.hard_line();
        writer.text("}");
    });
    writer.hard_line();
    writer.text("}");
    writer.hard_line();
    writer.finish().print(options)
}

fn line_document(options: FormatOptions) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("first");
    writer.hard_line();
    writer.text("second");
    writer.hard_line();
    writer.finish().print(options)
}

fn write_pair<'a>(writer: &mut Writer<'a>, first: &'a str, last: &'a str) {
    writer.group(false, |writer| {
        writer.text("[");
        writer.indent(|writer| {
            writer.soft_line();
            writer.text(first);
            writer.text(",");
            writer.line();
            writer.text(last);
            writer.if_break(",");
        });
        writer.soft_line();
        writer.text("]");
    });
}

fn pair_document(first: &str, last: &str, width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    write_pair(&mut writer, first, last);
    writer.text(";");
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn nested_group_document(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(true, |writer| {
        writer.text("[");
        writer.indent(|writer| {
            writer.soft_line();
            write_pair(writer, "aa", "bb");
            writer.text(",");
            writer.line();
            writer.text("tail");
            writer.if_break(",");
        });
        writer.soft_line();
        writer.text("]");
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn text_fragments() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("first");
    writer.text("");
    writer.text(" second");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first second"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("first\r\nsecond");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first\r\nsecond"
    );
}

#[test]
fn generated_text() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let source = String::from("source");
    writer.text(source.as_str());
    writer.text(format!(" {source}"));
    writer.text(" ");
    writer.text("x".repeat(32 * 1024));
    writer.text(String::from(" end"));
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        format!("source source {} end", "x".repeat(32 * 1024))
    );
}

#[test]
fn line_endings() {
    let lf = FormatOptions::prettier();
    assert_eq!(line_document(lf), "first\nsecond\n");

    let crlf = FormatOptions {
        line_ending: LineEnding::CrLf,
        ..lf
    };
    assert_eq!(line_document(crlf), "first\r\nsecond\r\n");

    let cr = FormatOptions {
        line_ending: LineEnding::Cr,
        ..lf
    };
    assert_eq!(line_document(cr), "first\rsecond\r");
}

#[test]
fn indentation() {
    let spaces = FormatOptions::prettier();
    assert_eq!(nested_document(spaces), "{\n  {\n    value;\n  }\n}\n");

    let tabs = FormatOptions {
        indent_style: IndentStyle::Tab,
        ..spaces
    };
    assert_eq!(nested_document(tabs), "{\n\t{\n\t\tvalue;\n\t}\n}\n");

    let zero = FormatOptions {
        indent_width: 0.try_into().unwrap(),
        ..spaces
    };
    assert_eq!(nested_document(zero), "{\n{\nvalue;\n}\n}\n");

    let crlf = FormatOptions {
        line_ending: LineEnding::CrLf,
        ..spaces
    };
    assert_eq!(
        nested_document(crlf),
        "{\r\n  {\r\n    value;\r\n  }\r\n}\r\n"
    );
}

#[test]
fn pending_whitespace() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("first");
    writer.space();
    writer.hard_line();
    writer.indent(|writer| {
        writer.hard_line();
        writer.space();
        writer.hard_line();
        writer.text("last");
        writer.space();
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first\n\n\n  last"
    );
}

#[test]
fn deep_indentation() {
    let mut elements = vec![FormatElement::StartIndent; 1025];
    elements.push(FormatElement::HardLine);
    elements.push(FormatElement::Text(super::text::Text::new("value")));
    elements.extend(std::iter::repeat_n(FormatElement::EndIndent, 1025));
    let document = Doc { elements };
    let options = FormatOptions {
        indent_width: 64.try_into().unwrap(),
        ..FormatOptions::prettier()
    };
    let output = document.print(options);
    assert_eq!(output.len(), 1 + 1025 * 64 + "value".len());
    assert!(output.starts_with('\n'));
    assert!(output.ends_with("value"));
    assert!(
        output[1..output.len() - "value".len()]
            .bytes()
            .all(|byte| byte == b' ')
    );
}

#[test]
fn literal_lines() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("{");
    writer.indent(|writer| {
        writer.hard_line();
        writer.text("/* first");
        writer.space();
        writer.literal_line();
        writer.text(" original */");
        writer.hard_line();
        writer.text("value;");
    });
    writer.hard_line();
    writer.text("}");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "{\n  /* first\n original */\n  value;\n}"
    );
}

#[test]
fn group_continuation() {
    assert_eq!(pair_document("aa", "bb", 9), "[aa, bb];");

    assert_eq!(pair_document("aa", "bb", 8), "[\n  aa,\n  bb,\n];");

    assert_eq!(nested_group_document(11), "[\n  [aa, bb],\n  tail,\n]");

    assert_eq!(
        nested_group_document(10),
        "[\n  [\n    aa,\n    bb,\n  ],\n  tail,\n]"
    );
}

#[test]
fn group_breaks() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("a");
        writer.line();
        writer.group(true, |writer| {
            writer.text("b");
            writer.line();
            writer.text("c");
        });
    });
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "a\nb\nc");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("a");
        writer.space();
        writer.line();
        writer.text("b");
    });
    assert_eq!(
        writer.finish().print(FormatOptions {
            line_width: 3,
            ..FormatOptions::prettier()
        }),
        "a b"
    );
}

#[test]
fn line_suffixes() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("a");
        writer.line_suffix("// comment");
        writer.text(",");
        writer.line();
        writer.text("b");
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "a, // comment\nb"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.text("a");
    writer.line_suffix("// last");
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "a // last"
    );
}

#[test]
fn display_columns() {
    assert_eq!(pair_document("世界", "a", 10), "[世界, a];");

    assert_eq!(pair_document("世界", "a", 9), "[\n  世界,\n  a,\n];");

    assert_eq!(pair_document("é", "a", 7), "[é, a];");

    assert_eq!(pair_document("é", "a", 6), "[\n  é,\n  a,\n];");
}
