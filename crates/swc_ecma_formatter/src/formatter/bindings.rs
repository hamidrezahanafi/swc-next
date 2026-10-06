//! Declaration bindings and their parameter-owned punctuation boundaries.

use swc_next_ecma_ast::{
    ArrayPattern, BindingPattern, BindingPatternData, BindingProperty, CommentKind, Expr,
    FormalParameterPattern, FormalParameterPatternData, GetSpan, ObjectPattern, PropertyKey, Span,
    TsEnumMemberName,
};

use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::arguments::{ArgumentLayout, ElisionLayout, FollowingItem, Separator};
use super::expressions::assignment::AssignmentBoundary;
use super::expressions::{ExpressionContext, ExpressionPosition};
use super::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, TrailingCommas, document::Writer};

enum DefaultOwner<'ast> {
    Whole,
    Bound {
        gap: Span,
        left_mode: ContentMode,
        right_mode: ContentMode,
    },
    Formatted {
        boundary: AssignmentBoundary<'ast>,
        break_after: bool,
    },
}

#[derive(Clone, Copy)]
pub(super) enum InitializerLeft {
    Binding(BindingPattern),
    Enum {
        name: TsEnumMemberName,
        computed: Option<Span>,
    },
    Property {
        key: PropertyKey,
        computed: Option<Span>,
    },
}

pub(super) enum InitializerLayout<'ast> {
    Formatted {
        boundary: AssignmentBoundary<'ast>,
        break_after: bool,
    },
    Bound {
        gap: Span,
        right_mode: ContentMode,
    },
}

pub(super) struct PreparedInitializer<'ast> {
    pub(super) left: InitializerLeft,
    pub(super) right: Expr,
    pub(super) layout: InitializerLayout<'ast>,
    pub(super) left_mode: ContentMode,
    pub(super) context: ExpressionContext,
}

pub(super) struct PreparedBindingDefault<'ast> {
    left: BindingPattern,
    right: Expr,
    span: Span,
    owner: DefaultOwner<'ast>,
}

impl PreparedBindingDefault<'_> {
    pub(super) fn is_verbatim(&self) -> bool {
        matches!(self.owner, DefaultOwner::Whole)
    }
}

struct BindingSlot {
    holes: usize,
    binding: Option<BindingPattern>,
}

fn next_slot(elements: &mut impl Iterator<Item = Option<BindingPattern>>) -> Option<BindingSlot> {
    let mut holes = 0;
    for element in elements.by_ref() {
        if let Some(binding) = element {
            return Some(BindingSlot {
                holes,
                binding: Some(binding),
            });
        }
        holes += 1;
    }
    (holes != 0).then_some(BindingSlot {
        holes,
        binding: None,
    })
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_parameter_binding(
        &mut self,
        pattern: FormalParameterPattern,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let pattern = self.parameter_binding(pattern);
        self.format_declaration_binding(pattern, mode, writer)
    }

    pub(super) fn parameter_binding(&self, pattern: FormalParameterPattern) -> BindingPattern {
        match self.ast.formal_parameter_pattern_data(pattern) {
            FormalParameterPatternData::BindingIdentifier(node) => {
                BindingPattern::BindingIdentifier(node)
            }
            FormalParameterPatternData::ArrayPattern(node) => BindingPattern::ArrayPattern(node),
            FormalParameterPatternData::ObjectPattern(node) => BindingPattern::ObjectPattern(node),
        }
    }

    pub(super) fn format_declaration_binding(
        &mut self,
        pattern: BindingPattern,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(pattern.span(self.ast), writer)?;
            return Ok(None);
        }
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::BindingIdentifier(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                Ok(None)
            }
            BindingPatternData::ArrayPattern(array) => {
                self.format_array_binding(array, writer).map(|()| None)
            }
            BindingPatternData::ObjectPattern(object) => {
                self.format_object_binding(object, writer).map(|()| None)
            }
            BindingPatternData::AssignmentPattern(default) => {
                let default = self.prepare_binding_default(
                    default.left(self.ast),
                    default.right(self.ast),
                    default.span(self.ast),
                )?;
                self.format_binding_default(default, writer)
            }
            BindingPatternData::BindingRestElement(rest) => {
                writer.text("...");
                let argument = rest.argument(self.ast);
                let leading = self.comments.between(Span::new(
                    rest.span(self.ast).start + 3,
                    argument.span(self.ast).start,
                ))?;
                let mode = self.content_mode(leading);
                self.comments.claim(leading)?;
                self.format_comment_run(
                    leading,
                    CommentRunLayout::FlatLeading(argument.span(self.ast).start),
                    writer,
                )?;
                self.format_declaration_binding(argument, mode, writer)
            }
            BindingPatternData::SimpleAssignmentTarget(target) => self
                .format_simple_assignment_target(
                    target,
                    ExpressionContext::at(ExpressionPosition::Nested),
                    writer,
                ),
        }
    }

    pub(super) fn prepare_binding_default(
        &self,
        left: BindingPattern,
        right: Expr,
        span: Span,
    ) -> Result<PreparedBindingDefault<'ast>, FormatError> {
        let left_end = left.span(self.ast).end;
        let right_start = right.span(self.ast).start;
        let gap = Span::new(left_end, right_start);
        let comments = self.comments.in_range(gap);
        let Some(operator) = self.find_token(gap, comments, "=") else {
            return Err(self.unsupported("binding default boundary", right));
        };
        let mut left_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        let mut whole = false;
        if self.has_suppression {
            let split = comments.partition_point(|comment| comment.span.end <= operator);
            let (before, after) = comments.split_at(split);
            for position in self.suppression_positions(before, operator) {
                match position {
                    SuppressionPosition::OwnLine => whole = true,
                    SuppressionPosition::Inline | SuppressionPosition::EndOfLine => {
                        left_mode = ContentMode::Verbatim;
                    }
                }
            }
            for position in self.suppression_positions(after, right_start) {
                match position {
                    SuppressionPosition::OwnLine => whole = true,
                    SuppressionPosition::EndOfLine => left_mode = ContentMode::Verbatim,
                    SuppressionPosition::Inline => right_mode = ContentMode::Verbatim,
                }
            }
        }
        let owner = if whole {
            DefaultOwner::Whole
        } else if left_mode == ContentMode::Verbatim || right_mode == ContentMode::Verbatim {
            DefaultOwner::Bound {
                gap,
                left_mode,
                right_mode,
            }
        } else {
            let content = self.expression_content(right);
            let boundary =
                self.assignment_boundary(left_end, &content, Span::new(operator, operator + 1))?;
            let break_after = boundary.own_line
                || !boundary.operator_line.is_empty()
                || self.assignment_right_breaks_after_operator(content.data);
            DefaultOwner::Formatted {
                boundary,
                break_after,
            }
        };
        Ok(PreparedBindingDefault {
            left,
            right,
            span,
            owner,
        })
    }

    pub(super) fn format_binding_default(
        &mut self,
        default: PreparedBindingDefault<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let PreparedBindingDefault {
            left,
            right,
            span,
            owner,
        } = default;
        let (boundary, break_after) = match owner {
            DefaultOwner::Whole => {
                self.format_verbatim(span, writer)?;
                return Ok(None);
            }
            DefaultOwner::Bound {
                gap,
                left_mode,
                right_mode,
            } => {
                return self.format_initializer(
                    PreparedInitializer {
                        left: InitializerLeft::Binding(left),
                        right,
                        layout: InitializerLayout::Bound { gap, right_mode },
                        left_mode,
                        context: ExpressionContext::binding_initializer(),
                    },
                    writer,
                );
            }
            DefaultOwner::Formatted {
                boundary,
                break_after,
            } => (boundary, break_after),
        };
        self.format_initializer(
            PreparedInitializer {
                left: InitializerLeft::Binding(left),
                right,
                layout: InitializerLayout::Formatted {
                    boundary,
                    break_after,
                },
                left_mode: ContentMode::Formatted,
                context: ExpressionContext::binding_initializer(),
            },
            writer,
        )
    }

    pub(super) fn format_initializer(
        &mut self,
        initializer: PreparedInitializer<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let PreparedInitializer {
            left,
            right,
            layout,
            left_mode,
            context,
        } = initializer;
        let left_end = match left {
            InitializerLeft::Enum { name, computed } => {
                self.format_enum_member_key(name, computed, left_mode, writer)?;
                computed.map_or(name.span(self.ast).end, |span| span.end)
            }
            InitializerLeft::Binding(binding) => {
                let tail = self.format_declaration_binding(binding, left_mode, writer)?;
                self.format_expression_tail(tail, writer)?;
                binding.span(self.ast).end
            }
            InitializerLeft::Property { key, computed } => {
                self.format_field_key(key, computed, left_mode, writer)?;
                computed.map_or(key.span(self.ast).end, |span| span.end)
            }
        };
        self.format_initializer_right(left_end, right, layout, context, writer)
    }

    pub(super) fn format_initializer_right(
        &mut self,
        left_end: u32,
        right: Expr,
        layout: InitializerLayout<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let (boundary, break_after) = match layout {
            InitializerLayout::Bound { gap, right_mode } => {
                self.format_verbatim(gap, writer)?;
                return self.format_expression(
                    right,
                    context.with_content_mode(right_mode),
                    writer,
                );
            }
            InitializerLayout::Formatted {
                boundary,
                break_after,
            } => (boundary, break_after),
        };
        self.comments.claim(boundary.before)?;
        self.format_comment_run(
            boundary.before,
            CommentRunLayout::SourceAfter(left_end),
            writer,
        )?;
        boundary
            .before_operator
            .write(writer, FlatSeparator::Space, Writer::space);
        writer.text("=");
        self.comments.claim(boundary.operator_line)?;
        self.format_comment_run(
            boundary.operator_line,
            CommentRunLayout::SourceAfter(boundary.operator_span.end),
            writer,
        )?;
        writer.group(false, |writer| {
            let mut write_right = |writer: &mut Writer<'ast>| {
                boundary
                    .after_operator
                    .write(writer, FlatSeparator::Space, |writer| {
                        if boundary.own_line || !boundary.operator_line.is_empty() {
                            writer.hard_line();
                        } else if break_after {
                            writer.line();
                        } else {
                            writer.space();
                        }
                    });
                self.comments.claim(boundary.moved)?;
                self.format_comment_run(boundary.moved, CommentRunLayout::Source, writer)?;
                if let Some(last) = boundary.moved.last() {
                    let next = boundary
                        .leading
                        .first()
                        .map_or(boundary.layout_start, |comment| comment.span.start);
                    let breaks = self
                        .line_breaks(last.span.end, boundary.operator_span.start)
                        .max(self.line_breaks(boundary.operator_span.end, next));
                    match breaks {
                        0 => writer.space(),
                        1 => writer.hard_line(),
                        _ => {
                            writer.hard_line();
                            writer.hard_line();
                        }
                    }
                }
                self.comments.claim(boundary.leading)?;
                self.format_expression_with_leading_comments(
                    right,
                    boundary.right_head(),
                    context.with_content_mode(boundary.mode),
                    writer,
                )
            };
            if break_after {
                writer.indent(write_right)
            } else {
                write_right(writer)
            }
        })
    }

    fn format_array_binding(
        &mut self,
        array: ArrayPattern,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = array.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        let mut elements = array
            .elements(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .chain(
                array
                    .rest(ast)
                    .map(|rest| Some(BindingPattern::BindingRestElement(rest))),
            );
        let mut slots = std::iter::from_fn(|| next_slot(&mut elements)).peekable();
        if slots.peek().is_none()
            && self.content_mode(self.comments.in_range(content)) == ContentMode::Verbatim
        {
            return self.format_verbatim(span, writer);
        }
        let has_content = slots.peek().is_some() || !self.comments.in_range(content).is_empty();
        writer.group(false, |writer| {
            writer.text("[");
            let end = writer.indent(|writer| {
                if has_content {
                    self.opening_line_constraint(content).write(
                        writer,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                    );
                }
                let mut end = content.start;
                let mut separator = Separator::Line;
                let mut has_slot = false;
                let mut following_mode = None;
                let mut elisions = ElisionLayout::Formatted;
                let constrained = self.diagnostic_range_is_sensitive(content);
                while let Some(slot) = slots.next() {
                    let mode =
                        slot.binding.map_or(ContentMode::Formatted, |binding| {
                            following_mode.unwrap_or_else(|| {
                                if self.has_suppression {
                                    self.content_mode(self.comments.in_range(Span::new(
                                        content.start,
                                        binding.span(ast).start,
                                    )))
                                } else {
                                    ContentMode::Formatted
                                }
                            })
                        });
                    let source_elisions = match elisions {
                        ElisionLayout::Formatted => 0,
                        ElisionLayout::Source(holes) => {
                            debug_assert_eq!(
                                holes, slot.holes,
                                "a source elision boundary covers the prepared AST slots"
                            );
                            holes
                        }
                    };
                    for _ in source_elisions..slot.holes {
                        if constrained || self.has_suppression {
                            let next = slot
                                .binding
                                .map_or(content.end, |node| node.span(ast).start);
                            let gap = Span::new(end, next);
                            let comments = self.comments.between(gap)?;
                            if constrained || !comments.is_empty() {
                                let Some(comma) = self.find_comma(gap, comments) else {
                                    return Err(self.unsupported("binding elision boundary", array));
                                };
                                let count =
                                    comments.partition_point(|comment| comment.span.end <= comma);
                                let leading = self.comments.take(count);
                                match (leading.first(), has_slot) {
                                    (Some(first), _) => {
                                        if has_slot {
                                            self.comment_gap(end, first.span.start, writer);
                                        }
                                        self.format_comment_run(
                                            leading,
                                            CommentRunLayout::Leading(comma),
                                            writer,
                                        )?;
                                    }
                                    (None, true) => ArgumentLayout::Lines.separator(
                                        separator,
                                        self.line_constraint(end, comma),
                                        writer,
                                    ),
                                    (None, false) => self.line_constraint(end, comma).write(
                                        writer,
                                        FlatSeparator::Empty,
                                        |_| {},
                                    ),
                                }
                                writer.text(",");
                                end = comma + 1;
                                separator = Separator::Line;
                                has_slot = true;
                                continue;
                            }
                        }
                        if has_slot {
                            ArgumentLayout::Lines.separator(
                                separator,
                                LineConstraint::Unconstrained,
                                writer,
                            );
                            separator = Separator::Line;
                        }
                        writer.text(",");
                        has_slot = true;
                    }
                    let Some(binding) = slot.binding else {
                        continue;
                    };
                    if has_slot {
                        let next = self
                            .comments
                            .between(Span::new(end, binding.span(ast).start))?
                            .first()
                            .map_or(binding.span(ast).start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.line_constraint(end, next),
                            writer,
                        );
                    }
                    let following = match slots.peek() {
                        Some(slot) if slot.holes != 0 => FollowingItem::Elision {
                            node: slot.binding.map(|node| node.span(ast)),
                            holes: slot.holes,
                        },
                        Some(slot) => slot.binding.map_or(FollowingItem::End, |node| {
                            FollowingItem::Node(node.span(ast))
                        }),
                        None => FollowingItem::End,
                    };
                    let rest = matches!(
                        ast.binding_pattern_data(binding),
                        BindingPatternData::BindingRestElement(_)
                    );
                    let mut item = self.prepare_delimited_item(
                        binding.span(ast),
                        following,
                        Span::new(end, content.end),
                        !rest && self.options.trailing_commas != TrailingCommas::None,
                        writer,
                    )?;
                    item.set_leading_mode(mode);
                    let tail = self.format_declaration_binding(binding, item.mode, writer)?;
                    let boundary = self.finish_delimited_item(item, tail, writer)?;
                    end = boundary.end;
                    separator = boundary.separator;
                    following_mode = Some(boundary.following_mode);
                    elisions = boundary.elisions;
                    has_slot = true;
                }
                let dangling = self.comments.between(Span::new(end, content.end))?;
                self.comments.claim(dangling)?;
                if dangling
                    .iter()
                    .any(|comment| comment.kind == CommentKind::Line)
                {
                    writer.break_parent();
                }
                self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
                Ok::<_, FormatError>(dangling.last().map_or(end, |comment| comment.span.end))
            })?;
            if has_content {
                self.line_constraint(end, content.end).write(
                    writer,
                    FlatSeparator::Empty,
                    Writer::soft_line,
                );
            }
            writer.text("]");
            Ok(())
        })
    }

    fn format_object_binding(
        &mut self,
        object: ObjectPattern,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = object.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        let mut properties = object
            .properties(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .peekable();
        let rest = object.rest(ast);
        if properties.peek().is_none()
            && rest.is_none()
            && self.content_mode(self.comments.in_range(content)) == ContentMode::Verbatim
        {
            return self.format_verbatim(span, writer);
        }
        writer.group(false, |writer| {
            writer.text("{");
            let has_items = properties.peek().is_some() || rest.is_some();
            let has_comments = !self.comments.in_range(content).is_empty();
            let end = writer.indent(|writer| {
                if has_items || has_comments {
                    self.binding_brace_line(content, writer);
                }
                let mut end = content.start;
                let mut separator = Separator::Line;
                let mut has_previous = false;
                let mut following_mode = None;
                while let Some(property) = properties.next() {
                    if has_previous {
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.line_constraint(end, property.span(ast).start),
                            writer,
                        );
                    }
                    let following = properties
                        .peek()
                        .map(|next| next.span(ast))
                        .or_else(|| rest.map(|rest| rest.span(ast)))
                        .map_or(FollowingItem::End, FollowingItem::Node);
                    let mut item = self.prepare_delimited_item(
                        property.span(ast),
                        following,
                        Span::new(end, content.end),
                        self.options.trailing_commas != TrailingCommas::None,
                        writer,
                    )?;
                    if let Some(mode) = following_mode {
                        item.set_leading_mode(mode);
                    }
                    let tail = if item.mode == ContentMode::Verbatim {
                        self.format_verbatim(property.span(ast), writer)?;
                        None
                    } else {
                        self.format_binding_property(property, writer)?
                    };
                    let boundary = self.finish_delimited_item(item, tail, writer)?;
                    end = boundary.end;
                    separator = boundary.separator;
                    following_mode = Some(boundary.following_mode);
                    has_previous = true;
                }
                if let Some(rest) = rest {
                    if has_previous {
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.line_constraint(end, rest.span(ast).start),
                            writer,
                        );
                    }
                    let mut item = self.prepare_delimited_item(
                        rest.span(ast),
                        FollowingItem::End,
                        Span::new(end, content.end),
                        false,
                        writer,
                    )?;
                    if let Some(mode) = following_mode {
                        item.set_leading_mode(mode);
                    }
                    let tail = self.format_declaration_binding(
                        BindingPattern::BindingRestElement(rest),
                        item.mode,
                        writer,
                    )?;
                    end = self.finish_delimited_item(item, tail, writer)?.end;
                }
                let dangling = self.comments.between(Span::new(end, content.end))?;
                self.comments.claim(dangling)?;
                if dangling
                    .iter()
                    .any(|comment| comment.kind == CommentKind::Line)
                {
                    writer.break_parent();
                }
                self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
                Ok::<_, FormatError>(dangling.last().map_or(end, |comment| comment.span.end))
            })?;
            if has_items || has_comments {
                self.line_constraint(end, content.end).write(
                    writer,
                    FlatSeparator::Space,
                    |writer| {
                        if self.options.bracket_spacing {
                            writer.line();
                        } else {
                            writer.soft_line();
                        }
                    },
                );
            }
            writer.text("}");
            Ok(())
        })
    }

    fn binding_brace_line(&self, content: Span, writer: &mut Writer<'ast>) {
        self.opening_line_constraint(content)
            .write(writer, FlatSeparator::Space, |writer| {
                if self.options.bracket_spacing {
                    writer.line();
                } else {
                    writer.soft_line();
                }
            });
    }

    fn format_binding_property(
        &mut self,
        property: BindingProperty,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let value = property.value(ast);
        if property.shorthand(ast) {
            return self.format_declaration_binding(value, ContentMode::Formatted, writer);
        }
        let key = property.key(ast);
        let value_span = value.span(ast);
        let key_end = if property.computed(ast) {
            let gap = Span::new(key.span(ast).end, value_span.start);
            self.find_token(gap, self.comments.in_range(gap), "]")
                .map(|end| end + 1)
                .ok_or_else(|| self.unsupported("binding computed key boundary", property))?
        } else {
            key.span(ast).end
        };
        let gap = Span::new(key_end, value_span.start);
        let comments = self.comments.in_range(gap);
        let Some(colon) = self.find_token(gap, comments, ":") else {
            return Err(self.unsupported("binding property colon", property));
        };
        let (before, after) =
            comments.split_at(comments.partition_point(|comment| comment.span.end <= colon));
        let key_trailing = if self.has_suppression && property.computed(ast) {
            self.comments.in_range(Span::new(key.span(ast).end, colon))
        } else {
            before
        };
        let key_leading = (self.has_suppression && property.computed(ast))
            .then(|| Span::new(property.span(ast).start + 1, key.span(ast).start));
        let suppression = self.property_suppression(
            key_leading,
            key_trailing,
            after,
            Span::new(colon, colon + 1),
            value_span.start,
        );
        if suppression.enclosing == ContentMode::Verbatim {
            self.format_verbatim(property.span(ast), writer)?;
            return Ok(None);
        }
        let mode = suppression.value;
        let next = after
            .first()
            .map_or(value_span.start, |comment| comment.span.start);
        let value_prefix = if mode == ContentMode::Verbatim {
            self.source_line_constraint(colon + 1, next)
        } else {
            self.line_constraint(colon + 1, next)
        };
        let computed = property
            .computed(ast)
            .then_some(Span::new(property.span(ast).start, key_end));
        self.format_property_key(key, computed, suppression.key, writer)?;
        self.comments.claim(before)?;
        self.format_comment_run(before, CommentRunLayout::SourceAfter(key_end), writer)?;
        let end = before.last().map_or(key_end, |comment| comment.span.end);
        if before
            .last()
            .is_some_and(|comment| comment.kind == CommentKind::Line)
        {
            writer.hard_line();
        } else {
            self.line_constraint(end, colon)
                .write(writer, FlatSeparator::Empty, |_| {});
        }
        writer.text(":");
        value_prefix.write(writer, FlatSeparator::Space, Writer::space);
        self.comments.claim(after)?;
        self.format_comment_run(
            after,
            CommentRunLayout::FlatLeading(value_span.start),
            writer,
        )?;
        self.format_declaration_binding(value, mode, writer)
    }
}
