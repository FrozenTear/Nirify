//! Procedural macros for niri-settings
//!
//! This crate provides derive macros to reduce boilerplate code.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derive macro for generating Slint UI index conversion methods.
///
/// This macro generates `to_index(&self) -> i32` and `from_index(idx: i32) -> Self`
/// methods for enums used in Slint combobox bindings.
///
/// # Attributes
///
/// - `#[slint_index(N)]` - Specify the index for a variant (optional, defaults to order)
/// - `#[slint_index(default)]` - Mark this variant as the default for unknown indices
///
/// If no indices are specified, variants are numbered starting from 0 in declaration order.
/// If no default is specified, the first variant is used as the default.
///
/// # Example
///
/// ```ignore
/// use niri_settings_macros::SlintIndex;
///
/// #[derive(SlintIndex)]
/// pub enum AccelProfile {
///     #[slint_index(default)]  // Index 0 (first), also default
///     Adaptive,
///     Flat,                     // Index 1
/// }
///
/// // Generates:
/// // impl AccelProfile {
/// //     pub fn to_index(&self) -> i32 { ... }
/// //     pub fn from_index(idx: i32) -> Self { ... }
/// // }
/// ```
#[proc_macro_derive(SlintIndex, attributes(slint_index))]
pub fn derive_slint_index(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Only works on enums
    let data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return syn::Error::new_spanned(&input, "SlintIndex can only be derived for enums")
                .to_compile_error()
                .into();
        }
    };

    // Collect variant info: (variant_ident, index, is_default)
    let mut variants_info: Vec<(&syn::Ident, i32, bool)> = Vec::new();
    let mut explicit_default: Option<usize> = None;

    for (i, variant) in data.variants.iter().enumerate() {
        // Check that variant has no fields (unit variant)
        if !matches!(variant.fields, Fields::Unit) {
            return syn::Error::new_spanned(
                variant,
                "SlintIndex only supports unit variants (no fields)",
            )
            .to_compile_error()
            .into();
        }

        let mut index = i as i32;
        let mut is_default = false;

        // Parse attributes using syn 2.0 API
        for attr in &variant.attrs {
            if attr.path().is_ident("slint_index") {
                // Parse the attribute contents
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("default") {
                        is_default = true;
                        explicit_default = Some(variants_info.len());
                    } else if let Ok(value) = meta.value() {
                        // Parse as integer literal
                        let lit: syn::LitInt = value.parse()?;
                        index = lit.base10_parse()?;
                    } else {
                        // Try parsing as just an integer (e.g., #[slint_index(0)])
                        let lit: syn::LitInt = meta
                            .path
                            .get_ident()
                            .and_then(|id| id.to_string().parse::<i32>().ok())
                            .map(|n| {
                                syn::LitInt::new(&n.to_string(), proc_macro2::Span::call_site())
                            })
                            .ok_or_else(|| meta.error("expected integer or 'default'"))?;
                        index = lit.base10_parse()?;
                    }
                    Ok(())
                });
            }
        }

        variants_info.push((&variant.ident, index, is_default));
    }

    // If no explicit default, first variant is default
    let default_idx = explicit_default.unwrap_or(0);
    if !variants_info.is_empty() && !variants_info[default_idx].2 {
        // Mark the first one as default if none was explicitly marked
        variants_info[default_idx].2 = true;
    }

    // Generate to_index match arms
    let to_index_arms = variants_info.iter().map(|(ident, index, _)| {
        quote! {
            Self::#ident => #index
        }
    });

    // Generate from_index match arms (excluding default, which goes in _ arm)
    let from_index_arms = variants_info
        .iter()
        .filter_map(|(ident, index, is_default)| {
            if *is_default {
                None
            } else {
                Some(quote! {
                    #index => Self::#ident
                })
            }
        });

    // Get the default variant
    let default_variant = variants_info
        .iter()
        .find(|(_, _, is_default)| *is_default)
        .map(|(ident, _, _)| *ident)
        .unwrap_or(variants_info[0].0);

    // Generate the impl block
    let expanded = quote! {
        impl #name {
            /// Convert this enum variant to its Slint UI combobox index.
            #[inline]
            pub fn to_index(&self) -> i32 {
                match self {
                    #(#to_index_arms),*
                }
            }

            /// Convert a Slint UI combobox index to this enum.
            ///
            /// Unknown indices return the default variant.
            #[inline]
            pub fn from_index(idx: i32) -> Self {
                match idx {
                    #(#from_index_arms,)*
                    _ => Self::#default_variant
                }
            }
        }
    };

    TokenStream::from(expanded)
}
