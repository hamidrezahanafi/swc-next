use swc_next_ecma_ast::{NodeKindData, Program, Stmt};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

use crate::mutation::{MutationTransaction, PrivateBinding};

const CLASS_HELPERS_SOURCE: &str = r#"
function _set_prototype_of(o, p) {
    _set_prototype_of = Object.setPrototypeOf || function setPrototypeOf(o, p) {
        o.__proto__ = p;
        return o;
    };
    return _set_prototype_of(o, p);
}
function _inherits(subClass, superClass) {
    if (typeof superClass !== "function" && superClass !== null) {
        throw new TypeError("Super expression must either be null or a function");
    }
    subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, writable: true, configurable: true } });
    if (superClass) _set_prototype_of(subClass, superClass);
}
function _get_prototype_of(o) {
    _get_prototype_of = Object.setPrototypeOf ? Object.getPrototypeOf : function getPrototypeOf(o) {
        return o.__proto__ || Object.getPrototypeOf(o);
    };
    return _get_prototype_of(o);
}
function _type_of(obj) {
    "@swc/helpers - typeof";
    return obj && typeof Symbol !== "undefined" && obj.constructor === Symbol ? "symbol" : typeof obj;
}
function _assert_this_initialized(self) {
    if (self === void 0) throw new ReferenceError("this hasn't been initialised - super() hasn't been called");
    return self;
}
function _possible_constructor_return(self, call, allowPrimitive) {
    if (call && (_type_of(call) === "object" || typeof call === "function")) return call;
    if (call !== void 0 && !allowPrimitive) {
        throw new TypeError("Derived constructors may only return object or undefined");
    }
    return _assert_this_initialized(self);
}
function _is_native_reflect_construct() {
    try {
        var result = !Boolean.prototype.valueOf.call(Reflect.construct(Boolean, [], function() {}));
    } catch (_) {}
    return (_is_native_reflect_construct = function() {
        return !!result;
    })();
}
function _call_super(_this, derived, args) {
    derived = _get_prototype_of(derived);
    return _possible_constructor_return(
        _this,
        _is_native_reflect_construct()
            ? Reflect.construct(derived, args || [], _get_prototype_of(_this).constructor)
            : derived.apply(_this, args),
        true
    );
}
"#;

const SUPER_GET_HELPERS_SOURCE: &str = r#"
function _super_prop_base(object, property) {
    while (!Object.prototype.hasOwnProperty.call(object, property)) {
        object = Object.getPrototypeOf(object);
        if (object === null) break;
    }
    return object;
}
function _get() {
    if (typeof Reflect !== "undefined" && Reflect.get) {
        _get = Reflect.get.bind();
    } else {
        _get = function _get(target, property, receiver) {
            var base = _super_prop_base(target, property);
            if (!base) return;
            var desc = Object.getOwnPropertyDescriptor(base, property);
            if (desc.get) return desc.get.call(arguments.length < 3 ? target : receiver);
            return desc.value;
        };
    }
    return _get.apply(null, arguments);
}
"#;

const SUPER_GET_SET_HELPERS_SOURCE: &str = r#"
function _super_prop_base(object, property) {
    while (!Object.prototype.hasOwnProperty.call(object, property)) {
        object = Object.getPrototypeOf(object);
        if (object === null) break;
    }
    return object;
}
function _get() {
    if (typeof Reflect !== "undefined" && Reflect.get) {
        _get = Reflect.get.bind();
    } else {
        _get = function _get(target, property, receiver) {
            var base = _super_prop_base(target, property);
            if (!base) return;
            var desc = Object.getOwnPropertyDescriptor(base, property);
            if (desc.get) return desc.get.call(arguments.length < 3 ? target : receiver);
            return desc.value;
        };
    }
    return _get.apply(null, arguments);
}
function _set_property(target, property, value, receiver) {
    if (typeof Reflect !== "undefined" && Reflect.set) {
        _set_property = Reflect.set;
    } else {
        _set_property = function _set_property(target, property, value, receiver) {
            var base = _super_prop_base(target, property);
            var desc;
            if (base) {
                desc = Object.getOwnPropertyDescriptor(base, property);
                if (desc.set) {
                    desc.set.call(receiver, value);
                    return true;
                }
                if (!desc.writable) return false;
            }
            desc = Object.getOwnPropertyDescriptor(receiver, property);
            if (desc) {
                if (!desc.writable) return false;
                desc.value = value;
                Object.defineProperty(receiver, property, desc);
            } else {
                Object.defineProperty(receiver, property, {
                    value: value,
                    enumerable: true,
                    configurable: true,
                    writable: true
                });
            }
            return true;
        };
    }
    return _set_property(target, property, value, receiver);
}
function _set(target, property, value, receiver, isStrict) {
    var success = _set_property(target, property, value, receiver || target);
    if (!success && isStrict) throw new TypeError("failed to set property");
    return value;
}
"#;

const CLASS_NAME_TDZ_REF_SOURCE: &str = r#"
function _class_name_tdz_ref(value, name) {
    if (value === _class_name_tdz_ref) {
        throw new ReferenceError("Class \"" + name + "\" cannot be referenced before initialization.");
    }
    return value;
}
"#;

const HELPER_SPECS: [(&str, &str); 8] = [
    ("swc.helper.set_prototype_of", "_set_prototype_of"),
    ("swc.helper.inherits", "_inherits"),
    ("swc.helper.get_prototype_of", "_get_prototype_of"),
    ("swc.helper.type_of", "_type_of"),
    (
        "swc.helper.assert_this_initialized",
        "_assert_this_initialized",
    ),
    (
        "swc.helper.possible_constructor_return",
        "_possible_constructor_return",
    ),
    (
        "swc.helper.is_native_reflect_construct",
        "_is_native_reflect_construct",
    ),
    ("swc.helper.call_super", "_call_super"),
];

const SUPER_GET_HELPER_SPECS: [(&str, &str); 2] = [
    ("swc.helper.super_prop_base", "_super_prop_base"),
    ("swc.helper.get", "_get"),
];

const SUPER_GET_SET_HELPER_SPECS: [(&str, &str); 4] = [
    ("swc.helper.super_prop_base", "_super_prop_base"),
    ("swc.helper.get", "_get"),
    ("swc.helper.set_property", "_set_property"),
    ("swc.helper.set", "_set"),
];

const CLASS_NAME_TDZ_REF_SPECS: [(&str, &str); 1] =
    [("swc.helper.class_name_tdz_ref", "_class_name_tdz_ref")];

#[derive(Clone, Copy)]
pub(super) struct ClassHelpers {
    pub inherits: PrivateBinding,
    pub get_prototype_of: PrivateBinding,
    pub get: Option<PrivateBinding>,
    pub set: Option<PrivateBinding>,
    pub class_name_tdz_ref: Option<PrivateBinding>,
    pub call_super: PrivateBinding,
    pub assert_this_initialized: PrivateBinding,
    pub possible_constructor_return: PrivateBinding,
}

pub(super) struct PublishedClassHelpers {
    pub bindings: ClassHelpers,
    pub statements: Vec<Stmt>,
}

pub(super) fn publish(
    program: Program,
    needs_super_get: bool,
    needs_super_set: bool,
    needs_class_name_tdz: bool,
    tx: &mut MutationTransaction<'_, '_>,
) -> PublishedClassHelpers {
    let (bindings, mut statements) =
        publish_source(program, CLASS_HELPERS_SOURCE, &HELPER_SPECS, tx);
    let super_bindings = needs_super_get.then(|| {
        let (source, specs): (&'static str, &'static [(&'static str, &'static str)]) =
            if needs_super_set {
                (SUPER_GET_SET_HELPERS_SOURCE, &SUPER_GET_SET_HELPER_SPECS)
            } else {
                (SUPER_GET_HELPERS_SOURCE, &SUPER_GET_HELPER_SPECS)
            };
        let (bindings, extra_statements) = publish_source(program, source, specs, tx);
        statements.extend(extra_statements);
        bindings
    });
    let class_name_tdz_ref = needs_class_name_tdz.then(|| {
        let (bindings, extra_statements) = publish_source(
            program,
            CLASS_NAME_TDZ_REF_SOURCE,
            &CLASS_NAME_TDZ_REF_SPECS,
            tx,
        );
        statements.extend(extra_statements);
        bindings[0]
    });
    PublishedClassHelpers {
        bindings: ClassHelpers {
            inherits: bindings[1],
            get_prototype_of: bindings[2],
            get: super_bindings.as_ref().map(|bindings| bindings[1]),
            set: super_bindings.as_ref().and_then(|bindings| {
                if needs_super_set {
                    Some(bindings[3])
                } else {
                    None
                }
            }),
            class_name_tdz_ref,
            call_super: bindings[7],
            assert_this_initialized: bindings[4],
            possible_constructor_return: bindings[5],
        },
        statements,
    }
}

fn publish_source(
    program: Program,
    source: &'static str,
    specs: &'static [(&'static str, &'static str)],
    tx: &mut MutationTransaction<'_, '_>,
) -> (Vec<PrivateBinding>, Vec<Stmt>) {
    let parsed = tx.build_detached(|ast| {
        Parser::init_generated(
            ast,
            source,
            Options {
                source_type: program.source_type(ast),
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse()
    });
    assert!(
        parsed.diagnostics.is_empty(),
        "canonical class helpers must parse: {:#?}",
        parsed.diagnostics
    );

    let template = analyze(
        parsed.program,
        tx.ast(),
        AnalyzeOptions {
            features: tx.semantic().features(),
            ..AnalyzeOptions::default()
        },
    )
    .semantic;
    let source_scope = template.scope_of(parsed.program.node_id());
    let target_scope = tx.semantic().scope_of(program.node_id());
    let mut bindings = Vec::with_capacity(specs.len());
    for &(key, base) in specs {
        let source_symbol = template
            .binding_text(tx.ast(), source_scope, base)
            .unwrap_or_else(|| panic!("missing canonical helper declaration {base}"));
        let source = template.symbol(source_symbol);
        let declaration = template.declarations(source_symbol)[0];
        let function = match tx.ast().kind_data(
            template
                .parent_of(declaration.node)
                .expect("helper binding must have a function parent"),
        ) {
            NodeKindData::Function(function) => function,
            _ => panic!("helper declaration must be a function"),
        };
        let references: Vec<_> = template
            .uses(source_symbol)
            .iter()
            .map(|reference| template.reference(*reference).node)
            .collect();
        let binding = tx.declare_shared_private(target_scope, key, base, source.flags);
        tx.build_detached(|ast| {
            function.set_id(ast, Some(binding.declaration));
            for node in references {
                let NodeKindData::IdentifierReference(reference) = ast.kind_data(node) else {
                    panic!("helper symbol use must be an identifier reference")
                };
                reference.set_name(ast, binding.text);
                ast.set_syntax_context(reference.node_id(), binding.context);
            }
        });
        bindings.push(binding);
    }
    drop(template);

    let imported = tx.import_generated_program(parsed.program, program.node_id(), target_scope);
    (bindings, imported.statements)
}
