use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DataEnum, DataStruct, Fields, Generics};

pub fn actor_struct(name: Ident, s: DataStruct, generics: Generics) -> TokenStream {
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
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
		unsafe impl #impl_generics send::Actor for #name #ty_generics #where_clause {
			#[inline]
			fn accept<T, R>(&mut self, visitor: &mut impl send::ActorVisitor<T, R>) {
				#(#subfields)*
								
				visitor.visit(self);
			}
		}

		impl #impl_generics !send::NotActor for #name #ty_generics #where_clause {}
	}
}

pub fn actor_enum(name: Ident, e: DataEnum, generics: Generics) -> TokenStream {
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
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
		unsafe impl #impl_generics send::Actor for #name #ty_generics #where_clause {
			#[inline]
			fn accept<T, R>(&mut self, visitor: &mut impl send::ActorVisitor<T, R>) {
				match self {
					#(#variants)*
				}
			}
		}

		impl #impl_generics !send::NotActor for #name #ty_generics #where_clause {}
	}
}
