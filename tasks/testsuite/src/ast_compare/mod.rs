//! Structural comparison for parse/print round trips, independent of codegen.
//!
//! Compare reachable nodes and decoded strings, never arena IDs or source spans.
//! Compare the parsed structure directly, including parenthesis nodes when
//! present. Literal spellings may change; node kinds, flags, tagged-template raw
//! text, directive spellings, and optional-chain boundaries may not.

use swc_next_ecma_ast::*;

mod generated;

pub(crate) fn compare<'left, 'right>(
    left: &Ast<'_, 'left>,
    right: &Ast<'_, 'right>,
) -> Result<(), String> {
    let asts = AstPair { left, right };
    if left.source_type() != right.source_type() {
        return Err(format!(
            "source type: {:?} != {:?}",
            left.source_type(),
            right.source_type()
        ));
    }
    left.root_program().compare(right.root_program(), &asts)
}

struct AstPair<'a, 'b, 'left, 'right> {
    left: &'a Ast<'a, 'left>,
    right: &'b Ast<'b, 'right>,
}

impl<'left, 'right> AstPair<'_, '_, 'left, 'right> {
    fn nodes(&self, left: NodeId<'left>, right: NodeId<'right>) -> Result<(), String> {
        let left = self.left.kind_data(left);
        let right = self.right.kind_data(right);
        generated::compare_nodes(left, right, self)
    }
}

trait Compare<'left, 'right, Other = Self>: Sized {
    fn compare(self, other: Other, asts: &AstPair<'_, '_, 'left, 'right>) -> Result<(), String>;
}

fn compare_bigint<'left, 'right>(
    left: AstUtf8<'left>,
    right: AstUtf8<'right>,
    asts: &AstPair<'_, '_, 'left, 'right>,
) -> Result<(), String> {
    let left = asts.left.get_utf8(left);
    let right = asts.right.get_utf8(right);
    fn digits(text: &str) -> impl Iterator<Item = u8> + '_ {
        text.bytes().filter(|byte| *byte != b'_')
    }
    if digits(left).eq(digits(right)) {
        Ok(())
    } else {
        Err(format!("BigInt spelling: {left:?} != {right:?}"))
    }
}

macro_rules! compare_value {
    ($ty:ty) => {
        impl<'left, 'right> Compare<'left, 'right> for $ty {
            fn compare(
                self,
                other: Self,
                _: &AstPair<'_, '_, 'left, 'right>,
            ) -> Result<(), String> {
                if self == other {
                    Ok(())
                } else {
                    Err(format!("value: {self:?} != {other:?}"))
                }
            }
        }
    };
}
use compare_value;

impl<'left, 'right> Compare<'left, 'right, AstUtf8<'right>> for AstUtf8<'left> {
    fn compare(
        self,
        other: AstUtf8<'right>,
        asts: &AstPair<'_, '_, 'left, 'right>,
    ) -> Result<(), String> {
        let left = asts.left.get_utf8(self);
        let right = asts.right.get_utf8(other);
        if left == right {
            Ok(())
        } else {
            Err(format!("value: {left:?} != {right:?}"))
        }
    }
}

impl<'left, 'right> Compare<'left, 'right, AstWtf8<'right>> for AstWtf8<'left> {
    fn compare(
        self,
        other: AstWtf8<'right>,
        asts: &AstPair<'_, '_, 'left, 'right>,
    ) -> Result<(), String> {
        let left = asts.left.get_wtf8(self);
        let right = asts.right.get_wtf8(other);
        if left == right {
            Ok(())
        } else {
            Err(format!("value: {left:?} != {right:?}"))
        }
    }
}

impl<'left, 'right> Compare<'left, 'right> for f64 {
    fn compare(self, other: Self, _: &AstPair<'_, '_, 'left, 'right>) -> Result<(), String> {
        // Preserve negative zero and compare NaN payloads deterministically.
        if self.to_bits() == other.to_bits() {
            Ok(())
        } else {
            Err(format!("value: {self:?} != {other:?}"))
        }
    }
}

impl<'left, 'right, T: Compare<'left, 'right, U>, U> Compare<'left, 'right, Option<U>>
    for Option<T>
{
    fn compare(
        self,
        other: Option<U>,
        asts: &AstPair<'_, '_, 'left, 'right>,
    ) -> Result<(), String> {
        match (self, other) {
            (Some(left), Some(right)) => left.compare(right, asts),
            (None, None) => Ok(()),
            _ => Err("presence changed".into()),
        }
    }
}

impl<'left, 'right, T: Compare<'left, 'right, U> + ExtraDataValue<'left>, U: ExtraDataValue<'right>>
    Compare<'left, 'right, SubRange<'right, U>> for SubRange<'left, T>
{
    fn compare(
        self,
        other: SubRange<'right, U>,
        asts: &AstPair<'_, '_, 'left, 'right>,
    ) -> Result<(), String> {
        if self.len() != other.len() {
            return Err("list length changed".into());
        }
        for (index, (left, right)) in self.iter().zip(other.iter()).enumerate() {
            let left = asts.left.get_node_in_sub_range(left);
            let right = asts.right.get_node_in_sub_range(right);
            left.compare(right, asts)
                .map_err(|error| format!("[{index}].{error}"))?;
        }
        Ok(())
    }
}
