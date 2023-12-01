use std::iter;

use full_moon::{
    ast::{
        punctuated::{Pair, Punctuated},
        types::{ExportedTypeDeclaration, GenericParameterInfo, IndexedTypeInfo, TypeInfo},
        Block, Call, Expression, FunctionCall, LastStmt, LocalAssignment, Prefix, Return, Stmt,
        Suffix, Value, Var,
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
            multi_line: None,
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

impl Visitor for CollectTypeExports {
    fn visit_block(&mut self, node: &Block) {
        let export_statements = node.stmts().filter_map(|statement| match statement {
            full_moon::ast::Stmt::ExportedTypeDeclaration(declaration) => {
                let declaration = declaration.type_declaration();

                let indexed_type_info = if let Some(generics_declaration) = declaration.generics() {
                    let arrows = generics_declaration.arrows().clone();

                    let generics = generics_declaration
                        .generics()
                        .pairs()
                        .map(|pair| {
                            let punctation = pair.punctuation().cloned();

                            let generic_value = match pair.value().parameter() {
                                GenericParameterInfo::Name(name) => TypeInfo::Basic(name.clone()),
                                GenericParameterInfo::Variadic { name, ellipse } => {
                                    TypeInfo::GenericPack {
                                        name: name.clone(),
                                        ellipse: ellipse.clone(),
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

                    IndexedTypeInfo::Generic {
                        base: declaration.type_name().clone(),
                        arrows,
                        generics,
                    }
                } else {
                    IndexedTypeInfo::Basic(declaration.type_name().clone())
                };

                let re_declaration = declaration.clone().with_type_definition(TypeInfo::Module {
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
    let parsed_code =
        full_moon::parse(code).map_err(|err| format!("unable to parse code: {}", err))?;

    let identifier = create_identifier("module");
    let require_token = create_identifier("require");
    let module_path_token = create_string(module_path);

    let module_definition = LocalAssignment::new(into_punctuated(identifier.clone()))
        .with_expressions(into_punctuated(Expression::Value {
            value: Box::new(Value::FunctionCall(
                FunctionCall::new(Prefix::Name(require_token)).with_suffixes(vec![Suffix::Call(
                    Call::AnonymousCall(full_moon::ast::FunctionArgs::String(module_path_token)),
                )]),
            )),
            type_assertion: None,
        }))
        .with_equal_token(Some(create_symbol(Symbol::Equal)));

    let return_module = Return::new().with_returns(into_punctuated(Expression::Value {
        value: Box::new(Value::Var(Var::Name(identifier.clone()))),
        type_assertion: None,
    }));

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
        Config::new()
            .with_quote_style(stylua_lib::QuoteStyle::AutoPreferSingle)
            .with_column_width(80),
        None,
        stylua_lib::OutputVerification::None,
    )
    .map_err(|err| format!("unable to format code: {}", err))?;

    Ok(full_moon::print(&formatted_ast))
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
