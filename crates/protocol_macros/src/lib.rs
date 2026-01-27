use proc_macro::TokenStream;
use quote::{
	quote,
	quote_spanned,
};
use syn::{
	parse_macro_input,
	spanned::Spanned,
	Data,
	DeriveInput,
	Error,
	Fields,
};

mod define_packet;

#[proc_macro_derive(FixedSize)]
pub fn derive_fixed_size(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let fixed_size = quote! { crate::codec::FixedSize };

	let size_expr = match input.data {
		Data::Struct(data) => match data.fields {
			Fields::Named(fields) => {
				let sums = fields.named.iter().map(|f| {
					let ty = &f.ty;
					quote_spanned! {f.span()=> <#ty as #fixed_size>::SIZE }
				});
				quote! { 0 #(+ #sums)* }
			}
			Fields::Unnamed(fields) => {
				let sums = fields.unnamed.iter().map(|f| {
					let ty = &f.ty;
					quote_spanned! {f.span()=> <#ty as #fixed_size>::SIZE }
				});
				quote! { 0 #(+ #sums)* }
			}
			Fields::Unit => quote! { 0 },
		},
		Data::Enum(_) => {
			let mut repr_type = None;

			for attr in &input.attrs {
				if attr.path().is_ident("repr") {
					// This handles #[repr(u8)], #[repr(C, u8)], etc.
					let _ = attr.parse_nested_meta(|meta| {
						if let Some(ident) = meta.path.get_ident() {
							let s = ident.to_string();
							match s.as_str() {
								// Match standard integer types
								"u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "u128" | "i128" | "usize" | "isize" => {
									repr_type = Some(ident.clone());
								}
								_ => {} // Ignore "C", "packed", "align", etc.
							}
						}
						Ok(())
					});
				}
			}

			match repr_type {
				Some(ty) => quote! { <#ty as #fixed_size>::SIZE },
				None => {
					return Error::new(name.span(), "Enum must have a #[repr(int_type)] to derive FixedSize").to_compile_error().into();
				}
			}
		}

		Data::Union(_) => {
			return Error::new(name.span(), "FixedSize cannot be derived for Unions").to_compile_error().into();
		}
	};

	// Not needed but why not
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let expanded = quote! {
		impl #impl_generics crate::codec::FixedSize for #name #ty_generics #where_clause {
			const SIZE: usize = #size_expr;
		}
	};

	expanded.into()
}

/// Defines a packet.
/// There are 3 different forms;
/// 1. Empty packet:
/// ```
/// define_packet! { PacketName }
/// ```
/// 2. Packet with only fixed-size and always required fields:
/// ```
/// define_packet! {
///   // All fields are required, and must implement FixedSize. This can also be done with a fixed block wrapping the fields and `required` keyword in front of each field, but this is just more concise.
///   PacketName {
///     field1: Type1,
///     field2: Type2,
///   }
/// }
/// ```
/// 3. Packet with fixed and variable fields:
/// ```
/// define_packet! {
///   PacketName {
///     // All fields in the fixed block must implement FixedSize.
///     fixed {
///       required field1: Type1,
///       required field2: Type2,
///       opt(1) field3: Type3, // This is actually Option<Type3> in the generated struct.
///       opt(4) field4: Type4,
///       opt(8) field5: Type5,
///       opt(16) field6: Type6,
///     }
///     variable {
///       required field4: Type4,
///       opt(2) field5: Type5,
///     }
///   }
/// }
/// ```
///
/// The `opt` format is either `opt(N)` or `opt(B, N)` where:
///  - `N` is the number to check against in the null mask,
///  - `B` is the index of the byte in the byte vec if there's more than 1 byte. This is 0-indexed as arrays are in Rust.
///
/// When there are multiple bytes, the number of bytes needed for the struct is inferred from the highest null byte used in all `opt`s, but can also be manually specified like so:
/// ```
/// define_packet! {
///   PacketName {
///     mask_bytes = 2
///     fixed { ... }
///     variable { ... }
///   }
/// }
/// ```
///
/// Similarly, the padding for each optional field in the required block can be manually specified like so:
/// ```
/// define_packet! {
///   PacketName {
///     fixed {
///       opt(1) field1: Type1 [pad=12],
///       opt(4) field2: Type2 [pad=6],
///     }
///   }
/// }
/// ```
/// The padding is inferred from the FixedSize implementation of the types if not specified. It's usually better to not specify it unless necessary.
#[proc_macro]
pub fn define_packet(input: TokenStream) -> TokenStream {
	parse_macro_input!(input as define_packet::PacketDefinition).to_tokens().into()
}
