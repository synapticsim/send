#![feature(proc_macro_diagnostic)]

mod actor;

use proc_macro::{Diagnostic, Level};
use proc_macro2::TokenStream;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput};

#[proc_macro_derive(Actor)]
pub fn actor(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let data = parse_macro_input!(input as DeriveInput);
	match data.data {
		Data::Struct(s) => actor::actor_struct(data.ident, s),
		Data::Enum(e) => actor::actor_enum(data.ident, e),
		Data::Union(_) => {
			Diagnostic::spanned(
				data.span().unwrap(),
				Level::Error,
				"Actor cannot be derived on `union`s. Use `enum`s please",
			)
			.emit();
			TokenStream::new()
		},
	}
	.into()
}
