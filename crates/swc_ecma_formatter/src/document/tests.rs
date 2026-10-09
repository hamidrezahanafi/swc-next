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

fn dependent_document(options: FormatOptions) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.text("target =");
        let separator = writer.named_group(false, |writer| {
            writer.indent(Writer::line);
        });
        writer.indent_if_break(separator, |writer| write_pair(writer, "first", "second"));
    });
    writer.text(";");
    writer.finish().print(options)
}

#[test]
fn dependent_indentation() {
    assert_eq!(
        dependent_document(FormatOptions::prettier()),
        "target = [first, second];"
    );

    let medium = FormatOptions {
        line_width: 16,
        ..FormatOptions::prettier()
    };
    assert_eq!(
        dependent_document(medium),
        "target = [\n  first,\n  second,\n];"
    );

    let narrow = FormatOptions {
        line_width: 8,
        ..medium
    };
    assert_eq!(
        dependent_document(narrow),
        "target =\n  [\n    first,\n    second,\n  ];"
    );

    let tabs = FormatOptions {
        indent_style: IndentStyle::Tab,
        ..narrow
    };
    assert_eq!(
        dependent_document(tabs),
        "target =\n\t[\n\t\tfirst,\n\t\tsecond,\n\t];"
    );
}

#[test]
fn named_group_scopes() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let first = writer.named_group(true, |writer| {
        writer.text("first");
        writer.line();
        writer.text("second");
    });
    writer.hard_line();
    writer.named_group(false, |writer| writer.text("third"));
    writer.indent_if_break(first, |writer| {
        writer.hard_line();
        writer.text("last");
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "first\nsecond\nthird\n  last"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let group = writer.flat(|writer| {
        writer.named_group(true, |writer| {
            writer.text("first");
            writer.line();
            writer.text("second");
        })
    });
    writer.indent_if_break(group, |writer| {
        writer.hard_line();
        writer.text("last");
    });
    assert_eq!(
        writer.finish().print(FormatOptions {
            line_width: 8,
            ..FormatOptions::prettier()
        }),
        "first second\nlast"
    );
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
    let document = Doc {
        elements: &elements,
        group_count: 0,
        conditional_lines: false,
    };
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

fn recorded_fill_document(width: u16, inherited: bool, nested: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let inner = nested.then(|| {
        let (group, ()) = writer
            .record_group_with_value(|writer| {
                writer.fill(|writer| {
                    writer.fill_entry(|writer| writer.text("aa"));
                    writer.fill_entry(Writer::line);
                    writer.fill_entry(|writer| writer.text("bb"));
                });
                Ok::<_, std::convert::Infallible>(())
            })
            .unwrap();
        group.contents
    });
    let (group, ()) = writer
        .record_group_with_value(|writer| {
            writer.fill(|writer| {
                writer.fill_entry(|writer| match inner {
                    Some(fragment) => writer.write_fragment(fragment),
                    None => writer.text("aa"),
                });
                if inner.is_none() {
                    writer.fill_entry(Writer::line);
                    writer.fill_entry(|writer| writer.text("bb"));
                }
                writer.fill_entry(Writer::line);
                writer.fill_entry(|writer| writer.text("cc"));
            });
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap();
    let fragment = if inherited {
        group.contents
    } else {
        group.grouped
    };
    let relocated = record(&mut writer, |writer| {
        writer.text("p:");
        writer.hard_line();
        writer.write_fragment(fragment);
    });
    writer.group(true, |writer| writer.write_fragment(relocated));
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
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

    assert_eq!(recorded_fill_document(5, false, false), "p:\naa bb\ncc");

    assert_eq!(recorded_fill_document(5, true, false), "p:\naa bb\ncc");

    assert_eq!(recorded_fill_document(80, false, false), "p:\naa bb cc");

    assert_eq!(recorded_fill_document(80, true, false), "p:\naa bb cc");

    assert_eq!(recorded_fill_document(5, false, true), "p:\naa bb\ncc");

    assert_eq!(recorded_fill_document(5, true, true), "p:\naa bb\ncc");

    assert_eq!(recorded_fill_document(80, false, true), "p:\naa bb cc");

    assert_eq!(recorded_fill_document(80, true, true), "p:\naa bb cc");
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

fn current_mode_fill(width: u16, expand_next: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(true, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| writer.text("aa"));
            writer.fill_entry(|writer| {
                writer.if_break(",");
                writer.if_flat("unused");
                writer.if_current_break("!");
                writer.if_current_flat(" ");
                writer.soft_line();
            });
            writer.fill_entry(|writer| writer.group(expand_next, |writer| writer.text("bb")));
        });
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn current_mode_group(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.group(false, |writer| {
        writer.if_current_break("B");
        writer.if_current_flat("1234");
        writer.line();
        writer.text("x");
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn current_mode_scopes() {
    assert_eq!(current_mode_fill(8, false), "aa, bb");

    assert_eq!(current_mode_fill(5, false), "aa,!\nbb");

    assert_eq!(current_mode_fill(8, true), "aa,!\nbb");

    assert_eq!(current_mode_group(6), "1234 x");

    assert_eq!(current_mode_group(5), "B\nx");
}

fn record<'a>(writer: &mut Writer<'a>, write: impl FnOnce(&mut Writer<'a>)) -> super::Fragment<'a> {
    writer
        .record_with_value(|writer| {
            write(writer);
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap()
        .0
}

#[test]
fn fragment_capabilities() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let atom = record(&mut writer, |writer| {
        writer.group(false, |writer| {
            writer.text("宽\tvalue");
            writer.space();
            writer.text("tail");
        });
    });
    assert!(atom.try_neutral().is_some());
    assert!(atom.try_flat().is_some());

    let nested = record(&mut writer, |writer| {
        writer.write_fragment(atom);
        writer.write_flat_fragment(atom.try_flat().unwrap());
    });
    assert!(nested.try_neutral().is_some());

    let conditional = record(&mut writer, |writer| {
        writer.text("value");
        writer.if_break(",");
        writer.if_flat(".");
    });
    assert!(conditional.try_neutral().is_none());
    assert!(conditional.try_flat().is_some());

    let flat_conditional = record(&mut writer, |writer| {
        writer.write_flat_fragment(conditional.try_flat().unwrap());
    });
    assert!(flat_conditional.try_neutral().is_none());
    assert!(flat_conditional.try_flat().is_some());

    let current = record(&mut writer, |writer| {
        writer.if_current_break("broken\nbranch");
        writer.if_current_flat("flat");
    });
    assert!(current.try_neutral().is_none());
    assert!(current.try_flat().is_some());

    let projected = record(&mut writer, |writer| {
        writer.flat(|writer| writer.write_fragment(current));
    });
    assert!(projected.try_neutral().is_none());
    assert!(projected.try_flat().is_some());
    writer.write_flat_fragment(projected.try_flat().unwrap());

    let multiline_current = record(&mut writer, |writer| {
        writer.if_current_flat("first\nlast");
    });
    assert!(multiline_current.try_neutral().is_none());
    assert!(multiline_current.try_flat().is_none());

    let named = record(&mut writer, |writer| {
        writer.named_group(false, |writer| writer.text("value"));
    });
    assert!(named.try_neutral().is_none());
    assert!(named.try_flat().is_some());

    let expanded = record(&mut writer, |writer| {
        writer.group(true, |writer| writer.text("value"));
    });
    assert!(expanded.try_neutral().is_none());
    assert!(expanded.try_flat().is_some());

    let mandatory = record(&mut writer, |writer| {
        writer.text("first");
        writer.hard_line();
        writer.text("last");
    });
    assert!(mandatory.try_neutral().is_none());
    assert!(mandatory.try_flat().is_none());

    let multiline = record(&mut writer, |writer| writer.text("first\u{2028}last"));
    assert!(multiline.try_neutral().is_none());
    assert!(multiline.try_flat().is_none());

    let choices = record(&mut writer, |writer| {
        writer.write_alternatives(atom, &[atom], expanded);
    });
    assert!(choices.try_neutral().is_none());
    assert!(choices.try_flat().is_some());

    let flattened = record(&mut writer, |writer| {
        writer.flat(|writer| {
            writer.group(true, |writer| {
                writer.text("value");
                writer.if_break(",");
                writer.if_flat(".");
                writer.if_current_break("broken\nbranch");
                writer.if_current_flat("flat");
            });
        });
    });
    assert!(flattened.try_neutral().is_some());

    let grouped = writer.group_fragment(false, atom);
    assert!(grouped.try_neutral().is_some());
    assert!(!grouped.preferred_breaks());

    let forced = writer.group_fragment(true, atom);
    assert!(forced.try_neutral().is_none());
    assert!(forced.try_flat().is_some());
    assert!(forced.preferred_breaks());

    let required = writer.group_fragment(false, mandatory);
    assert!(required.try_flat().is_none());
    assert!(required.preferred_breaks());

    let flat = writer.flat_delimited_fragment(
        "(",
        atom.try_flat().unwrap(),
        atom,
        atom.try_flat().unwrap(),
        ")",
    );
    assert!(flat.try_neutral().is_some());

    let projected = writer.flat_delimited_fragment(
        "(",
        expanded.try_flat().unwrap(),
        atom,
        current.try_flat().unwrap(),
        ")",
    );
    assert!(projected.try_neutral().is_none());
    assert!(projected.try_flat().is_some());
    assert!(!projected.preferred_breaks());
    assert!(!projected.conditional_lines);

    let multiline = writer.flat_delimited_fragment(
        "(\n",
        atom.try_flat().unwrap(),
        atom,
        atom.try_flat().unwrap(),
        ")",
    );
    assert!(multiline.try_flat().is_none());

    let fallback = writer.indented_delimited_fragment("(", atom, atom, atom, ")");
    assert!(fallback.try_neutral().is_none());
    assert!(fallback.try_flat().is_some());

    let required = writer.indented_delimited_fragment("(", atom, mandatory, atom, ")");
    assert!(required.try_flat().is_none());
    assert!(required.preferred_breaks());
}

#[test]
fn current_mode_flat_projection() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let conditional = record(&mut writer, |writer| {
        writer.if_current_break("broken\nbranch");
        writer.if_current_flat("flat");
    });
    writer.write_flat_fragment(conditional.try_flat().unwrap());
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "flat");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    writer.flat(|writer| {
        writer.if_current_break("broken\nbranch");
        writer.if_current_flat("flat");
    });
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "flat");
}

fn recorded_group_document(write: impl FnOnce(&mut Writer<'_>), inherited: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (group, ()) = writer
        .record_group_with_value(|writer| {
            write(writer);
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap();
    writer.group(true, |writer| {
        let fragment = if inherited {
            group.contents
        } else {
            group.grouped
        };
        writer.write_fragment(fragment);
    });
    writer.finish().print(FormatOptions::prettier())
}

fn completed_group_document(write: impl FnOnce(&mut Writer<'_>), expand: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let content = record(&mut writer, write);
    let group = writer.group_fragment(expand, content);
    writer.group(true, |writer| writer.write_fragment(group));
    writer.finish().print(FormatOptions::prettier())
}

fn write_group_lines(writer: &mut Writer<'_>) {
    writer.text("first");
    writer.line();
    writer.if_flat("flat");
    writer.if_break("broken");
}

#[test]
fn recorded_group_layouts() {
    assert_eq!(
        recorded_group_document(write_group_lines, false),
        "first flat"
    );

    assert_eq!(
        recorded_group_document(write_group_lines, true),
        "first\nbroken"
    );

    assert_eq!(
        completed_group_document(write_group_lines, false),
        "first flat"
    );

    assert_eq!(
        completed_group_document(write_group_lines, true),
        "first\nbroken"
    );
}

fn write_group_suffix(writer: &mut Writer<'_>) {
    writer.text("first");
    writer.line_suffix("// trailing");
    writer.soft_line();
    writer.text("last");
}

#[test]
fn recorded_group_boundaries() {
    assert_eq!(
        recorded_group_document(write_group_suffix, false),
        "first // trailing\nlast"
    );

    assert_eq!(
        recorded_group_document(write_group_suffix, true),
        "first // trailing\nlast"
    );

    assert_eq!(
        completed_group_document(write_group_suffix, false),
        "first // trailing\nlast"
    );
}

fn write_group_named_scope(writer: &mut Writer<'_>) {
    let group = writer.named_group(true, |writer| {
        writer.text("first");
        writer.line();
        writer.text("last");
    });
    writer.soft_line_if_break(group);
    writer.text("tail");
}

#[test]
fn recorded_group_named_scopes() {
    assert_eq!(
        recorded_group_document(write_group_named_scope, false),
        "first\nlast\ntail"
    );

    assert_eq!(
        recorded_group_document(write_group_named_scope, true),
        "first\nlast\ntail"
    );

    assert_eq!(
        completed_group_document(write_group_named_scope, false),
        "first\nlast\ntail"
    );
}

fn completed_delimiter_document(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let prefix = record(&mut writer, |writer| {
        writer.soft_line();
        writer.text("source,");
        writer.line();
    });
    let content = record(&mut writer, |writer| {
        writer.text("{");
        writer.indent(|writer| {
            writer.line();
            writer.text("value");
        });
        writer.line();
        writer.text("}");
    });
    let suffix = record(&mut writer, Writer::soft_line);
    let normal = writer.group_fragment(false, content);
    let forced = writer.group_fragment(true, content);
    let fallback = writer.indented_delimited_fragment("(", prefix, normal, suffix, ")");
    let prefix = prefix.try_flat().unwrap();
    let suffix = suffix.try_flat().unwrap();
    let grouped = writer.flat_delimited_fragment("(", prefix, normal, suffix, ")");
    let expanded = writer.flat_delimited_fragment("(", prefix, forced, suffix, ")");
    writer.write_alternatives(grouped, &[grouped, expanded], fallback);
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn completed_delimiter_layouts() {
    assert_eq!(completed_delimiter_document(20), "(source, { value })");

    assert_eq!(completed_delimiter_document(12), "(source, {\n  value\n})");

    assert_eq!(
        completed_delimiter_document(8),
        "(\n  source,\n  {\n    value\n  }\n)"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (prefix, ()) = writer
        .record_with_value(|writer| {
            let group = writer.named_group(true, |writer| writer.text("source"));
            writer.soft_line_if_break(group);
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap();
    let body = record(&mut writer, |writer| writer.text("value"));
    let suffix = record(&mut writer, |writer| writer.text("tail"));
    let projected = writer.flat_delimited_fragment(
        "(",
        prefix.try_flat().unwrap(),
        body,
        suffix.try_flat().unwrap(),
        ")",
    );
    assert!(!projected.preferred_breaks());
    assert!(!projected.conditional_lines);
    assert!(projected.try_neutral().is_none());
    let unused = writer.indented_delimited_fragment("(", prefix, body, suffix, ")");
    assert!(unused.preferred_breaks());
    assert!(unused.conditional_lines);
    writer.write_fragment(projected);
    let doc = writer.finish();
    assert_eq!(doc.group_count, 1);
    assert!(!doc.conditional_lines);
    assert_eq!(doc.print(FormatOptions::prettier()), "(sourcevaluetail)");
}

fn neutral_continuation(width: u16, neutral: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let fragment = record(&mut writer, |writer| {
        writer.group(false, |writer| writer.text("abc"));
    });
    writer.group(false, |writer| {
        if neutral {
            writer.write_neutral_fragment(fragment.try_neutral().unwrap());
        } else {
            writer.write_fragment(fragment);
        }
        writer.line();
        writer.if_flat("flat");
        writer.if_break("broken");
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn neutral_named_scope(neutral: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let fragment = record(&mut writer, |writer| {
        writer.group(false, |writer| writer.text("宽\tz"));
    });
    let group = writer.named_group(true, |writer| {
        writer.text(">");
        if neutral {
            writer.write_neutral_fragment(fragment.try_neutral().unwrap());
        } else {
            writer.write_fragment(fragment);
        }
        writer.line();
        writer.if_flat("flat");
        writer.if_break("broken");
    });
    writer.soft_line_if_break(group);
    writer.text("after");
    writer.finish().print(FormatOptions::prettier())
}

fn neutral_fill(width: u16, neutral: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let fragment = record(&mut writer, |writer| {
        writer.group(false, |writer| writer.text("abc"));
    });
    writer.group(false, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| {
                if neutral {
                    writer.write_neutral_fragment(fragment.try_neutral().unwrap());
                } else {
                    writer.write_fragment(fragment);
                }
            });
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.text("xy"));
        });
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn neutral_replay() {
    assert_eq!(neutral_continuation(8, false), "abc flat");
    assert_eq!(
        neutral_continuation(8, true),
        neutral_continuation(8, false)
    );

    assert_eq!(neutral_continuation(7, false), "abc\nbroken");
    assert_eq!(
        neutral_continuation(7, true),
        neutral_continuation(7, false)
    );

    assert_eq!(neutral_named_scope(false), ">宽\tz\nbroken\nafter");
    assert_eq!(neutral_named_scope(true), neutral_named_scope(false));

    assert_eq!(neutral_fill(6, false), "abc xy");
    assert_eq!(neutral_fill(6, true), neutral_fill(6, false));

    assert_eq!(neutral_fill(5, false), "abc\nxy");
    assert_eq!(neutral_fill(5, true), neutral_fill(5, false));
}

fn retained_pair(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let pair = record(&mut writer, |writer| write_pair(writer, "aa", "bb"));
    writer.write_fragment(pair);
    writer.text(";");
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn retained_fill(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let inner = record(&mut writer, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| write_pair(writer, "a", "b"));
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.text("x"));
        });
    });
    let outer = record(&mut writer, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| writer.write_fragment(inner));
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.text("y"));
        });
    });
    writer.text("prefix:");
    writer.hard_line();
    writer.write_fragment(outer);
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

fn rejected_candidate(options: FormatOptions, alternatives: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (preferred, first) = writer
        .record_with_value(|writer| {
            let group = writer.named_group(false, |writer| {
                writer.text("a");
                writer.indent(|writer| {
                    writer.align(3, |writer| {
                        writer.line_suffix("// rejected");
                        writer.text("too wide for the available line");
                        writer.hard_line();
                        writer.text("unselected");
                    })
                });
            });
            Ok::<_, std::convert::Infallible>(group)
        })
        .unwrap();
    let (fallback, selected) = writer
        .record_with_value(|writer| {
            let group = writer.named_group(true, |writer| {
                writer.text("a");
                writer.indent(|writer| {
                    writer.line();
                    writer.text("b");
                });
            });
            Ok::<_, std::convert::Infallible>(group)
        })
        .unwrap();
    if alternatives {
        writer.write_alternatives(preferred, &[preferred], fallback);
    } else {
        writer.write_fragment(fallback);
    }
    writer.indent_if_break(first, |writer| {
        writer.hard_line();
        writer.text("first");
    });
    writer.indent_if_break(selected, |writer| {
        writer.hard_line();
        writer.text("selected");
    });
    writer.finish().print(options)
}

#[test]
fn retained_scopes() {
    assert_eq!(retained_pair(9), "[aa, bb];");

    assert_eq!(retained_pair(8), "[\n  aa,\n  bb,\n];");

    assert_eq!(retained_fill(20), "prefix:\n[a, b] x y");

    assert_eq!(retained_fill(6), "prefix:\n[a, b]\nx\ny");
}

#[test]
fn candidate_state() {
    let narrow = FormatOptions {
        line_width: 8,
        ..FormatOptions::prettier()
    };
    assert_eq!(
        rejected_candidate(narrow, true),
        "a\n  b\nfirst\n  selected"
    );
    assert_eq!(
        rejected_candidate(narrow, true),
        rejected_candidate(narrow, false)
    );

    let tabs = FormatOptions {
        indent_style: IndentStyle::Tab,
        ..narrow
    };
    assert_eq!(rejected_candidate(tabs, true), "a\n\tb\nfirst\n\tselected");
    assert_eq!(
        rejected_candidate(tabs, true),
        rejected_candidate(tabs, false)
    );
}

fn source_break_fill() -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let fragment = record(&mut writer, |writer| {
        writer.text("// required");
        writer.break_parent();
        writer.soft_line();
        writer.text("next");
    });
    writer.group(false, |writer| {
        writer.fill(|writer| {
            writer.fill_entry(|writer| writer.write_fragment(fragment));
            writer.fill_entry(Writer::line);
            writer.fill_entry(|writer| writer.text("tail"));
        });
    });
    writer.finish().print(FormatOptions::prettier())
}

fn source_break_candidate(expanded: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preferred = record(&mut writer, |writer| {
        let contents = |writer: &mut Writer<'_>| {
            writer.text("// required");
            writer.break_parent();
            writer.soft_line();
            writer.text("next");
        };
        if expanded {
            writer.group(true, |writer| {
                contents(writer);
                writer.if_break(" broken");
            });
        } else {
            contents(writer);
        }
    });
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    writer.group(false, |writer| {
        writer.write_alternatives(preferred, &[preferred], fallback)
    });
    writer.finish().print(FormatOptions::prettier())
}

#[test]
fn retained_flat_scopes() {
    assert_eq!(source_break_fill(), "// required\nnext\ntail");

    assert_eq!(source_break_candidate(false), "fallback");

    assert_eq!(source_break_candidate(true), "// required\nnext broken");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let mandatory = record(&mut writer, |writer| {
        writer.text("a");
        writer.hard_line();
        writer.text("b");
    });
    assert!(mandatory.try_flat().is_none());
    let literal = record(&mut writer, |writer| writer.literal_text("a\nb"));
    assert!(literal.try_flat().is_none());
    let raw = record(&mut writer, |writer| writer.text("a\nb"));
    assert!(raw.try_flat().is_none());

    let carriage_return = record(&mut writer, |writer| writer.text("a\rb"));
    assert!(carriage_return.try_flat().is_none());

    let line_separator = record(&mut writer, |writer| writer.text("a\u{2028}b"));
    assert!(line_separator.try_flat().is_none());

    let paragraph_separator = record(&mut writer, |writer| writer.text("a\u{2029}b"));
    assert!(paragraph_separator.try_flat().is_none());

    let unicode = record(&mut writer, |writer| writer.text("宽文字"));
    assert!(unicode.try_flat().is_some());

    let tab = record(&mut writer, |writer| writer.text("a\tb"));
    assert!(tab.try_flat().is_some());

    let suffix = record(&mut writer, |writer| {
        writer.text("a");
        writer.line_suffix("// kept");
        writer.line();
        writer.text("b");
    });
    assert!(suffix.try_flat().is_none());
    let source_break = record(&mut writer, |writer| {
        writer.group(false, |writer| {
            writer.text("// required line");
            writer.break_parent();
            writer.soft_line();
            writer.text("next");
        })
    });
    assert!(source_break.try_flat().is_none());
    let compact = record(&mut writer, |writer| write_pair(writer, "a", "b"));
    let choices = record(&mut writer, |writer| {
        writer.write_alternatives(compact, &[compact], mandatory)
    });
    let flat = choices.try_flat().unwrap();
    writer.group(false, |writer| {
        writer.text("before");
        writer.line();
        writer.write_flat_fragment(flat);
        writer.line();
        writer.text("after");
    });
    writer.hard_line();
    writer.write_fragment(mandatory);
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "before [a, b] after\na\nb"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preferred = record(&mut writer, |writer| write_pair(writer, "first", "second"));
    let fallback = record(&mut writer, |writer| {
        writer.text("fallback");
        writer.hard_line();
        writer.text("break");
    });
    writer.flat(|writer| writer.write_alternatives(preferred, &[preferred], fallback));
    assert_eq!(
        writer.finish().print(FormatOptions {
            line_width: 3,
            ..FormatOptions::prettier()
        }),
        "[first, second]"
    );

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let first = record(&mut writer, |writer| {
        writer.text("required");
        writer.hard_line();
        writer.text("line");
    });
    let fallback = record(&mut writer, |writer| writer.text("flat"));
    let choices = record(&mut writer, |writer| {
        writer.write_alternatives(first, &[first], fallback)
    });
    assert!(choices.try_flat().is_none());
    writer.flat(|writer| writer.write_fragment(choices));
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "required\nline"
    );
}

fn choice_continuation(width: u16) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preferred = record(&mut writer, |writer| writer.text("abc"));
    let fallback = record(&mut writer, |writer| {
        writer.group(true, |writer| {
            writer.text("a");
            writer.line();
            writer.text("b");
        })
    });
    let choices = record(&mut writer, |writer| {
        writer.write_alternatives(preferred, &[preferred], fallback)
    });
    writer.group(false, |writer| {
        writer.write_fragment(choices);
        writer.line();
        writer.if_flat("flat");
        writer.if_break("broken");
    });
    writer.finish().print(FormatOptions {
        line_width: width,
        ..FormatOptions::prettier()
    })
}

#[test]
fn candidate_continuation() {
    assert_eq!(choice_continuation(8), "abc flat");
    assert_eq!(choice_continuation(7), "abc\nbroken");
    assert_eq!(choice_continuation(2), "a\nb\nbroken");
}

fn named_line_reuse(alternatives: bool) -> String {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (signature, group) = writer
        .record_with_value(|writer| {
            let group = writer.named_group(false, |writer| {
                writer.text("a");
                writer.line();
                writer.text("b");
            });
            Ok::<_, std::convert::Infallible>(group)
        })
        .unwrap();
    writer.text("123456");
    writer.write_fragment(signature);
    writer.hard_line();
    let fallback = record(&mut writer, |writer| {
        writer.write_fragment(signature);
        writer.soft_line_if_break(group);
        writer.text("123456");
    });
    if alternatives {
        let preferred = record(&mut writer, |writer| writer.text("rejected candidate"));
        writer.write_alternatives(preferred, &[preferred], fallback);
    } else {
        writer.write_fragment(fallback);
    }
    writer.finish().print(FormatOptions {
        line_width: 8,
        ..FormatOptions::prettier()
    })
}

#[test]
fn named_line_scopes() {
    assert_eq!(named_line_reuse(false), "123456a\nb\na\nb\n123456");
    assert_eq!(named_line_reuse(true), named_line_reuse(false));

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let (named, id) = writer
        .record_with_value(|writer| {
            let id = writer.named_group(false, |writer| writer.text("a"));
            Ok::<_, std::convert::Infallible>(id)
        })
        .unwrap();
    writer.group(true, |writer| {
        writer.write_fragment(named);
        writer.soft_line_if_break(id);
        writer.text("b");
    });
    let fragment = record(&mut writer, |writer| {
        writer.named_group(true, |writer| writer.text("c"));
        writer.soft_line_if_break(id);
    });
    writer.write_flat_fragment(fragment.try_flat().unwrap());
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "abc");

    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let id = writer.named_group(true, |writer| writer.text("a"));
    writer.group(false, |writer| {
        writer.soft_line_if_break(id);
        writer.text("b");
    });
    assert_eq!(writer.finish().print(FormatOptions::prettier()), "ab");
}

#[test]
fn alternative_expansion_boundaries() {
    let allocator = Allocator::new();
    let mut writer = Writer::new(&allocator);
    let preferred = record(&mut writer, |writer| {
        writer.group(true, |writer| {
            writer.text("body");
            writer.line();
            writer.text("value");
        });
    });
    let fallback = record(&mut writer, |writer| writer.text("fallback"));
    let scope = record(&mut writer, |writer| {
        writer.write_alternatives(fallback, &[preferred], fallback)
    });
    assert!(preferred.preferred_breaks());
    assert!(!scope.preferred_breaks());
    writer.group(false, |writer| {
        writer.text("head ");
        writer.if_flat("F");
        writer.write_fragment(scope);
        writer.if_current_flat(" tail");
    });
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "head Fbody\nvalue tail"
    );

    let mut writer = Writer::new(&allocator);
    let normal = preferred;
    let inherited = record(&mut writer, |writer| {
        writer.write_alternatives(normal, &[normal], fallback);
    });
    assert!(inherited.preferred_breaks());
    let flat = inherited
        .try_flat()
        .expect("expansion metadata permits a flat view");
    writer.write_flat_fragment(flat);
    assert_eq!(
        writer.finish().print(FormatOptions::prettier()),
        "body value"
    );
}
