use std::iter;

use full_moon::{
    ast::{
        luau::{
            ExportedTypeDeclaration, GenericDeclarationParameter, GenericParameterInfo,
            IndexedTypeInfo, TypeFieldKey, TypeInfo,
        },
        punctuated::{Pair, Punctuated},
        Block, Call, Expression, FunctionCall, LastStmt, LocalAssignment, Prefix, Return, Stmt,
        Suffix, Var,
    },
    tokenizer::{Symbol, Token, TokenReference, TokenType},
    visitors::Visitor,
    ShortString,
};
use stylua_lib::Config;
use wasm_bindgen::prelude::wasm_bindgen;

fn into_punctuated<T>(value: T) -> Punctuated<T> {
    let mut punctuated = Punctuated::new();
    punctuated.push(Pair::End(value));
    punctuated
}

fn create_identifier(content: &str) -> TokenReference {
    TokenReference::new(
        Vec::new(),
        Token::new(TokenType::Identifier {
            identifier: ShortString::new(content),
        }),
        Vec::new(),
    )
}

fn create_string(content: &str) -> TokenReference {
    TokenReference::new(
        Vec::new(),
        Token::new(TokenType::StringLiteral {
            literal: ShortString::new(content),
            multi_line_depth: 0,
            quote_type: full_moon::tokenizer::StringLiteralQuoteType::Double,
        }),
        Vec::new(),
    )
}

fn create_symbol(symbol: Symbol) -> TokenReference {
    TokenReference::new(
        Vec::new(),
        Token::new(TokenType::Symbol { symbol }),
        Vec::new(),
    )
}

struct CollectTypeExports {
    identifier: TokenReference,
    statements: Vec<Stmt>,
}

impl CollectTypeExports {
    fn new(identifier: TokenReference) -> Self {
        Self {
            identifier,
            statements: Default::default(),
        }
    }
}

fn remove_generics_foreign_default(
    generic_parameters: Punctuated<GenericDeclarationParameter>,
) -> Punctuated<full_moon::ast::luau::GenericDeclarationParameter> {
    let new_generics = generic_parameters
        .into_pairs()
        .map(|mut generic_parameter| {
            if let Some(default_generic_parameter) = generic_parameter.value_mut().default_type() {
                if has_private_type(default_generic_parameter) {
                    let new_value = generic_parameter.value().clone().with_default(None);

                    *generic_parameter.value_mut() = new_value;
                }
            }
            generic_parameter
        })
        .collect();
    new_generics
}

// make a best effort at keeping types which can re-export easily
fn has_private_type(type_info: &TypeInfo) -> bool {
    let mut check_types = vec![type_info];

    while let Some(current) = check_types.pop() {
        match current {
            TypeInfo::Array { type_info, .. } => {
                check_types.push(type_info);
            }
            TypeInfo::Basic(token_reference) => {
                if let Some(value) = is_standard_type(token_reference) {
                    return value;
                }
            }
            TypeInfo::String(_) | TypeInfo::Boolean(_) => {}
            TypeInfo::Callback {
                arguments,
                return_type,
                ..
            } => {
                check_types.push(return_type);
                for argument in arguments.into_iter() {
                    check_types.push(argument.type_info());
                }
            }
            TypeInfo::Intersection(type_intersection) => {
                for sub_type in type_intersection.types().into_iter() {
                    check_types.push(sub_type);
                }
            }
            TypeInfo::Union(type_union) => {
                for sub_type in type_union.types().into_iter() {
                    check_types.push(sub_type);
                }
            }
            TypeInfo::Optional { base, .. } => {
                check_types.push(base);
            }
            TypeInfo::Table { fields, .. } => {
                for field in fields.into_iter() {
                    check_types.push(field.value());
                    match field.key() {
                        TypeFieldKey::Name(_) => {}
                        TypeFieldKey::IndexSignature { inner, .. } => {
                            check_types.push(inner);
                        }
                        _ => return true,
                    }
                }
            }
            TypeInfo::Tuple { types, .. } => {
                for sub_type in types.into_iter() {
                    check_types.push(sub_type);
                }
            }
            TypeInfo::Variadic { type_info, .. } => {
                check_types.push(type_info);
            }
            TypeInfo::Generic { .. }
            | TypeInfo::GenericPack { .. }
            | TypeInfo::Typeof { .. }
            | TypeInfo::Module { .. }
            | TypeInfo::VariadicPack { .. }
            | _ => {
                return true;
            }
        }
    }

    false
}

fn is_standard_type(token_reference: &TokenReference) -> Option<bool> {
    match token_reference.token().to_string().as_str() {
        "string" | "boolean" | "nil" | "number" | "userdata" | "buffer" | "thread" | "never"
        | "any" | "unknown" => {}
        _ => return Some(true),
    }
    None
}

impl Visitor for CollectTypeExports {
    fn visit_block(&mut self, node: &Block) {
        let export_statements = node.stmts().filter_map(|statement| match statement {
            Stmt::ExportedTypeDeclaration(declaration) => {
                let declaration = declaration.type_declaration();

                let (indexed_type_info, new_generics) = if let Some(generics_declaration) =
                    declaration.generics()
                {
                    let arrows = generics_declaration.arrows().clone();

                    let generics = generics_declaration
                        .generics()
                        .pairs()
                        .map(|pair| {
                            let punctation = pair.punctuation().cloned();

                            let generic_value = match pair.value().parameter() {
                                GenericParameterInfo::Name(name) => TypeInfo::Basic(name.clone()),
                                GenericParameterInfo::Variadic { name, ellipsis } => {
                                    TypeInfo::GenericPack {
                                        name: name.clone(),
                                        ellipsis: ellipsis.clone(),
                                    }
                                }
                                _ => unimplemented!("unknown GenericParameterInfo variant"),
                            };

                            (punctation, generic_value)
                        })
                        .fold(Punctuated::new(), |mut punctuated, (punctuation, value)| {
                            if let Some(token) = punctuation {
                                punctuated.push(Pair::Punctuated(value, token));
                            } else {
                                punctuated.push(Pair::End(value));
                            }
                            punctuated
                        });

                    (
                        IndexedTypeInfo::Generic {
                            base: declaration.type_name().clone(),
                            arrows,
                            generics,
                        },
                        Some(generics_declaration.clone().with_generics(
                            remove_generics_foreign_default(
                                generics_declaration.generics().clone(),
                            ),
                        )),
                    )
                } else {
                    (
                        IndexedTypeInfo::Basic(declaration.type_name().clone()),
                        None,
                    )
                };

                let re_declaration = declaration
                    .clone()
                    .with_generics(new_generics)
                    .with_type_definition(TypeInfo::Module {
                        module: self.identifier.clone(),
                        punctuation: TokenReference::symbol(".").ok()?,
                        type_info: Box::new(indexed_type_info),
                    });

                Some(Stmt::ExportedTypeDeclaration(ExportedTypeDeclaration::new(
                    re_declaration,
                )))
            }
            _ => None,
        });

        self.statements.extend(export_statements);
    }
}

#[wasm_bindgen]
pub fn reexport(module_path: &str, code: &str) -> Result<String, String> {
    let parsed_code = full_moon::parse(code).map_err(|errors| match errors.len() {
        0 => "failed to parse code".to_owned(),
        1 => {
            format!(
                "unable to parse code: {}",
                errors.first().expect("expected one error")
            )
        }
        _ => {
            let display_errors: Vec<_> = errors.into_iter().map(|err| err.to_string()).collect();
            format!("unable to parse code:\n- {}", display_errors.join("\n- "))
        }
    })?;

    let identifier = create_identifier("module");
    let require_token = create_identifier("require");
    let module_path_token = create_string(module_path);

    let module_definition = LocalAssignment::new(into_punctuated(identifier.clone()))
        .with_expressions(into_punctuated(Expression::FunctionCall(
            FunctionCall::new(Prefix::Name(require_token)).with_suffixes(vec![Suffix::Call(
                Call::AnonymousCall(full_moon::ast::FunctionArgs::String(module_path_token)),
            )]),
        )))
        .with_equal_token(Some(create_symbol(Symbol::Equal)));

    let return_module = Return::new().with_returns(into_punctuated(Expression::Var(Var::Name(
        identifier.clone(),
    ))));

    let mut collect_exports = CollectTypeExports::new(identifier);

    collect_exports.visit_ast(&parsed_code);

    let new_ast = parsed_code.clone().with_nodes(
        Block::new()
            .with_stmts(
                iter::once(Stmt::LocalAssignment(module_definition))
                    .chain(collect_exports.statements)
                    .map(|statement| (statement, None))
                    .collect(),
            )
            .with_last_stmt(Some((LastStmt::Return(return_module), None))),
    );

    let formatted_ast = stylua_lib::format_ast(
        new_ast,
        get_stylua_config(),
        None,
        stylua_lib::OutputVerification::None,
    )
    .map_err(|err| format!("unable to format code: {}", err))?;

    Ok(formatted_ast.to_string())
}

fn get_stylua_config() -> Config {
    let mut config = Config::new();
    config.quote_style = stylua_lib::QuoteStyle::AutoPreferSingle;
    config.column_width = 80;
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_simple_type_name() {
        let result = reexport(
            "../packages/module",
            r"
export type Value = number
        ",
        )
        .unwrap();

        insta::assert_snapshot!("export_simple_type_name", result);
    }

    #[test]
    fn export_simple_type_name_in_do_block() {
        let result = reexport(
            "../packages/module",
            r"
do
    export type Value = number
end
        ",
        )
        .unwrap();

        insta::assert_snapshot!("export_simple_type_name", result);
    }

    #[test]
    fn export_simple_generic_type() {
        let result = reexport(
            "../packages/module",
            r"
export type List<T> = { T }
        ",
        )
        .unwrap();

        insta::assert_snapshot!("export_simple_generic_type", result);
    }

    #[test]
    fn export_simple_generic_type_with_default() {
        let result = reexport(
            "../packages/module",
            r"
export type List<T=string> = { T }
        ",
        )
        .unwrap();

        insta::assert_snapshot!("export_simple_generic_type_with_default", result);
    }

    #[test]
    fn export_simple_generic_type_with_default_simple_array() {
        let result = reexport(
            "../packages/module",
            r"
export type List<T = { number }> = { T }
        ",
        )
        .unwrap();

        insta::assert_snapshot!(
            "export_simple_generic_type_with_default_simple_array",
            result
        );
    }

    #[test]
    fn export_generic_type_with_unexported_default() {
        let result = reexport(
            "../packages/module",
            r"
export type List<T=Object> = { T }
        ",
        )
        .unwrap();

        insta::assert_snapshot!("export_generic_type_with_unexported_default", result);
    }

    #[test]
    fn export_generic_type_with_unexported_default_array_of_object() {
        let result = reexport(
            "../packages/module",
            r"
export type List<T={ Object }> = { T }
        ",
        )
        .unwrap();

        insta::assert_snapshot!("export_generic_type_with_unexported_default", result);
    }

    #[test]
    fn export_generic_type_with_generic_pack() {
        let result = reexport(
            "../packages/module",
            r"
export type Fn<R...> = () -> R...
        ",
        )
        .unwrap();

        insta::assert_snapshot!("export_generic_type_with_generic_pack", result);
    }

    #[test]
    fn export_generic_type_with_generic_pack_with_default() {
        let result = reexport(
            "../packages/module",
            r"
export type Fn<R...=()> = () -> R...
        ",
        )
        .unwrap();

        insta::assert_snapshot!("export_generic_type_with_generic_pack_with_default", result);
    }

    #[test]
    fn export_generic_type_with_type_name_and_generic_pack() {
        let result = reexport(
            "../packages/module",
            r"
export type Fn<Arg, R...> = (Arg) -> R...
        ",
        )
        .unwrap();

        insta::assert_snapshot!(
            "export_generic_type_with_type_name_and_generic_pack",
            result
        );
    }
}
