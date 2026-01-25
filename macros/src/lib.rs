//! Procedural macros for nirify
//!
//! This crate provides derive macros to reduce boilerplate code.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

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

/// Derive macro for generating `has_any()` method on structs with Option fields.
///
/// This macro generates a `has_any(&self) -> bool` method that returns true
/// if any of the struct's Option fields are Some.
///
/// # Requirements
///
/// - The struct must have named fields
/// - All fields should be `Option<T>` (non-Option fields are ignored)
///
/// # Example
///
/// ```ignore
/// use nirify_macros::HasAny;
///
/// #[derive(HasAny, Default)]
/// pub struct LayoutOverride {
///     pub gaps: Option<f32>,
///     pub strut_left: Option<f32>,
///     pub center_focused: Option<bool>,
/// }
///
/// // Generates:
/// // impl LayoutOverride {
/// //     pub fn has_any(&self) -> bool {
/// //         self.gaps.is_some()
/// //             || self.strut_left.is_some()
/// //             || self.center_focused.is_some()
/// //     }
/// // }
/// ```
#[proc_macro_derive(HasAny)]
pub fn derive_has_any(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Only works on structs with named fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "HasAny can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "HasAny can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    // Collect Option fields
    let option_checks: Vec<_> = fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.as_ref()?;

            // Check if the field type is Option<T>
            if is_option_type(&field.ty) {
                Some(quote! { self.#field_name.is_some() })
            } else {
                None
            }
        })
        .collect();

    if option_checks.is_empty() {
        return syn::Error::new_spanned(&input, "HasAny requires at least one Option field")
            .to_compile_error()
            .into();
    }

    // Build the has_any method
    let expanded = quote! {
        impl #name {
            /// Returns true if any optional field is set (not None).
            #[inline]
            pub fn has_any(&self) -> bool {
                #(#option_checks)||*
            }
        }
    };

    TokenStream::from(expanded)
}

/// Check if a type is Option<T>
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}
