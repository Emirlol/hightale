use quote::{
	quote,
	ToTokens,
};
use syn::{
	braced,
	bracketed,
	parse::{
		Parse,
		ParseStream,
	},
	parse_quote,
	spanned::Spanned,
	Attribute,
	Expr,
	Ident,
	ItemStruct,
	Token,
	Type,
};

mod kw {
	syn::custom_keyword!(required);
	syn::custom_keyword!(optional);
	syn::custom_keyword!(req);
	syn::custom_keyword!(opt);
	syn::custom_keyword!(fixed);
	syn::custom_keyword!(variable);
	syn::custom_keyword!(pad);
	syn::custom_keyword!(mask_size);
}

mod paths {
	use syn::Path;

	use super::*;

	// Returns ::crate_name::codec::HytaleCodec
	pub fn hytale_codec() -> Path {
		parse_quote!(crate::codec::HytaleCodec)
	}

	// Returns ::crate_name::codec::FixedSize
	pub fn fixed_size() -> Path {
		parse_quote!(crate::codec::FixedSize)
	}

	// Returns ::crate_name::codec::PacketResult
	pub fn packet_result() -> Path {
		parse_quote!(crate::codec::PacketResult)
	}

	// Returns ::crate_name::codec::PacketError
	pub fn packet_error() -> Path {
		parse_quote!(crate::codec::PacketError)
	}

	// Returns ::crate_name::codec::PacketContext
	pub fn packet_context() -> Path {
		parse_quote!(crate::codec::PacketContext)
	}
}

pub(crate) struct PacketDefinition {
	name: Ident,
	body: PacketBody,
	attributes: Vec<Attribute>,
}

struct SequentialField {
	name: Ident,
	ty: Type,
}

struct MaskedField {
	name: Ident,
	ty: Type,
	kind: MaskedFieldKind,
	explicit_padding: Option<usize>, // Number of padding bits to add after this field
}

#[derive(PartialEq, Eq)]
enum MaskedFieldKind {
	Required, // This isn't necessary for sequential fields, since all fields are required there
	Optional(OptionalType),
}

#[derive(PartialEq, Eq)]
enum OptionalType {
	SingleByteMask(u8),       // The bit within the byte
	MultiByteMask(usize, u8), // The nth byte in the mask, and the bit within that byte
}

impl OptionalType {
	fn get_is_set_quote(&self) -> proc_macro2::TokenStream {
		match self {
			OptionalType::SingleByteMask(bit) => quote! { mask & #bit != 0 },
			OptionalType::MultiByteMask(byte, bit) => quote! { mask[ #byte ] & #bit != 0 },
		}
	}
}

enum PacketBody {
	Unit,
	Sequential(Vec<SequentialField>),
	Masked {
		mask_size: Option<usize>,
		fixed_block: Vec<MaskedField>,
		variable_block: Vec<MaskedField>,
	},
}

impl Parse for SequentialField {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		// name: Type
		let name: Ident = input.parse()?;
		let _colon_token: Token![:] = input.parse()?;
		let ty: Type = input.parse()?;
		Ok(SequentialField { name, ty })
	}
}

impl Parse for MaskedField {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		// All expected inputs:
		// req name Type,
		// opt(u8) name: Type,
		// opt(u8) name: Type [pad=N],
		// opt(usize, u8) name: Type,
		// opt(usize, u8) name: Type [pad=N],
		// Or the above with "required" and "optional" instead of "req" and "opt", but expanding those in this comment is unnecessary.
		let kind = if input.peek(kw::req) || input.peek(kw::required) {
			let _: Ident = input.parse()?;
			MaskedFieldKind::Required
		} else if input.peek(kw::opt) || input.peek(kw::optional) {
			let _: Ident = input.parse()?;
			let content;
			syn::parenthesized!(content in input);
			let first_lit: syn::LitInt = content.parse()?;
			// Check if there are 1 or 2 literals
			if content.peek(Token![,]) {
				let _comma_token: Token![,] = content.parse()?;
				let second_lit: syn::LitInt = content.parse()?;
				MaskedFieldKind::Optional(OptionalType::MultiByteMask(first_lit.base10_parse::<usize>()?, second_lit.base10_parse::<u8>()?))
			} else {
				MaskedFieldKind::Optional(OptionalType::SingleByteMask(first_lit.base10_parse::<u8>()?))
			}
		} else {
			return Err(syn::Error::new(input.span(), "Expected 'required' or 'optional' keyword for MaskedField"));
		};
		let name: Ident = input.parse()?;
		let _colon_token: Token![:] = input.parse()?;
		let ty: Type = input.parse()?;
		let explicit_padding = {
			let bracket_start = input.fork();
			if bracket_start.peek(syn::token::Bracket) {
				let content;
				bracketed!(content in input);
				if !content.peek(kw::pad) {
					return Err(syn::Error::new(content.span(), "Expected 'pad' keyword in padding specification"));
				}
				let _pad_ident = content.parse::<kw::pad>()?;
				let _eq_token: Token![=] = content.parse()?;
				let pad_lit: syn::LitInt = content.parse()?;
				let pad_bits = pad_lit.base10_parse::<usize>()?;
				Some(pad_bits)
			} else {
				None
			}
		};
		Ok(MaskedField { name, ty, kind, explicit_padding })
	}
}

impl MaskedField {
	fn get_padding_quote(&self) -> Expr {
		match self.explicit_padding {
			Some(pad_bits) => parse_quote!( #pad_bits ),
			None => {
				let fixed_size = paths::fixed_size();
				let ty = &self.ty;
				parse_quote!( <#ty as #fixed_size>::SIZE )
			}
		}
	}
}

impl Parse for PacketBody {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let is_masked = if input.peek(syn::Ident) {
			let lookahead = input.fork();
			// We have 3 keywords: mask_size, fixed, variable. If any of these 3 matches the next token, we parse it as a Masked body.
			lookahead.peek(kw::mask_size) || lookahead.peek(kw::fixed) || lookahead.peek(kw::variable)
		} else {
			false
		};
		if !is_masked {
			let fields = input.parse_terminated(SequentialField::parse, Token![,])?;
			return Ok(PacketBody::Sequential(fields.into_iter().collect()));
		}
		let mut mask_size = None;
		let mut fixed_block = Vec::new();
		let mut variable_block = Vec::new();

		loop {
			if input.peek(kw::mask_size) {
				let ident = input.parse::<kw::mask_size>()?;
				if mask_size.is_some() {
					return Err(syn::Error::new(ident.span(), "mask_size already defined in Masked PacketBody"));
				}
				let _eq_token: Token![=] = input.parse()?;
				let size_lit: syn::LitInt = input.parse()?;
				let size = size_lit.base10_parse::<usize>()?;
				mask_size = Some(size);
				continue;
			} else if input.peek(kw::fixed) {
				let ident = input.parse::<kw::fixed>()?;
				if !fixed_block.is_empty() {
					return Err(syn::Error::new(ident.span(), "fixed block already defined in Masked PacketBody"));
				}
				let content;
				braced!(content in input);
				fixed_block.extend(content.parse_terminated(MaskedField::parse, Token![,])?);
				continue;
			} else if input.peek(kw::variable) {
				let ident = input.parse::<kw::variable>()?;
				if !variable_block.is_empty() {
					return Err(syn::Error::new(ident.span(), "variable block already defined in Masked PacketBody"));
				}
				let content;
				braced!(content in input);
				variable_block.extend(content.parse_terminated(MaskedField::parse, Token![,])?);
				continue;
			} else {
				break; // This falls into the `is_empty` checks below
			}
		}

		if fixed_block.is_empty() && variable_block.is_empty() {
			return Err(syn::Error::new(input.span(), "Expected at least one of 'fixed' or 'variable' blocks in Masked PacketBody"));
		}

		Ok(PacketBody::Masked {
			mask_size,
			fixed_block,
			variable_block,
		})
	}
}

impl Parse for PacketDefinition {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		// #[...] attributes
		// Name { ... }
		let attributes = if input.peek(Token![#]) { input.call(Attribute::parse_outer)? } else { Vec::new() };
		let name: Ident = input.parse()?;
		let body: PacketBody = if input.peek(syn::token::Brace) {
			let content;
			braced!(content in input);
			if content.is_empty() {
				// Allow both `PacketName {}` and `PacketName`
				PacketBody::Unit
			} else {
				content.parse()?
			}
		} else {
			PacketBody::Unit
		};

		Ok(PacketDefinition { name, body, attributes })
	}
}

impl ToTokens for SequentialField {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let name = &self.name;
		let ty = &self.ty;
		tokens.extend(quote! {
			#name: #ty
		});
	}
}

impl ToTokens for MaskedField {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let name = &self.name;

		match self.kind {
			MaskedFieldKind::Required => {
				let ty = &self.ty;
				tokens.extend(quote! {
					#name: #ty
				});
			}
			MaskedFieldKind::Optional(_) => {
				let ty = &self.ty;
				tokens.extend(quote! {
					#name: Option<#ty>
				});
			}
		}
	}
}

impl PacketDefinition {
	#[allow(clippy::wrong_self_convention)]
	pub(crate) fn to_tokens(self) -> proc_macro2::TokenStream {
		let hytale_codec = paths::hytale_codec();
		let fixed_size = paths::fixed_size();
		let packet_context = paths::packet_context();
		let packet_result = paths::packet_result();
		let packet_error = paths::packet_error();
		let name = self.name;
		let attributes = self.attributes;

		match self.body {
			PacketBody::Unit => {
				quote! {
					#(#attributes)*
					#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, protocol_macros::FixedSize)]
					pub struct #name;

					impl #hytale_codec for #name {
						fn encode(&self, _buf: &mut bytes::BytesMut) -> #packet_result<()> {
							Ok(())
						}

						fn decode(_buf: &mut impl bytes::Buf) -> #packet_result<Self> {
							Ok(Self)
						}
					}
				}
			}
			PacketBody::Sequential(fields) => {
				let types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
				let names: Vec<_> = fields.iter().map(|f| &f.name).collect();
				let name_strs: Vec<_> = fields.iter().map(|f| f.name.to_string()).collect();
				quote! {
					#[derive(Debug, Clone, protocol_macros::FixedSize)]
					#(#attributes)*
					pub struct #name {
						#(
							pub #names: #types,
						)*
					}

					impl #hytale_codec for #name {
						fn encode(&self, buf: &mut bytes::BytesMut) -> #packet_result<()> {
							use #packet_context;
							#(
								<#types as #hytale_codec>::encode(&self.#names, buf).context(#name_strs)?;
							)*
							Ok(())
						}

						fn decode(buf: &mut impl bytes::Buf) -> #packet_result<Self> {
							use #packet_context;
							let remaining = buf.remaining();
							if remaining < <Self as #fixed_size>::SIZE {
								Err(#packet_error::incomplete_bytes_exact(remaining, <Self as #fixed_size>::SIZE))
							} else {
								Ok(Self {
									#(
										#names: <#types as #hytale_codec>::decode(buf).context(#name_strs)?,
									)*
								})
							}
						}
					}
				}
			}
			PacketBody::Masked {
				mask_size,
				fixed_block,
				variable_block,
			} => {
				let is_mask_required = fixed_block.iter().chain(variable_block.iter()).any(|f| f.kind != MaskedFieldKind::Required);

				let calculated_mask_size = {
					// Find the highest byte index used in MultiByteMask optional fields
					let mut max_byte = 0usize;
					for field in fixed_block.iter().chain(variable_block.iter()) {
						if let MaskedFieldKind::Optional(opt_type) = &field.kind
							&& let OptionalType::MultiByteMask(byte, _) = opt_type
							&& *byte > max_byte
						{
							max_byte = *byte;
						}
					}
					max_byte + 1 // +1 because byte index starts at 0
				};

				if !is_mask_required && mask_size.is_some() {
					return syn::Error::new(name.span(), "mask_size specified but no optional fields present; mask is not required").to_compile_error();
				}

				let final_mask_size = if let Some(explicit_size) = mask_size {
					if explicit_size < calculated_mask_size {
						return syn::Error::new(
							name.span(),
							format!(
								"Explicit mask_size {} is too small; minimum required is {} based on field definitions",
								explicit_size, calculated_mask_size
							),
						)
						.to_compile_error();
					} else {
						explicit_size // Use the explicitly defined size if it's bigger, so the user can have extra unused bits if they desire
					}
				} else if is_mask_required {
					calculated_mask_size
				} else {
					0
				};

				let struct_def: ItemStruct = parse_quote!(
					#(#attributes)*
					#[derive(Debug, Clone)]
					pub struct #name {
						#(pub #fixed_block,)*
						#(pub #variable_block,)*
					}
				);

				let size_iter = fixed_block.iter().map(|f| {
					let ty = &f.ty;
					match f.kind {
						MaskedFieldKind::Required => {
							parse_quote! { <#ty as #fixed_size>::SIZE }
						}
						MaskedFieldKind::Optional(_) => f.get_padding_quote(),
					}
				});

				let impl_fixed_size = if variable_block.is_empty() {
					quote! {
						impl #fixed_size for #name {
							const SIZE: usize = Self::FIXED_BLOCK_SIZE + Self::MASK_SIZE;
						}
					}
				} else {
					quote! {}
				};

				let offsets_size = if variable_block.len() <= 1 {
					0usize
				} else {
					variable_block.len() * 4 // 4 bytes per offset
				};

				let impl_block = quote!(
					impl #name {
						const MASK_SIZE: usize = #final_mask_size;
						const FIXED_BLOCK_SIZE: usize = 0usize #( + #size_iter )* + #offsets_size;
					}

					#impl_fixed_size
				);

				// Cheese the mask definition based on size. The compiler most likely does this anyway but whatever.
				let mask_def = if final_mask_size == 0 {
					quote! {}
				} else if final_mask_size == 1 {
					quote! { let mut mask: u8 = 0; }
				} else {
					quote! { let mut mask: [u8; Self::MASK_SIZE] = [0u8; Self::MASK_SIZE]; }
				};

				let mask_read = if final_mask_size == 0 {
					quote! {}
				} else if final_mask_size == 1 {
					quote! { let mask: u8 = buf.get_u8(); }
				} else {
					quote! {
						let mut mask: [u8; Self::MASK_SIZE] = [0u8; Self::MASK_SIZE];
						buf.copy_to_slice(&mut mask);
					}
				};

				let null_bits_initialize: Vec<proc_macro2::TokenStream> = fixed_block
					.iter()
					.chain(variable_block.iter())
					.map(|field| {
						let name = &field.name;
						match field.kind {
							MaskedFieldKind::Optional(OptionalType::MultiByteMask(bytes, bits)) => {
								quote!(
									if self.#name.is_some() {
										mask[ #bytes ] |= #bits;
									}
								)
							}
							MaskedFieldKind::Optional(OptionalType::SingleByteMask(bits)) => {
								quote! {
									if self.#name.is_some() {
										mask |= #bits;
									}
								}
							}
							_ => quote! {},
						}
					})
					.collect();

				let null_bits_write = if final_mask_size == 0 {
					quote! {}
				} else if final_mask_size == 1 {
					quote! { buf.put_u8(mask); }
				} else {
					quote! { buf.put_slice(&mask); }
				};

				let fixed_block_name: Vec<&Ident> = fixed_block.iter().map(|f| &f.name).collect();
				let variable_block_name: Vec<&Ident> = variable_block.iter().map(|f| &f.name).collect();

				let decode_helper = |ty: &Type, name_str: String| quote! { <#ty as #hytale_codec>::decode(buf).context(#name_str)? };
				let encode_helper = |ty: &Type, name: &Ident| {
					let name_str = name.to_string();
					quote! { <#ty as #hytale_codec>::encode(&self.#name, buf).context(#name_str)?; }
				};

				let fixed_encode = fixed_block.iter().map(|field| {
					let name = &field.name;
					let name_str = name.to_string();
					let ty = &field.ty;
					match field.kind {
						MaskedFieldKind::Required => encode_helper(ty, name),
						MaskedFieldKind::Optional(_) => {
							let padding = field.get_padding_quote();
							quote! {
								if let Some(value) = &self.#name {
									<#ty as #hytale_codec>::encode(value, buf).context(#name_str)?;
								} else {
									buf.extend_from_slice(&[0u8; #padding]);
								}
							}
						}
					}
				});

				let fixed_decode = fixed_block.iter().map(|field| {
					let name = &field.name;
					let name_str = name.to_string();
					let ty = &field.ty;
					let decode = decode_helper(ty, name_str.clone());
					match &field.kind {
						MaskedFieldKind::Required => decode,
						MaskedFieldKind::Optional(optional_type) => {
							let is_set = optional_type.get_is_set_quote();
							let padding = field.get_padding_quote();
							quote! {
								{
									let is_set = #is_set;
									let start = buf.remaining();
									let val = if is_set {
										Some(#decode)
									} else {
										None
									};

									let consumed = start - buf.remaining();
									if consumed > #padding { return Err(#packet_error::decoded_more_than_padding(consumed, #padding)).context(#name_str); }
									let padding = #padding - consumed;
									let remaining = buf.remaining();
									if remaining < padding { return Err(#packet_error::incomplete_bytes(remaining, padding)).context(#name_str); }
									buf.advance(padding);

									val
								}
							}
						}
					}
				});

				let variable_decode = if variable_block.len() == 1 {
					let field = variable_block.first().unwrap();
					let name = &field.name;
					let name_str = name.to_string();
					let ty = &field.ty;
					let decode = decode_helper(ty, name_str.clone());
					let val = match &field.kind {
						MaskedFieldKind::Required => quote! {
							#decode
						},
						MaskedFieldKind::Optional(optional_type) => {
							let is_set = optional_type.get_is_set_quote();
							quote! {
								if #is_set {
									Some(#decode)
								} else {
									None
								}
							}
						}
					};
					quote! {
						let #name = #val;
					}
				} else {
					let variable_decode = variable_block.iter().map(|field| {
						let name = &field.name;
						let name_str = name.to_string();
						let ty = &field.ty;
						let decode = decode_helper(ty, name_str.clone());
						let val = match &field.kind {
							MaskedFieldKind::Required => decode,
							MaskedFieldKind::Optional(_) => quote! { Some(#decode) },
						};
						let main_body = quote! {
							let target_offset = offsets[var_idx];
							if target_offset < var_read_bytes {
								return Err(#packet_error::incomplete_bytes(target_offset, var_read_bytes)).context(#name_str);
							}

							let skip = (target_offset - var_read_bytes) as usize;
							let remaining = buf.remaining();
							if remaining < skip { return Err(#packet_error::incomplete_bytes(remaining, skip)).context(#name_str); }
							buf.advance(skip);
							var_read_bytes += skip as u32;

							let start = buf.remaining();
							let val = #val;
							var_read_bytes += (start - buf.remaining()) as u32;
							val
						};
						match &field.kind {
							MaskedFieldKind::Required => quote! { { #main_body } }, // Has to be wrapped in a block to be an expression
							MaskedFieldKind::Optional(optional_type) => {
								let is_set = optional_type.get_is_set_quote();
								quote! {
									if #is_set {
										#main_body
									} else {
										None
									}
								}
							}
						}
					});
					quote! {
						#(
							let #variable_block_name = #variable_decode;
							var_idx += 1;
						)*
					}
				};

				let variable_encode = if variable_block.len() == 1 {
					let field = variable_block.first().unwrap();
					let name = &field.name;
					let name_str = name.to_string();
					let ty = &field.ty;
					let encode = match field.kind {
						MaskedFieldKind::Required => encode_helper(ty, name),
						MaskedFieldKind::Optional(_) => quote! {
							if let Some(value) = &self.#name {
								<#ty as #hytale_codec>::encode(value, buf).context(#name_str)?;
							} // No padding
						},
					};

					quote! { #encode }
				} else {
					let variable_encode = variable_block.iter().map(|field| {
						let name = &field.name;
						let name_str = name.to_string();
						let ty = &field.ty;
						let encode = match field.kind {
							MaskedFieldKind::Required => encode_helper(ty, name),
							MaskedFieldKind::Optional(_) => quote! {
								if let Some(value) = &self.#name {
									<#ty as #hytale_codec>::encode(value, buf).context(#name_str)?;
								}
							},
						};

						quote! {
							let current_pos = buf.len();
							let offset_val = (current_pos - var_block_start) as u32;
							let patch_pos = offsets[var_idx] as usize;
							buf[patch_pos..patch_pos+4].copy_from_slice(&offset_val.to_le_bytes());
							#encode

							var_idx += 1;
						}
					});

					quote! {
						#(#variable_encode)*
					}
				};

				// The variable block works with an offset table. First initialized to all 0s, then filled in with the correct offsets after encoding the variable-length fields.
				let encode_offset_initialize = {
					let len = variable_block.len();
					if len == 0 || len == 1 {
						// 0 is trivial, 1 has no offset table so it ends up empty as well
						quote! {}
					} else {
						// Create offset placeholders
						let offsets_push = (0..len).map(|_| {
							quote! {
								offsets.push(buf.len() as u32);
								buf.put_i32_le(0);
							}
						});
						quote! {
							let mut offsets: Vec<u32> = vec![];
							#(#offsets_push)*

							let var_block_start = buf.len();
							let mut var_idx = 0;
						}
					}
				};

				let decode_offset_initialize = {
					let len = variable_block.len();
					if len == 0 || len == 1 {
						// 0 is trivial, 1 has no offset table so it ends up empty as well
						quote! {}
					} else {
						let offsets_read = (0..len).map(|_| {
							quote! {
								offsets.push(buf.get_u32_le());
							}
						});
						quote! {
							let mut offsets: Vec<u32> = vec![];
							#(#offsets_read)*

							let mut var_read_bytes = 0u32;
							let mut var_idx = 0;
						}
					}
				};

				quote! {
					#struct_def
					#impl_block

					impl #hytale_codec for #name {
						fn encode(&self, buf: &mut bytes::BytesMut) -> #packet_result<()> {
							use bytes::BufMut;
							use #packet_context;

							#mask_def
							#(#null_bits_initialize)*
							#null_bits_write

							// Encode fixed block
							#(#fixed_encode)*

							#encode_offset_initialize
							// Encode variable block
							#variable_encode
							Ok(())
						}

						fn decode(buf: &mut impl bytes::Buf) -> #packet_result<Self> {
							use bytes::Buf;
							use #packet_context;

							let remaining = buf.remaining();
							if remaining < Self::FIXED_BLOCK_SIZE + Self::MASK_SIZE {
								return Err(#packet_error::incomplete_bytes(remaining, Self::FIXED_BLOCK_SIZE + Self::MASK_SIZE));
							}

							#mask_read

							#(let #fixed_block_name = #fixed_decode;)*

							#decode_offset_initialize

							#variable_decode

							Ok(Self {
								#(#fixed_block_name,)*
								#(#variable_block_name,)*
							})
						}
					}
				}
			}
		}
	}
}
