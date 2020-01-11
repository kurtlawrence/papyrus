use super::*;
use crate::code::{self, Statement};
use syn::export::ToTokens;
use syn::{self, Block, File, Item, Stmt};

/// Parses a line of input as a command.
/// Returns either a `Command` value or an `InputError` value.
pub fn parse_command(line: &str) -> InputResult {
    InputResult::Command(
        if line.starts_with(crate::CMD_PREFIX) {
            &line[crate::CMD_PREFIX.len()..]
        } else {
            line
        }
        .to_string(),
    )
}

/// Parses a line of input as a program.
pub fn parse_program(code: &str) -> InputResult {
    debug!("parse program: {}", code);

    let reterr = |e: syn::Error| {
        let e = e.to_string();
        if &e == "LexError" {
            InputResult::More
        } else {
            InputResult::InputError(e)
        }
    };

    // handle program level items such as #![feature(test)]
    if code.starts_with("#![") {
        return syn::parse_str::<File>(code)
            .map(|file| {
                let items = file
                    .attrs
                    .into_iter()
                    .map(|attr| attr.into_token_stream().to_string())
                    .map(fmt)
                    .map(|s| (s, true))
                    .collect::<Vec<_>>();
                InputResult::Program(Input {
                    items: items,
                    stmts: vec![],
                    crates: vec![],
                })
            })
            .unwrap_or_else(reterr);
    }

    let code = format!("{{ {} }}", code); // wrap in a block so the parser can parse through it without need to guess the type!

    syn::parse_str::<Block>(&code)
        .map(|block| {
            let mut stmts = Vec::new();
            let mut items: Vec<code::Item> = Vec::new();
            let mut crates = Vec::new();
            for stmt in block.stmts {
                match stmt {
                    Stmt::Local(local) => {
                        let mut s = fmt(local.into_token_stream().to_string());
                        s.pop(); // local is handled slightly differently, the trailing semi is dropped.
                        stmts.push(Statement {
                            expr: s,
                            semi: true,
                        })
                    }
                    Stmt::Item(item) => match parse_item(item) {
                        ParseItemResult::ExternCrate(string) => {
                            match CrateType::parse_str(&fmt(string)) {
                                Ok(c) => crates.push(c),
                                Err(e) => error!("crate parsing failed: {}", e),
                            }
                        }
                        ParseItemResult::Span(string) => items.push((fmt(string), false)),
                        ParseItemResult::MacroBlock(string, semi) => stmts.push(Statement {
                            expr: fmt(string),
                            semi,
                        }), // macro item are plopped in as exprs
                        ParseItemResult::Error(s) => return InputResult::InputError(s),
                    },
                    Stmt::Expr(expr) => match parse_expr(expr) {
                        Ok(string) => stmts.push(Statement {
                            expr: fmt(string),
                            semi: false,
                        }),
                        Err(s) => return InputResult::InputError(s),
                    },
                    Stmt::Semi(expr, _) => match parse_expr(expr) {
                        Ok(string) => stmts.push(Statement {
                            expr: fmt(string),
                            semi: true,
                        }),
                        Err(s) => return InputResult::InputError(s),
                    },
                }
            }
            InputResult::Program(Input {
                items: items,
                stmts: stmts,
                crates: crates,
            })
        })
        .unwrap_or_else(reterr)
}

#[cfg(feature = "format")]
fn fmt(s: String) -> String {
    crate::fmt::format(&s).unwrap_or(s)
}

#[cfg(not(feature = "format"))]
fn fmt(s: String) -> String {
    s
}

enum ParseItemResult {
    Span(String),
    ExternCrate(String),
    MacroBlock(String, bool),
    Error(String),
}

fn parse_item(item: Item) -> ParseItemResult {
    match &item {
        Item::Static(_) => {
            error!("haven't handled item variant Static");
            ParseItemResult::Error("haven't handled item variant Static. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
        }
        Item::Const(_) => {
            error!("haven't handled item variant Const");
            ParseItemResult::Error("haven't handled item variant Const. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
        }
        Item::Mod(_) => {
            error!("haven't handled item variant Mod");
            ParseItemResult::Error("haven't handled item variant Mod. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
        }
        Item::ForeignMod(_) => {
            error!("haven't handled item variant ForeignMod");
            ParseItemResult::Error("haven't handled item variant ForeignMod. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
        }
        Item::Type(_) => {
            error!("haven't handled item variant Type");
            ParseItemResult::Error("haven't handled item variant Type. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
        }

        Item::Enum(_) => {
            error!("haven't handled item variant Enum");
            ParseItemResult::Error("haven't handled item variant Enum. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
        }
        Item::Union(_) => {
            error!("haven't handled item variant Union");
            ParseItemResult::Error("haven't handled item variant Union. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
        }
        Item::Trait(_) => {
            error!("haven't handled item variant Trait");
            ParseItemResult::Error("haven't handled item variant Trait. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
        }
        Item::TraitAlias(_) => {
            error!("haven't handled item variant TraitAlias");
            ParseItemResult::Error("haven't handled item variant TraitAlias. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
        }
        Item::Macro2(_) => {
            error!("haven't handled item variant Macro2");
            ParseItemResult::Error("haven't handled item variant Macro2. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
        }
        Item::Verbatim(_) => {
            error!("haven't handled item variant Verbatim");
            ParseItemResult::Error("haven't handled item variant Verbatim. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
        }
        Item::ExternCrate(_) => {
            let s = format!("{}", item.into_token_stream());
            debug!("Item parsed, its a crate: {}", s);
            ParseItemResult::ExternCrate(s)
        }
        Item::Macro(m) => {
            let semi = m.semi_token.is_some();
            ParseItemResult::MacroBlock(item.into_token_stream().to_string(), semi)
        }
        _ => ParseItemResult::Span(item.into_token_stream().to_string()),
    }
}

fn parse_expr(expr: Expr) -> Result<String, String> {
    match expr {
		Expr::Box(_) => {
			error!("haven't handled expr variant Box");
			Err("haven't handled expr variant Box. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Array(_) => {
			error!("haven't handled expr variant Array");
			Err("haven't handled expr variant Array. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Unary(_) => {
			error!("haven't handled expr variant Unary");
			Err("haven't handled expr variant Unary. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Lit(_) => {
			error!("haven't handled expr variant Lit");
			Err("haven't handled expr variant Lit. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Cast(_) => {
			error!("haven't handled expr variant Cast");
			Err("haven't handled expr variant Cast. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Type(_) => {
			error!("haven't handled expr variant Type");
			Err("haven't handled expr variant Type. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::If(_) => {
			error!("haven't handled expr variant If");
			Err("haven't handled expr variant If. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::While(_) => {
			error!("haven't handled expr variant While");
			Err("haven't handled expr variant While. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Loop(_) => {
			error!("haven't handled expr variant For");
			Err("haven't handled expr variant For. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Match(_) => {
			error!("haven't handled expr variant Match");
			Err("haven't handled expr variant Match. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Closure(_) => {
			error!("haven't handled expr variant Closure");
			Err("haven't handled expr variant Closure. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Unsafe(_) => {
			error!("haven't handled expr variant Unsafe");
			Err("haven't handled expr variant Unsafe. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Block(_) => {
			error!("haven't handled expr variant Block");
			Err("haven't handled expr variant Block. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Assign(_) => {
			error!("haven't handled expr variant Assign");
			Err("haven't handled expr variant Assign. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::AssignOp(_) => {
			error!("haven't handled expr variant AssignOp");
			Err("haven't handled expr variant AssignOp. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Field(_) => {
			error!("haven't handled expr variant Field");
			Err("haven't handled expr variant Field. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Index(_) => {
			Err("haven't handled expr variant Index. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Range(_) => {
			error!("haven't handled expr variant Range");
			Err("haven't handled expr variant Range. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Reference(_) => {
			error!("haven't handled expr variant Reference");
			Err("haven't handled expr variant Reference. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Break(_) => {
			error!("haven't handled expr variant Break");
			Err("haven't handled expr variant Break. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Continue(_) => {
			error!("haven't handled expr variant Continue");
			Err("haven't handled expr variant Continue. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Return(_) => {
			error!("haven't handled expr variant Return");
			Err("haven't handled expr variant Return. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Struct(_) => {
			error!("haven't handled expr variant Struct");
			Err("haven't handled expr variant Struct. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Repeat(_) => {
			error!("haven't handled expr variant Repeat");
			Err("haven't handled expr variant Repeat. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Paren(_) => {
			error!("haven't handled expr variant Paren");
			Err("haven't handled expr variant Paren. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Group(_) => {
			error!("haven't handled expr variant Group");
			Err("haven't handled expr variant Group. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Try(_) => {
			error!("haven't handled expr variant Try");
			Err("haven't handled expr variant Try. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Async(_) => {
			error!("haven't handled expr variant Async");
			Err("haven't handled expr variant Async. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::TryBlock(_) => {
			error!("haven't handled expr variant TryBlock");
			Err("haven't handled expr variant TryBlock. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Yield(_) => {
			error!("haven't handled expr variant Yield");
			Err("haven't handled expr variant Yield. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Verbatim(_) => {
			error!("haven't handled expr variant Verbatim");
			Err("haven't handled expr variant Verbatim. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		_ => {
			let s = format!("{}", expr.into_token_stream());
			debug!("Expression parsed: {:?}", s);
			Ok(s)
		}
	}
}
