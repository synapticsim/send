#![feature(proc_macro_diagnostic)]

use proc_macro::{Diagnostic, Level};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(Actor)]
pub fn actor(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let data = parse_macro_input!(input as DeriveInput);
	match data.data {
		Data::Struct(s) => actor_struct(data.ident, s),
		Data::Enum(e) => actor_enum(data.ident, e),
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

fn actor_struct(name: Ident, s: DataStruct) -> TokenStream {
	let subfields = match s.fields {
		Fields::Named(fields) => fields
			.named
			.into_iter()
			.map(|field| {
				let ident = field.ident.unwrap();
				quote! {
					self.#ident.accept(visitor);
				}
			})
			.collect(),
		Fields::Unnamed(fields) => (0..fields.unnamed.len())
			.map(|field| {
				quote! {
					self.#field.accept(visitor);
				}
			})
			.collect(),
		_ => Vec::new(),
	};

	quote! {
		impl send::Actor for #name {
			fn accept<T, R>(&mut self, visitor: &mut impl send::ActorVisitor<T, R>) {
				visitor.visit(self);

				#(#subfields)*
			}
		}
	}
}

fn actor_enum(name: Ident, e: DataEnum) -> TokenStream {
	let variants: Vec<_> = e
		.variants
		.into_iter()
		.map(|variant| {
			let ident = variant.ident;
			match variant.fields {
				Fields::Named(fields) => {
					let names = fields.named.into_iter().map(|field| field.ident.unwrap());
					let names_2 = names.clone();
					quote! {
						#name::#ident { #(#names,)* } => {
							#(#names_2.accept(visitor);)*
						}
					}
				},
				Fields::Unnamed(fields) => {
					let names = fields
						.unnamed
						.into_iter()
						.enumerate()
						.map(|field| "_".to_string() + &field.0.to_string());
					let names_2 = names.clone();
					quote! {
						#name::#ident(#(#names,)*) => {
							#(#names_2.accept(visitor);)*
						}
					}
				},
				Fields::Unit => quote! {
					#name::#ident => {},
				},
			}
		})
		.collect();

	quote! {
		impl send::Actor for #name {
			fn accept<T, R>(&mut self, visitor: &mut impl send::ActorVisitor<T, R>) {
				match self {
					#(#variants)*
				}
			}
		}
	}
}
