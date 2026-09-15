# Plan: Express ESTree Differences in AST Attributes

## Goal

Move the differences between the Rust storage AST and the decoded ESTree AST
into attributes on AST definitions. Derive each attribute from an existing
generator special case, then remove the corresponding branches from both the
decoder and TypeScript declaration generators.


## 1. Start with StringLiteral: Override the Node Tag

The current StringLiteral decoder branch emits `type: "Literal"`, preserves
`value` and `raw`, and adds the usual `start` and `end`. Its only difference from
generic node construction is the tag.

`#[estree(ty = "Literal")]` is sufficient to remove this decoder branch.
The public TypeScript interface must still be named `StringLiteral`.

Other mappings change both the public name and the tag:

- IdentifierReference, BindingIdentifier, IdentifierName, and LabelIdentifier
  become Identifier.
- BindingProperty and ObjectProperty become Property.
- FunctionBody becomes BlockStatement.
- BindingRestElement becomes RestElement.

Introduce two distinct settings:

- `rename = "Property"` changes the public name. The tag follows this name by
  default.
- `ty = "Literal"` overrides only the tag.

The resolution rules are:

1. Public name: explicit `rename`, otherwise the default normalized Rust name.
2. Node tag: explicit `ty`, otherwise the public name.

For AST unions such as `Stmt`, `rename = "Statement"` changes only the public
name because the union has no independent runtime object.

Keep general naming conventions such as `Ts` to `TS` and `Jsx` to `JSX` in the
generator. Express exceptions such as TSBigIntKeyword and TSJSDocNullableType
on their definitions.

This removes the concrete name mappings in `estree_type()` and `public_name()`,
and the declaration generator's special test for the six literal structs.

## 2. Directive: Rename an Existing Field

Directive becomes an ExpressionStatement with its `value` field exposed as
`directive`. Its `expression` field is unchanged.

Use node-level `rename = "ExpressionStatement"` and field-level
`rename = "directive"`. No custom converter is needed.

The same field attribute covers all current explicit field renames:

| Rust field                      | ESTree field |
| ------------------------------- | ------------ |
| VariableDeclaration.declarators | declarations |
| TsEnumDeclaration.is_const      | const        |
| Directive.value                 | directive    |

Ordinary field names continue to use camelCase after removing Rust's `r#`
prefix. These annotations remove Directive's decoder and reshape branches, as
well as the concrete exceptions in `estree_field_name()`.

## 3. UnaryExpression and NullLiteral: Add Output Fields

UnaryExpression adds `prefix: true`. NullLiteral changes its tag to Literal and
adds `value: null` and `raw: "null"`.

Introduce a node-level output-field declaration, for example:

- `add_field(name = "prefix", value = true)`
- `add_field(name = "value", value = null)`

These fields belong to the ESTree output description and do not allocate Rust
storage fields.

The same operation covers the existing constant additions:

| Node                    | Added fields                                              |
| ----------------------- | --------------------------------------------------------- |
| BindingProperty         | `kind: "init"`, `method: false`                           |
| Function                | `expression: false`                                       |
| TsFunction              | `expression: false`, `body: null`                         |
| ArrowFunctionExpression | `id: null`, `generator: false`                            |
| TsModuleDeclaration     | `global: false`                                           |
| TsGlobalDeclaration     | `kind: "global"`, `global: true`                          |
| TsThisParameter         | `name: "this"`, `decorators: []`, `optional: false`       |
| TsMethodSignature       | `accessibility: null`, `readonly: false`, `static: false` |

Infer literal TypeScript types from constants: `true` produces type `true`, and
`null` produces type `null`. Empty arrays need an explicit element type, such
as Decorator for `decorators`.

The implemented constant forms are `null`, booleans, strings, and `[]`. Specify
the TypeScript element type of an empty array with
`add_field(name = "decorators", value = [], element_type = "Decorator")`.
Multiple `add_field` entries may share an `estree` attribute or use separate
attributes. Added fields cannot duplicate stored output fields or the common
`type`, `start`, and `end` properties.

### Add TS Conditions When a Case Requires Them

BindingProperty also adds `optional: false`, but only in TS mode. Extend the
same declaration with a condition:

`add_field(name = "optional", value = false, ts)`

Existing Rust fields use `#[estree(ts)]` directly. This unifies:

- The entries in `is_ts_field()`.
- The additions in `ts_extra_fields()`.
- TS branches in special decoders for functions, classes, members, and patterns.
- Corresponding calls to `optional()` in the declaration generator.

Additional conditional constants include Function.declare,
ExpressionStatement.directive, BindingRestElement.value, and
ExportDefaultDeclaration.exportKind. Identifier variants without stored TS
metadata also add decorators, optional, and typeAnnotation in TS mode.

The attribute describes runtime presence: the field exists only in TS mode.
The declaration generator derives optional properties from that condition.
A normal `Option<T>` field remains a required property whose value is
`T | null`.

Fixed fields on TS nodes, such as TsPropertySignature.accessibility and static,
TsIndexSignature.accessibility, and TsParameterProperty.static, can be declared
as unconditional additions. Their declaration optionality should not depend on
a check for a `Ts` prefix in the Rust name.

## 4. FunctionBody and Patterns: Combine Lists

The repeated structural operations are:

| Nodes                                | Operation                                   |
| ------------------------------------ | ------------------------------------------- |
| Program, FunctionBody, TsModuleBlock | Prepend directives to body                  |
| ArrayPattern                         | Append a present rest element to elements   |
| ObjectPattern                        | Append a present rest element to properties |
| FormalParameters                     | Append a present rest element to items      |

Express these operations on the source fields:

- `prepend_to = body` on directives.
- `append_to = elements`, `properties`, or `items` on rest.

Define their semantics together:

1. The source field is no longer emitted separately.
2. A list source contributes its elements in order.
3. An optional node source contributes one element when present and none when
   absent.
4. Existing holes in the destination array remain intact.
5. The output element type is the union of the contributing element types.

ObjectPattern.properties therefore becomes `Array<Property | RestElement>`
without a handwritten declaration override. ArrayPattern's existing
`Array<Pattern | null>` already admits RestElement through Pattern.

### Remove Parameter Wrappers

FormalParameter currently returns its pattern directly. Represent that using
`transparent = pattern`: the entire decoded value is the selected field,
without the wrapper's tag or positions.

FormalParameters can combine the same operations:

- Append rest to items.
- Return items through `transparent = items`.
- Exclude kind from the output.

It then decodes directly to an array. The current `{ params: ... }` object is
an internal decoder intermediate; removing it turns each caller's
`f_params.params` into an ordinary decoded field reference.

This simplifies Function, TsFunction, ArrowFunctionExpression, TsFunctionType,
TsConstructorType, TsMethodSignature, TsCallSignatureDeclaration, and
TsConstructSignatureDeclaration. Their remaining differences are covered by
other attributes in this plan.

The declaration generator infers references from the transparent output and
does not emit interfaces for the wrappers. These cases require list operations
and transparent output; they do not yet require general object flattening.

## 5. Function and Class: Derive the Node Tag

After parameter unwrapping, conditional fields, and constant additions,
Function's remaining difference is `type = function_type`. Class similarly
uses `class_type`.

Introduce `type_from = function_type` or `type_from = class_type`:

- The field provides the output tag.
- It is not also emitted as functionType or classType.
- Its scalar enum mapping supplies the possible tags.

Generate FunctionDeclaration and FunctionExpression interfaces from those
possibilities, then define Function as their union. Apply the same rule to
Class. This removes their entries in `node_names()` and the handwritten
Function and Class aliases.

### Computed Tags for Class Members

The next cases require a calculation rather than one field:

| Node               | Current tag selection                            |
| ------------------ | ------------------------------------------------ |
| MethodDefinition   | Abstract tag when `isTs && abstract`             |
| TsMethodDefinition | Abstract tag when `abstract`                     |
| PropertyDefinition | Accessor choice combined with `isTs && abstract` |

Introduce computed tag selection, provisionally
`type_via = PropertyDefinitionType`. The selector must describe its finite
set of possible tags along with the calculation. Both interface generation and
runtime construction consume this description.

Preserve the current distinction between MethodDefinition and
TsMethodDefinition during migration: the latter uses its abstract tag even
when the decoder is in JS mode. Any change to that behavior is a separate
decision.

Fields such as abstract and accessor can be excluded from direct output while
remaining available to the tag calculation. Output exclusion must not prevent
reading a field needed by a conversion.

### A Parent Overrides a Child's Tag

TsMethodDefinition currently changes its decoded value's tag to
TSEmptyBodyFunctionExpression. This is distinct from choosing its own tag.

For this concrete requirement, introduce a field-level
`type_override = "TSEmptyBodyFunctionExpression"`. It affects the child's
runtime tag, the field's declaration reference, and the generation of the
referenced output interface. The child otherwise retains TsFunction's output
fields, including its null body.

This narrow operation addresses the current case without requiring a general
node-view system.

### Union References Only Narrow Types

The declaration generator also narrows Function and Class references according
to the containing union:

- Expr uses FunctionExpression and ClassExpression.
- Decl and ExportDefaultDeclarationKind use FunctionDeclaration and
  ClassDeclaration.

A variant-level `ts_type` can express these existing declaration constraints.
It does not rewrite runtime tags. Keep this separate from a field-level
`type_override`.

## 6. Remaining Literals: Compute Output Values

After the earlier steps, the remaining value transformations are:

| Node            | Transformation                                                        |
| --------------- | --------------------------------------------------------------------- |
| NumericLiteral  | Use `String(value)` when raw is absent                                |
| BooleanLiteral  | Derive raw from value                                                 |
| BigIntLiteral   | Parse the bigint, produce decimal text, and extract raw source        |
| RegExpLiteral   | Construct a RegExp or null, build raw and regex, and sort regex flags |
| TemplateElement | Combine raw and cooked, interpreting the cooked-undefined flag        |
| JsxText         | Copy value to raw                                                     |
| TsMappedType    | Replace null optional with false                                      |

Extend output-field declarations with explicit sources:

- `value = ...` for constants.
- `from = value` for copying a storage field, as in JsxText.raw.
- `via = Converter` for a calculation.

Existing fields may also use `via`, such as NumericLiteral.raw. A converter
describes both the produced value and its output type. NumericLiteralRaw
produces string; RegExpValue produces `RegExp | null`; TemplateElementValue
produces an object containing raw and cooked.

Apply TsMappedType's null-to-false conversion only to optional. The readonly
field keeps the original TsMappedTypeModifier mapping.

Converters need explicit access to storage inputs. BigIntLiteral's stored raw
omits the trailing `n`, whereas its output raw is a source slice. Replacing the
output field must not hide or overwrite the stored input used for parsing.

BigIntLiteral.value and bigint share a parsed value. Support shared local
calculations and explicit dependencies rather than parsing twice or requiring
a handwritten decoder for the entire node.

Keep storage offsets and binary reads in the generator. Converters identify
their inputs by field and context, not by payload offsets.

## 7. Program and TemplateElement: Context and Positions

After list combination, Program still needs:

- sourceType from the decoding context.
- A TS-mode start position taken from the first combined body item, or end if
  the body is empty.

TemplateElement still adjusts start and end in TS mode, with end depending on
tail.

Extend field computation to read decoding context and override common output
fields such as start and end. This can reuse `via`, for example a ProgramStart
converter, rather than introducing a separate span attribute system.

Required inputs include source, sourceType, isTs, original positions, and
previously computed output values such as the combined body. Distinguish
original positions from final output positions so an override cannot
accidentally refer to itself.

Retain each existing raw-value policy. NumericLiteral's missing raw uses number
formatting; it must not silently switch to a source slice during migration.

## 8. Scalar Enums and Absent Values

### Generate Scalar Enum Mappings from Variants

Move the mappings in `ENUMS` onto scalar enum definitions and variants. Examples
include BinaryOperator.Equal mapping to `"=="`, Accessibility.None to null,
and TsMappedTypeModifier.True to true.

Generate runtime lookup tables and TypeScript literal unions from the same
metadata, preserving actual discriminants.

SourceType requires an explicit output exclusion: its Rust enum includes
Unambiguous, but the current decoded ESTree type only contains module, script,
and commonjs. Excluding a variant must not renumber the remaining values.

FunctionType, ClassType, and FormalParameterKind also need metadata suppressing
their standalone public declaration aliases. This replaces the declaration
generator's enum-name check.

Scalar enums need a metadata entry point that preserves the original Rust enum.
The current `#[ast]` macro turns AST definitions into NodeId wrappers and cannot
be applied unchanged. Include `ast/mod.rs`, where SourceType is defined, in the
metadata discovery process.

### Omit an Absent Property

TsModuleDeclaration.body omits the property when its value is None. Introduce
field-level `omit_if_none` for this specific distinction:

- Ordinary Option: required property with a nullable value.
- `omit_if_none`: optional property with a non-null value when present.
- `ts`: property presence depends on language mode.

### Keep Binary Decoding Responsibilities Separate

Primitive branches for booleans, numbers, strings, and Span remain binary
decoding rules. They do not need node-level attributes.

The ImportPhase-specific optional-enum branch can become a generic scalar-enum
operation once the schema records scalar enums and their optional storage
representation.

## Migration and Completion Criteria

Follow the numbered steps, starting with names, field renames, and constant or
conditional output fields. Each step should remove matching special cases from
both the decoder and declaration generators.

Parse the attributes into a shared ESTree output description alongside the
storage schema. This description records field presence, sources, transformations,
type references, and possible node tags. Keep the existing general interface
merging logic for storage nodes that share a public name.

Generate the Literal alias from the literal interfaces sharing its tag, using
the same grouping information needed by NodeMap, rather than a handwritten list.

Validate attribute dependencies and conflicting declarations during generation.
Converters should supply output types or structured fields so the declaration
generator does not grow a second list of node-specific corrections.

Review the encoder when introducing new output-exclusion semantics: it currently
also consumes `estree_skip`. An input needed by a computed output must remain
available, and reverse conversions must not be assumed to be automatic.

For implementation, compare generated decoder behavior and declarations with
the existing output, including JS/TS property presence, literal values, array
holes, parameter lists, dynamic tags, and positions. Use the existing ESTree
conformance workflow and relevant Node.js tests for behavioral validation.
Attribute migration should preserve AST storage layouts and node numbering.

The completion criterion for each migrated node is concrete: its definition
contains the required ESTree metadata, and neither generator needs to identify
that node by name to reconstruct its output shape.
