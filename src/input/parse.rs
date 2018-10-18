use super::*;
use proc_macro2::{Span, TokenStream};
use std::str::FromStr;
use syn::{self, buffer::TokenBuffer, export::ToTokens, spanned::Spanned, Block, Item, Stmt};

/// Parses a line of input as a program.
pub fn parse_program(code: &str) -> InputResult {
	debug!("parse program: {}", code);

	match TokenStream::from_str(code) {
		Ok(stream) => {
			let buffer = TokenBuffer::new2(stream);
			let cursor = buffer.begin();
		}
		Err(e) => {}
	}

	//return InputResult::InputError("some error".to_string());

	match syn::parse_str::<Block>(&format!("{{ {} }}", code)) {
		Ok(block) => {
			let mut stmts = Vec::new();
			for stmt in block.stmts {
				match stmt {
					Stmt::Local(_) => {
						error!("haven't handled Block variant Local");
					}
					Stmt::Item(_) => {
						error!("haven't handled Block variant Item");
					}
					Stmt::Expr(_) => {
						error!("haven't handled Block variant Expr");
					}
					Stmt::Semi(expr, _) => match parse_expr(expr) {
						Ok(span) => {
							println!("{:?}", span);
							let start = span.start();
							let end = span.end();
							println!("start line {} column {}", start.line, start.column);
							println!("end line {} column {}", end.line, end.column);
							let s = code[..].to_string();
							let s = String::from("adsf");
							stmts.push(Statement {
								expr: s,
								semi: true,
							});
						}
						Err(s) => return InputResult::InputError(s),
					},
				}
			}
			InputResult::InputError("some error".to_string())
		}
		Err(e) => if e.to_string() == "LexError" {
			InputResult::More
		} else {
			InputResult::InputError(e.to_string())
		},
	}
}

fn parse_item(item: Item) -> Result<String, String> {
	match item {
		Item::ExternCrate(_) => {
			error!("haven't handled item variant ExternCrate");
			Err("haven't handled item variant ExternCrate. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Use(_) => {
			error!("haven't handled item variant Use");
			Err("haven't handled item variant Use. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Static(_) => {
			error!("haven't handled item variant Static");
			Err("haven't handled item variant Static. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Const(_) => {
			error!("haven't handled item variant Const");
			Err("haven't handled item variant Const. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Mod(_) => {
			error!("haven't handled item variant Mod");
			Err("haven't handled item variant Mod. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::ForeignMod(_) => {
			error!("haven't handled item variant ForeignMod");
			Err("haven't handled item variant ForeignMod. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Type(_) => {
			error!("haven't handled item variant Type");
			Err("haven't handled item variant Type. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Existential(_) => {
			error!("haven't handled item variant Existential");
			Err("haven't handled item variant Existential. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Struct(_) => {
			error!("haven't handled item variant Struct");
			Err("haven't handled item variant Struct. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Enum(_) => {
			error!("haven't handled item variant Enum");
			Err("haven't handled item variant Enum. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Union(_) => {
			error!("haven't handled item variant Union");
			Err("haven't handled item variant Union. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Trait(_) => {
			error!("haven't handled item variant Trait");
			Err("haven't handled item variant Trait. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::TraitAlias(_) => {
			error!("haven't handled item variant TraitAlias");
			Err("haven't handled item variant TraitAlias. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Impl(_) => {
			error!("haven't handled item variant Impl");
			Err("haven't handled item variant Impl. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Macro(_) => {
			error!("haven't handled item variant Macro");
			Err("haven't handled item variant Macro. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Macro2(_) => {
			error!("haven't handled item variant Macro2");
			Err("haven't handled item variant Macro2. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Verbatim(_) => {
			error!("haven't handled item variant Verbatim");
			Err("haven't handled item variant Verbatim. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		_ => Ok("FUCK".to_string()),
	}
}

fn parse_expr(expr: Expr) -> Result<Span, String> {
	match expr {
		Expr::Box(_) => {
			error!("haven't handled expr variant Box");
			Err("haven't handled expr variant Box. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::InPlace(_) => {
			error!("haven't handled expr variant InPlace");
			Err("haven't handled expr variant InPlace. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Array(_) => {
			error!("haven't handled expr variant Array");
			Err("haven't handled expr variant Array. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::MethodCall(_) => {
			error!("haven't handled expr variant MethodCall");
			Err("haven't handled expr variant MethodCall. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
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
		Expr::ForLoop(_) => {
			error!("haven't handled expr variant ForLoop");
			Err("haven't handled expr variant ForLoop. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
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
		Expr::Path(_) => {
			error!("haven't handled expr variant Path");
			Err("haven't handled expr variant Path. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
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
			let span = expr.span();
let s = format!("{}", expr.into_token_stream());
debug!("Expression parsed: {}", s);
Ok(span)
		}
	}
}
