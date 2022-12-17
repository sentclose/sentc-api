#![allow(clippy::explicit_counter_loop)]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Fields, Ident, Type, TypePath, TypeReference};

#[proc_macro_derive(MariaDb)]
pub fn maria_db_impl(input: TokenStream) -> TokenStream
{
	let (struct_name, fields) = get_struct_properties(input);

	let mut impl_inside = Vec::with_capacity(fields.len());

	let mut i: usize = 0;

	//collect the properties of the struct to show them in the return Self block
	for field in fields {
		let field_ident = field.0;
		let field_type = field.1;

		impl_inside.push(quote! {
			#field_ident: server_core::take_or_err!(row, #i, #field_type),
		});

		i += 1;
	}

	//display the properties of the loop in the Self return block
	let expand = quote! {
		impl mysql_async::prelude::FromRow for #struct_name
		{
			fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
			where
				Self: Sized,
			{
				Ok(Self {
					#(#impl_inside) *
				})
			}
		}
	};

	expand.into()
}

#[proc_macro_derive(Sqlite)]
pub fn sqlite_impl(input: TokenStream) -> TokenStream
{
	let (struct_name, fields) = get_struct_properties(input);

	let mut impl_inside = Vec::with_capacity(fields.len());

	let mut i: usize = 0;

	for field in fields {
		let field_ident = field.0;
		let field_type = field.1;
		let real_type = get_real_type(&field_type);

		//use for sqlite different macros for u128 and usize because they are parsed from string
		if is_u128(&real_type) {
			impl_inside.push(quote! {
				#field_ident: server_core::take_or_err_u128!(row, #i),
			});
		} else if is_usize(&real_type) {
			impl_inside.push(quote! {
				#field_ident: server_core::take_or_err_usize!(row, #i),
			});
		} else {
			impl_inside.push(quote! {
				#field_ident: server_core::take_or_err!(row, #i),
			});
		}

		i += 1;
	}

	let expand = quote! {
		impl server_core::db::FromSqliteRow for #struct_name
		{
			fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
			where
				Self: Sized,
			{
				Ok(Self {
					#(#impl_inside) *
				})
			}
		}
	};

	expand.into()
}

fn get_struct_properties(input: TokenStream) -> (Ident, Vec<(Ident, Type)>)
{
	let ast: syn::DeriveInput = syn::parse(input).unwrap();

	let struct_name = ast.ident.clone();

	let fields = match ast.data {
		syn::Data::Struct(syn::DataStruct {
			fields, ..
		}) => {
			match fields {
				Fields::Named(fields) => {
					fields
						.named
						.iter()
						.map(|f| (f.ident.clone().unwrap(), f.ty.clone()))
						.collect()
				},
				Fields::Unnamed(_) => panic!("Tuple structs are not supported"),
				Fields::Unit => Vec::new(),
			}
		},
		_ => panic!("Only structs are supported"),
	};

	(struct_name, fields)
}

fn get_real_type(ty: &Type) -> Type
{
	let tokens = ty.to_token_stream();
	syn::parse(tokens.into()).unwrap()
}

fn get_real_type_str(real_type: &Type) -> String
{
	let rust_type = match real_type {
		Type::Path(TypePath {
			path, ..
		}) => path.segments[0].ident.clone(),
		Type::Reference(TypeReference {
			elem, ..
		}) => {
			match elem.as_ref() {
				Type::Path(TypePath {
					path, ..
				}) => path.segments[0].ident.clone(),
				_ => unreachable!(),
			}
		},
		_ => unreachable!(),
	};

	rust_type.to_string()
}

fn is_u128(real_type: &Type) -> bool
{
	let rust_type = get_real_type_str(real_type);

	rust_type.starts_with("u128")
}

fn is_usize(real_type: &Type) -> bool
{
	let rust_type = get_real_type_str(real_type);

	rust_type.starts_with("usize")
}
