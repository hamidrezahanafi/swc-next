//! Shared fluent layout over source-owned member and call operations.

use std::iter::{Peekable, Take};
use std::vec::IntoIter;

use swc_next_ecma_ast::{Expr, ExprData, GetSpan, IdentifierName, ParenthesizedExpression};

use super::arguments::{ArgumentGroup, SourceArguments};
use super::members::{
    MemberBoundary, MemberLookupLayout, MemberLookupOwner, PreparedMember, SourceMember,
};
use super::{
    AccessKind, ExpressionContext, ExpressionLayout, ExpressionPosition, Formatter,
    MemberChainLayout, MemberLayout,
};
use crate::formatter::comments::{CommentGap, ExpressionTail};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{
    FormatError,
    document::{Fragment, NeutralFragment, RecordedGroup, Writer},
};

pub(super) enum FluentChainEligibility {
    Fluent,
    MemberOnly,
    Unqualified,
}

pub(super) fn atomic_chain_head(data: ExprData) -> bool {
    matches!(
        data,
        ExprData::IdentifierReference(_)
            | ExprData::ThisExpression(_)
            | ExprData::MetaProperty(_)
            | ExprData::StringLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::RegExpLiteral(_)
    )
}

struct ChainMember {
    source: SourceMember,
    lookup_owner: MemberLookupOwner,
    numeric: bool,
}

enum ChainPart<'ast> {
    Member(ChainMember),
    Call(SourceArguments<'ast>),
}

struct PreparedChain<'ast> {
    head: Expr,
    head_context: ExpressionContext,
    cast_head: Option<ParenthesizedExpression>,
    parts: Vec<ChainPart<'ast>>,
    // The prepared operation boundary between the callee and terminal lookups.
    callee_parts: usize,
    calls: usize,
    // Source boundaries can add groups and independently require expansion.
    groups: usize,
}

enum ChainSuffix {
    Collecting { members: usize },
    Complete { members: usize },
}

enum ChainLookupLayout {
    Local,
    InlineContinuation,
    ExpressionContinuation,
}

struct ChainGrouping {
    groups: usize,
    pending_call: bool,
}

impl ChainGrouping {
    fn new() -> Self {
        Self {
            groups: 1,
            pending_call: false,
        }
    }

    fn call(&mut self) {
        self.pending_call = true;
    }

    fn member(&mut self, numeric: bool) {
        // The reverse walk closes the same call-bearing groups as the forward
        // printer. Numeric accesses stay with a call; terminal lookup groups
        // are owned separately. The remaining prefix is head.
        if !numeric {
            if self.pending_call {
                self.groups += 1;
            }
            self.pending_call = false;
        }
    }
}

struct ChainHead {
    expression: Expr,
    context: ExpressionContext,
    has_comments: bool,
    attach: bool,
    layout: ChainHeadLayout,
}

enum ChainHeadLayout {
    NeutralAtom,
    Grouped,
}

struct PendingMember<'ast> {
    member: ChainMember,
    boundary: MemberBoundary<'ast>,
}

impl PendingMember<'_> {
    fn separator(&self) -> ChainSeparator {
        ChainSeparator {
            gap: self.boundary.gap(),
            empty_line: self.boundary.empty_line,
        }
    }

    fn source_break(&self) -> bool {
        self.boundary.has_comments
            || self.boundary.empty_line
            || self.boundary.gap().requires_line()
    }
}

enum ChainOperation<'ast> {
    Member(PendingMember<'ast>),
    Call(SourceArguments<'ast>),
}

enum ChainArgumentLayout {
    ByWidth,
    Inspect,
    Complex,
}

struct ChainSource<'ast, 'parts> {
    parts: Peekable<Take<&'parts mut IntoIter<ChainPart<'ast>>>>,
    pending: Option<PendingMember<'ast>>,
    tail: Option<ExpressionTail<'ast>>,
    argument_layout: ChainArgumentLayout,
}

#[derive(Clone, Copy)]
struct ChainSeparator {
    gap: CommentGap,
    empty_line: bool,
}

impl ChainSeparator {
    fn write(self, expanded: bool, writer: &mut Writer<'_>) {
        match self.gap {
            CommentGap::LineComment => {
                writer.hard_line();
                if self.empty_line {
                    writer.hard_line();
                }
            }
            CommentGap::Source(constraint) => {
                constraint.write(writer, FlatSeparator::Empty, |writer| {
                    if expanded || self.empty_line {
                        writer.hard_line();
                        if self.empty_line {
                            writer.hard_line();
                        }
                    }
                });
            }
        }
    }
}

#[derive(Clone, Copy)]
struct RecordedBoundary<'ast> {
    trailing: Option<Fragment<'ast>>,
    separator: ChainSeparator,
}

#[derive(Default)]
struct GroupFacts {
    has_function: bool,
    has_boundary_comments: bool,
    source_break: bool,
    preferred_breaks: bool,
    has_parent_call_group: bool,
}

#[derive(Clone, Copy)]
enum ChainGroupDoc<'ast> {
    Recorded(Fragment<'ast>),
    ParentCall(Fragment<'ast>),
    Neutral(NeutralFragment<'ast>),
}

impl<'ast> ChainGroupDoc<'ast> {
    fn for_group(group: RecordedGroup<'ast>, parent_call: bool) -> Self {
        if parent_call {
            Self::ParentCall(group.contents)
        } else {
            Self::for_tail(group.grouped)
        }
    }

    fn for_tail(fragment: Fragment<'ast>) -> Self {
        match fragment.try_neutral() {
            Some(fragment) => Self::Neutral(fragment),
            None => Self::Recorded(fragment),
        }
    }

    fn can_flat(self) -> bool {
        match self {
            Self::Recorded(fragment) | Self::ParentCall(fragment) => fragment.try_flat().is_some(),
            Self::Neutral(_) => true,
        }
    }
}

struct ChainGroup<'ast> {
    doc: ChainGroupDoc<'ast>,
    before: Option<RecordedBoundary<'ast>>,
    can_flat: bool,
    facts: GroupFacts,
}

impl<'ast> ChainGroup<'ast> {
    fn new(
        doc: ChainGroupDoc<'ast>,
        before: Option<RecordedBoundary<'ast>>,
        facts: GroupFacts,
    ) -> Self {
        Self {
            doc,
            before,
            can_flat: doc.can_flat(),
            facts,
        }
    }
}

struct ChainGroups<'ast> {
    head: ChainGroup<'ast>,
    tail: Vec<ChainGroup<'ast>>,
}

impl ChainGroups<'_> {
    fn requires_expanded(&self) -> bool {
        if self.head.facts.source_break || self.tail.iter().any(|group| group.facts.source_break) {
            return true;
        }
        let Some((last, earlier)) = self.tail.split_last() else {
            return false;
        };
        self.head.facts.preferred_breaks
            || !self.head.can_flat
            || earlier
                .iter()
                .any(|group| group.facts.preferred_breaks || !group.can_flat)
            || (last.facts.preferred_breaks || !last.can_flat)
                && (self.head.facts.has_function
                    || earlier.iter().any(|group| group.facts.has_function))
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    /// Qualification retains a no-call proof only for bare member receivers.
    /// Source wrappers and grammar or diagnostic barriers keep their own rules.
    pub(super) fn fluent_chain_eligibility(
        &self,
        data: ExprData,
        context: ExpressionContext,
        layout: ExpressionLayout,
    ) -> FluentChainEligibility {
        if context.member_chain_layout == MemberChainLayout::Local
            || context.content_mode != ContentMode::Formatted
            || context.member_layout == MemberLayout::Tight
            || matches!(
                context.position,
                ExpressionPosition::NewCallee | ExpressionPosition::MemberAssignmentObject
            )
            || layout == ExpressionLayout::Flat
        {
            return FluentChainEligibility::Unqualified;
        }
        let (mut current, required_calls, mut calls, mut member) = match data {
            ExprData::CallExpression(call) => (call.callee(self.ast), 2, 1, false),
            ExprData::MemberExpression(access) => (access.object(self.ast), 1, 0, true),
            _ => return FluentChainEligibility::Unqualified,
        };
        let mut bare_members = member;
        loop {
            let data = self.ast.expr_data(current);
            current = match data {
                ExprData::CallExpression(call) => {
                    calls += 1;
                    bare_members = false;
                    call.callee(self.ast)
                }
                ExprData::MemberExpression(access) => {
                    member = true;
                    access.object(self.ast)
                }
                ExprData::ParenthesizedExpression(parenthesized) => {
                    bare_members = false;
                    parenthesized.expression(self.ast)
                }
                ExprData::ChainExpression(chain) => {
                    bare_members = false;
                    chain.expression(self.ast)
                }
                _ => {
                    return if bare_members && atomic_chain_head(data) {
                        FluentChainEligibility::MemberOnly
                    } else {
                        FluentChainEligibility::Unqualified
                    };
                }
            };
            if member && calls >= required_calls {
                return FluentChainEligibility::Fluent;
            }
        }
    }

    fn prepare_fluent_chain(
        &self,
        expression: Expr,
        context: ExpressionContext,
    ) -> Result<PreparedChain<'ast>, FormatError> {
        let mut current = expression;
        let mut current_context = context;
        let mut parts = Vec::new();
        let mut calls = 0;
        let terminal_member = matches!(
            self.ast.expr_data(expression),
            ExprData::MemberExpression(_)
        );
        let mut suffix = if terminal_member {
            ChainSuffix::Collecting { members: 0 }
        } else {
            ChainSuffix::Complete { members: 0 }
        };
        let mut grouping = ChainGrouping::new();
        while current_context.content_mode == ContentMode::Formatted {
            match self.ast.expr_data(current) {
                ExprData::MemberExpression(member) => match self.source_member(member)? {
                    PreparedMember::Formatted(source) => {
                        let lookup_owner = MemberLookupOwner::from_context(current_context);
                        let numeric = source.key().is_some_and(|key| {
                            matches!(self.ast.expr_data(key), ExprData::NumericLiteral(_))
                        });
                        match &mut suffix {
                            ChainSuffix::Collecting { members } => {
                                // The last non-numeric lookup encountered in reverse
                                // marks the suffix's beginning. Earlier numeric
                                // accesses stay with the final call's receiver.
                                if !numeric {
                                    *members = parts.len() + 1;
                                }
                            }
                            ChainSuffix::Complete { .. } => grouping.member(numeric),
                        }
                        current = source.object;
                        current_context = source.receiver_context(current_context);
                        parts.push(ChainPart::Member(ChainMember {
                            source,
                            lookup_owner,
                            numeric,
                        }));
                    }
                    PreparedMember::Verbatim(member) => {
                        current = Expr::MemberExpression(member);
                        current_context = current_context.with_content_mode(ContentMode::Verbatim);
                        break;
                    }
                },
                ExprData::CallExpression(call) => {
                    let arguments = self.source_call(call, current_context)?;
                    current = arguments.callee;
                    current_context = arguments.callee_context(current_context);
                    if let ChainSuffix::Collecting { members } = suffix {
                        suffix = ChainSuffix::Complete { members };
                    }
                    parts.push(ChainPart::Call(arguments));
                    calls += 1;
                    grouping.call();
                }
                ExprData::ParenthesizedExpression(parenthesized) => {
                    let Some(child) =
                        self.chain_parentheses_operand(parenthesized, current_context)
                    else {
                        break;
                    };
                    current = child;
                }
                ExprData::ChainExpression(chain)
                    if matches!(
                        current_context.position,
                        ExpressionPosition::MemberObject(AccessKind::Optional)
                            | ExpressionPosition::CallCallee {
                                access: AccessKind::Optional,
                                ..
                            }
                    ) =>
                {
                    current = chain.expression(self.ast);
                }
                _ => break,
            }
        }
        let continuation_parts = match suffix {
            ChainSuffix::Collecting { members } | ChainSuffix::Complete { members } => members,
        };
        let callee_parts = parts.len() - continuation_parts;
        parts.reverse();
        Ok(PreparedChain {
            head: current,
            head_context: current_context,
            cast_head: self.source_cast_head(current),
            parts,
            callee_parts,
            calls,
            groups: grouping.groups,
        })
    }

    pub(super) fn format_fluent_chain(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let chain = self.prepare_fluent_chain(expression, context)?;
        let terminal_member = matches!(
            self.ast.expr_data(expression),
            ExprData::MemberExpression(_)
        );
        // The local path consumes the prepared sources without any choice Docs.
        // Cast-owned heads keep their actual protected source parentheses.
        if chain.cast_head.is_some() || terminal_member && chain.calls <= 1 {
            return self.format_local_chain(chain, writer);
        }
        let head_comments = !self.comments.in_range(chain.head.span(self.ast)).is_empty();
        let head_data = self.ast.expr_data(chain.head);
        // Required source lines must retain the group's fill-probe barrier.
        // This tape prefix leaves a Text-only atom with no head operations.
        let head_layout = if chain.head_context.content_mode == ContentMode::Formatted
            && !head_comments
            && matches!(
                head_data,
                ExprData::IdentifierReference(_)
                    | ExprData::ThisExpression(_)
                    | ExprData::BooleanLiteral(_)
                    | ExprData::NullLiteral(_)
                    | ExprData::NumericLiteral(_)
                    | ExprData::BigIntLiteral(_)
                    | ExprData::RegExpLiteral(_)
            )
            && matches!(
                chain.parts.as_slice(),
                [ChainPart::Member(member), remaining @ ..]
                    if !member.numeric
                        && !matches!(remaining.first(), Some(ChainPart::Member(_)))
            ) {
            ChainHeadLayout::NeutralAtom
        } else {
            ChainHeadLayout::Grouped
        };
        let head_attach = match head_data {
            ExprData::ThisExpression(_) => true,
            ExprData::IdentifierReference(identifier) => {
                let name = self.ast.get_utf8(identifier.name(self.ast));
                factory_name(name)
                    || matches!(context.position, ExpressionPosition::Statement)
                        && name.len() <= usize::from(self.options.indent_width.value())
            }
            _ => false,
        };
        let mut parts = chain.parts.into_iter();
        let tail = {
            let mut source = ChainSource {
                parts: parts.by_ref().take(chain.callee_parts).peekable(),
                pending: None,
                tail: None,
                argument_layout: if chain.groups > 2 {
                    ChainArgumentLayout::Inspect
                } else {
                    ChainArgumentLayout::ByWidth
                },
            };
            let (head, merge_head) = self.record_chain_head(
                ChainHead {
                    expression: chain.head,
                    context: chain.head_context,
                    has_comments: head_comments,
                    attach: head_attach,
                    layout: head_layout,
                },
                &mut source,
                writer,
            )?;
            // Merging the head leaves at most two groups in a short chain. Source
            // boundaries may split those groups, but already require expansion.
            if merge_head && chain.groups <= 3 {
                source.argument_layout = ChainArgumentLayout::ByWidth;
            }
            let mut tail = Vec::new();
            while source.pending.is_some() || source.parts.peek().is_some() {
                tail.push(self.record_chain_group(&mut source, writer)?);
            }
            let groups = ChainGroups { head, tail };
            let merge_head = merge_head
                && !groups.head.facts.has_boundary_comments
                && groups.tail.first().is_some_and(|group| {
                    !group.facts.has_boundary_comments && !group.facts.source_break
                });
            let short_chain = groups.tail.len() <= if merge_head { 2 } else { 1 };
            // A short chain has one shared layout even when an argument's own
            // collection group breaks. Its new source shape must not change the
            // chain's strategy on the next pass.
            if short_chain
                && !groups.head.facts.source_break
                && !groups.tail.iter().any(|group| group.facts.source_break)
            {
                writer.group(false, |writer| write_compact(&groups, writer));
            } else if matches!(source.argument_layout, ChainArgumentLayout::Complex)
                || groups.requires_expanded()
            {
                write_expanded(&groups, merge_head, writer);
            } else {
                let (expanded, ()) = writer.record_with_value(|writer| {
                    write_expanded(&groups, merge_head, writer);
                    Ok::<_, FormatError>(())
                })?;
                let (compact, ()) = writer.record_with_value(|writer| {
                    write_compact(&groups, writer);
                    Ok::<_, FormatError>(())
                })?;
                writer.write_alternatives(compact, &[compact], expanded);
            }
            source.tail
        };
        let continuation_layout =
            if matches!(context.position, ExpressionPosition::AssignmentRight(_)) {
                ChainLookupLayout::InlineContinuation
            } else {
                ChainLookupLayout::ExpressionContinuation
            };
        self.format_chain_operations(parts, tail, continuation_layout, writer)
    }

    fn format_local_chain(
        &mut self,
        chain: PreparedChain<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let tail = self.format_expression(chain.head, chain.head_context, writer)?;
        self.format_chain_operations(
            chain.parts.into_iter(),
            tail,
            ChainLookupLayout::Local,
            writer,
        )
    }

    fn format_chain_operations(
        &mut self,
        parts: IntoIter<ChainPart<'ast>>,
        mut tail: Option<ExpressionTail<'ast>>,
        layout: ChainLookupLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        for part in parts {
            match part {
                ChainPart::Member(member) => {
                    let boundary = self.prepare_member_boundary(&member.source, tail.take())?;
                    let lookup = match layout {
                        ChainLookupLayout::InlineContinuation
                            if boundary.empty_line || boundary.gap().requires_line() =>
                        {
                            MemberLookupLayout::Grouped
                        }
                        ChainLookupLayout::InlineContinuation => MemberLookupLayout::Inline,
                        ChainLookupLayout::Local | ChainLookupLayout::ExpressionContinuation => {
                            self.member_lookup_layout(&member.source, member.lookup_owner)
                        }
                    };
                    let boundary_inside = !matches!(layout, ChainLookupLayout::Local)
                        && (boundary.has_before_comments() || boundary.empty_line);
                    if boundary_inside {
                        lookup.write_contents(writer, |writer| {
                            self.format_member_trailing(&boundary, writer)?;
                            if boundary.empty_line {
                                ChainSeparator {
                                    gap: boundary.gap(),
                                    empty_line: true,
                                }
                                .write(true, writer);
                            } else {
                                boundary
                                    .gap()
                                    .write(writer, FlatSeparator::Empty, |writer| {
                                        if matches!(lookup, MemberLookupLayout::Grouped) {
                                            writer.soft_line();
                                        }
                                    });
                            }
                            self.format_member_access(&member.source, boundary, writer)
                        })?;
                    } else {
                        self.format_member_trailing(&boundary, writer)?;
                        lookup.write(boundary.gap(), writer, |writer| {
                            self.format_member_access(&member.source, boundary, writer)
                        })?;
                    }
                }
                ChainPart::Call(arguments) => {
                    tail = self.format_source_arguments(arguments, tail.take(), writer)?;
                }
            }
        }
        Ok(tail)
    }

    fn next_chain_operation(
        &self,
        source: &mut ChainSource<'ast, '_>,
    ) -> Result<Option<ChainOperation<'ast>>, FormatError> {
        if let Some(member) = source.pending.take() {
            return Ok(Some(ChainOperation::Member(member)));
        }
        match source.parts.next() {
            Some(ChainPart::Member(member)) => {
                let boundary = self.prepare_member_boundary(&member.source, source.tail.take())?;
                Ok(Some(ChainOperation::Member(PendingMember {
                    member,
                    boundary,
                })))
            }
            Some(ChainPart::Call(arguments)) => Ok(Some(ChainOperation::Call(arguments))),
            None => Ok(None),
        }
    }

    fn record_chain_head(
        &mut self,
        head: ChainHead,
        source: &mut ChainSource<'ast, '_>,
        writer: &mut Writer<'ast>,
    ) -> Result<(ChainGroup<'ast>, bool), FormatError> {
        let mut facts = GroupFacts {
            has_boundary_comments: head.has_comments,
            ..GroupFacts::default()
        };
        let mut leading = true;
        let mut head_parts = 0;
        let mut last_name: Option<IdentifierName> = None;
        let write_head = |writer: &mut Writer<'ast>| {
            source.tail = self.format_expression(head.expression, head.context, writer)?;
            while let Some(operation) = self.next_chain_operation(source)? {
                if let ChainOperation::Member(member) = operation {
                    if !member.member.numeric || !leading {
                        leading = false;
                        let next_member = matches!(source.parts.peek(), Some(ChainPart::Member(_)));
                        if !next_member || member.source_break() {
                            source.pending = Some(member);
                            break;
                        }
                    }
                    last_name = member.member.source.name();
                    self.write_chain_member(member, false, &mut facts, writer)?;
                } else if let ChainOperation::Call(arguments) = operation {
                    last_name = None;
                    self.write_chain_call(arguments, source, &mut facts, writer)?;
                }
                head_parts += 1;
            }
            Ok::<_, FormatError>(())
        };
        let (doc, preferred_breaks) = match head.layout {
            ChainHeadLayout::NeutralAtom => {
                let (fragment, ()) = writer.record_with_value(write_head)?;
                (
                    ChainGroupDoc::Recorded(fragment),
                    fragment.preferred_breaks(),
                )
            }
            ChainHeadLayout::Grouped => {
                let (group, ()) = writer.record_group_with_value(write_head)?;
                let doc = if facts.has_parent_call_group {
                    ChainGroupDoc::ParentCall(group.contents)
                } else {
                    ChainGroupDoc::Recorded(group.grouped)
                };
                (doc, group.grouped.preferred_breaks())
            }
        };
        facts.preferred_breaks = preferred_breaks;
        let merge_head = !head.has_comments
            && if head_parts == 0 {
                source
                    .pending
                    .as_ref()
                    .is_some_and(|member| head.attach || member.member.source.is_computed())
            } else {
                last_name.is_some_and(|name| factory_name(self.ast.get_utf8(name.name(self.ast))))
            };
        Ok((ChainGroup::new(doc, None, facts), merge_head))
    }

    fn record_chain_group(
        &mut self,
        source: &mut ChainSource<'ast, '_>,
        writer: &mut Writer<'ast>,
    ) -> Result<ChainGroup<'ast>, FormatError> {
        let mut before = None;
        let mut facts = GroupFacts::default();
        let mut has_parts = false;
        let mut saw_call = false;
        let (group, ()) = writer.record_group_with_value(|writer| {
            while let Some(operation) = self.next_chain_operation(source)? {
                match operation {
                    ChainOperation::Member(member) => {
                        if has_parts
                            && (saw_call && !member.member.numeric || member.source_break())
                        {
                            source.pending = Some(member);
                            break;
                        }
                        if !has_parts {
                            let trailing = if member.boundary.has_before_comments() {
                                let (trailing, ()) = writer.record_with_value(|writer| {
                                    self.format_member_trailing(&member.boundary, writer)
                                })?;
                                Some(trailing)
                            } else {
                                None
                            };
                            before = Some(RecordedBoundary {
                                trailing,
                                separator: member.separator(),
                            });
                        }
                        self.write_chain_member(member, !has_parts, &mut facts, writer)?;
                    }
                    ChainOperation::Call(arguments) => {
                        saw_call = true;
                        self.write_chain_call(arguments, source, &mut facts, writer)?;
                    }
                }
                has_parts = true;
            }
            Ok::<_, FormatError>(())
        })?;
        facts.preferred_breaks = group.grouped.preferred_breaks();
        Ok(ChainGroup::new(
            ChainGroupDoc::for_group(group, facts.has_parent_call_group),
            before,
            facts,
        ))
    }

    fn write_chain_member(
        &mut self,
        member: PendingMember<'ast>,
        boundary_outside: bool,
        facts: &mut GroupFacts,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        facts.has_boundary_comments |= member.boundary.has_comments;
        facts.source_break |= member.source_break();
        if !boundary_outside {
            self.format_member_trailing(&member.boundary, writer)?;
            member.separator().write(false, writer);
        }
        self.format_member_access(&member.member.source, member.boundary, writer)
    }

    fn write_chain_call(
        &mut self,
        arguments: SourceArguments<'ast>,
        source: &mut ChainSource<'ast, '_>,
        facts: &mut GroupFacts,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        // Long chains with complex arguments keep an AST-owned expansion
        // decision when a nested collection changes its source layout.
        if matches!(source.argument_layout, ChainArgumentLayout::Inspect) {
            let complex = arguments
                .arguments()
                .iter()
                .any(|id| !self.simple_call_argument(self.ast.get_node_in_sub_range(id), 2));
            if complex {
                source.argument_layout = ChainArgumentLayout::Complex;
            }
        }
        // Comments owned by the actual arguments may move inside their opener.
        // Only an outside prefix is a boundary for both merging and expansion.
        let has_boundary_comments = arguments.has_outside_prefix();
        facts.has_boundary_comments |= has_boundary_comments;
        facts.has_parent_call_group |= matches!(arguments.group, ArgumentGroup::ParentCall);
        facts.source_break |= has_boundary_comments;
        let missing_function = if facts.has_function {
            None
        } else {
            Some(&mut facts.has_function)
        };
        source.tail = self.format_source_arguments_with_function_fact(
            arguments,
            source.tail.take(),
            missing_function,
            writer,
        )?;
        Ok(())
    }
}

fn factory_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
        || !name.is_empty() && name.bytes().all(|byte| byte == b'_' || byte == b'$')
}

fn write_group_boundary<'ast>(group: &ChainGroup<'ast>, expanded: bool, writer: &mut Writer<'ast>) {
    if let Some(boundary) = group.before {
        // The next member owns the preceding receiver's trailing comments.
        // Entering the shared indent affects any source lines in that run.
        if let Some(trailing) = boundary.trailing {
            writer.write_fragment(trailing);
        }
        boundary.separator.write(expanded, writer);
    }
}

fn write_group_contents<'ast>(group: &ChainGroup<'ast>, expanded: bool, writer: &mut Writer<'ast>) {
    match group.doc {
        ChainGroupDoc::Recorded(fragment) => writer.write_fragment(fragment),
        ChainGroupDoc::ParentCall(fragment) => {
            if expanded {
                writer.write_fragment(fragment);
            } else {
                writer.group(false, |writer| writer.write_fragment(fragment));
            }
        }
        ChainGroupDoc::Neutral(fragment) => writer.write_neutral_fragment(fragment),
    }
}

fn write_group<'ast>(group: &ChainGroup<'ast>, expanded: bool, writer: &mut Writer<'ast>) {
    write_group_boundary(group, expanded, writer);
    write_group_contents(group, expanded, writer);
}

fn write_compact<'ast>(groups: &ChainGroups<'ast>, writer: &mut Writer<'ast>) {
    write_group(&groups.head, false, writer);
    for group in &groups.tail {
        write_group(group, false, writer);
    }
}

fn write_expanded<'ast>(groups: &ChainGroups<'ast>, merge_head: bool, writer: &mut Writer<'ast>) {
    write_group(&groups.head, true, writer);
    let tail = if merge_head && let Some((first, remaining)) = groups.tail.split_first() {
        write_group_boundary(first, false, writer);
        write_group_contents(first, true, writer);
        remaining
    } else {
        &groups.tail
    };
    writer.indent(|writer| {
        for group in tail {
            write_group(group, true, writer);
        }
    });
}
