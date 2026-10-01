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

fn aligned_document(options: FormatOptions) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("prefix");
        writer.indent(|writer| {
            writer.line();
            writer.if_flat("(");
            writer.align(2, |writer| {
                writer.text("first");
                writer.indent(|writer| {
                    writer.line();
                    writer.text("second");
                });
            });
            writer.if_flat(")");
        });
    });
    writer.finish().print(options)
}

#[test]
fn aligned_groups() {
    assert_eq!(
        aligned_document(FormatOptions::prettier()),
        "prefix (first second)"
    );

    let narrow = FormatOptions {
        line_width: 8,
        ..FormatOptions::prettier()
    };
    assert_eq!(aligned_document(narrow), "prefix\n  first\n      second");

    let tabs = FormatOptions {
        indent_style: IndentStyle::Tab,
        ..narrow
    };
    assert_eq!(aligned_document(tabs), "prefix\n\tfirst\n\t\t\tsecond");
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

fn fill_document(parts: &[&str], width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("[");
        writer.indent(|writer| {
            writer.soft_line();
            writer.fill(|writer| {
                let mut parts = parts.iter().peekable();
                let mut first = true;
                while let Some(part) = parts.next() {
                    if !first {
                        writer.fill_entry(|writer| writer.line());
                    }
                    writer.fill_entry(|writer| {
                        writer.text(*part);
                        if parts.peek().is_some() {
                            writer.text(",");
                        } else {
                            writer.if_break(",");
                        }
                    });
                    first = false;
                }
            });
        });
        writer.soft_line();
        writer.text("]");
    });
    writer.text(";");
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn grouped_fill_document(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("[");
        writer.indent(|writer| {
            writer.soft_line();
            writer.fill(|writer| {
                writer.fill_entry(|writer| {
                    writer.group(false, |writer| {
                        writer.text("[a");
                        writer.line();
                        writer.text("b]");
                    });
                    writer.text(",");
                });
                writer.fill_entry(|writer| writer.line());
                writer.fill_entry(|writer| writer.text("x,"));
                writer.fill_entry(|writer| writer.line());
                writer.fill_entry(|writer| {
                    writer.text("y");
                    writer.if_break(",");
                });
            });
        });
        writer.soft_line();
        writer.text("]");
    });
    writer.text(";");
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

#[test]
fn fill_layout() {
    assert_eq!(
        fill_document(&["1", "2", "3", "4", "5", "6"], 20),
        "[1, 2, 3, 4, 5, 6];"
    );

    assert_eq!(
        fill_document(&["1", "2", "3", "4", "5", "6"], 10),
        "[\n  1, 2, 3,\n  4, 5, 6,\n];"
    );

    assert_eq!(
        fill_document(&["1", "2", "3", "4", "5", "6"], 8),
        "[\n  1, 2,\n  3, 4,\n  5, 6,\n];"
    );

    assert_eq!(
        fill_document(&["long_element", "2", "3"], 10),
        "[\n  long_element,\n  2, 3,\n];"
    );

    assert_eq!(grouped_fill_document(20), "[[a b], x, y];");

    assert_eq!(grouped_fill_document(10), "[\n  [a b],\n  x, y,\n];");

    assert_eq!(grouped_fill_document(7), "[\n  [a\n  b],\n  x, y,\n];");
}

#[test]
fn flat_scopes() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("before");
        writer.line();
        writer.flat(|writer| {
            writer.group(true, |writer| write_pair(writer, "first", "second"));
        });
        writer.line();
        writer.text("after");
    });
    assert_eq!(
        writer.finish().print(FormatOptions {
            line_width: 5,
            ..FormatOptions::prettier()
        }),
        "before\n[first, second]\nafter"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.flat(|writer| {
        writer.group(false, |writer| {
            writer.text("first");
            writer.hard_line();
            writer.text("second");
        });
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first\nsecond"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.flat(|writer| {
        writer.group(false, |writer| {
            writer.text("first");
            writer.line_suffix("// comment");
            writer.line();
            writer.text("second");
        });
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first // comment\nsecond"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| {
                writer.flat(|writer| write_pair(writer, "first", "second"));
            });
            writer.fill_entry(|writer| writer.line());
            writer.fill_entry(|writer| writer.text("last"));
        });
    });
    assert_eq!(
        writer.finish().print(FormatOptions {
            line_width: 5,
            ..FormatOptions::prettier()
        }),
        "[first, second]\nlast"
    );
}
