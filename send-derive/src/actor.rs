use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DataEnum, DataStruct, Fields};

pub fn actor_struct(name: Ident, s: DataStruct) -> TokenStream {
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
			#[inline]
			fn accept<T, R>(&mut self, visitor: &mut impl send::ActorVisitor<T, R>) {
				visitor.visit(self);

				#(#subfields)*
			}
		}
	}
}

pub fn actor_enum(name: Ident, e: DataEnum) -> TokenStream {
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
			#[inline]
			fn accept<T, R>(&mut self, visitor: &mut impl send::ActorVisitor<T, R>) {
				match self {
					#(#variants)*
				}
			}
		}
	}
}
