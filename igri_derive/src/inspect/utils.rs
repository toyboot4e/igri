//! Utilities

use darling::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::*;

use crate::inspect::args;

pub fn imgui_path() -> TokenStream2 {
    quote!(igri::imgui)
}

pub fn inspect_path() -> TokenStream2 {
    quote!(igri::Inspect)
}

/// Code for `#[inspect(as = ..)]` in `inspect` function
pub fn impl_inspect_as(x_ref: TokenStream2, as_: &String) -> TokenStream2 {
    let inspect = inspect_path();

    let as_ = parse_str::<Type>(as_).expect("#[inspect(as = ..)] must refer to a type");
    quote! {
        let mut bridge: #as_ = (*#x_ref).into();
        #inspect::inspect(&mut bridge, ui, label);
        *#x_ref = bridge.into();
    }
}

/// Code for `#[inspect(with = ..)]` in `inspect` function
pub fn impl_inspect_with(x_ref: TokenStream2, with: &String) -> TokenStream2 {
    let with = parse_str::<ExprPath>(with).expect("#[inspect(with = ..)] must refer to a path");
    quote! {
        #with(#x_ref, ui, label)
    }
}

/// `<prefix>.field.inspect(ui, label);`
pub fn field_inspectors<'a>(
    prefix: TokenStream2,
    field_args: &'a ast::Fields<args::FieldArgs>,
) -> impl Iterator<Item = TokenStream2> + 'a {
    let inspect = inspect_path();

    field_args
        .fields
        .iter()
        .filter(|field| !field.skip)
        .enumerate()
        .map(move |(field_index, field)| {
            let (field_ident, label) = match field_args.style {
                ast::Style::Struct => {
                    let field_ident = field.ident.as_ref().unwrap_or_else(|| unreachable!());
                    (quote!(#field_ident), format!("{}", field_ident))
                }
                ast::Style::Tuple => {
                    let field_ident = Index::from(field_index);
                    (quote!(#field_ident), format!("{}", field_index))
                }
                ast::Style::Unit => unreachable!(),
            };

            if let Some(as_) = field.as_.as_ref() {
                // #[inspect(as = "type")]

                let as_ = parse_str::<Type>(as_).unwrap();
                quote! {
                    let mut x: #as_ = (*#prefix.#field_ident).into();
                    #inspect::inspect(&mut x, ui, label);
                    *#prefix.#field_ident = x.into();
                }
            } else if let Some(with) = field.with.as_ref() {
                // #[inspect(with = "function")]

                if let Ok(with) = parse_str::<ExprPath>(with) {
                    quote! {
                        (#with)(&mut #prefix.#field_ident);
                    }
                } else {
                    panic!("invalid #[inspect(with)] argument; be sure to give path");
                }
            } else {
                // inspect the value as-is
                quote! {
                    #prefix.#field_ident.inspect(ui, #label);
                }
            }
        })
}

/// `<prefix>.field.inspect(ui, label);`
pub fn enum_tag_selector<'a>(
    ty_args: &args::TypeArgs,
    ty_variants: &[args::VariantArgs],
) -> TokenStream2 {
    let ty_ident = &ty_args.ident;

    // List of `TypeName::Variant`
    let variant_idents = ty_variants
        .iter()
        .map(|v| format_ident!("{}", v.ident))
        .collect::<Vec<_>>();

    quote! {
        const VARIANTS: &[#ty_ident] = &[#(#ty_ident::#variant_idents,)*];

        fn variant_index(variant: &#ty_ident) -> Option<usize> {
            VARIANTS
                .iter()
                .enumerate()
                .find_map(|(i, v)| if v == variant { Some(i) } else { None })
        }

        const fn variant_name(ix: usize) -> &'static str {
            const NAMES: &'static [&'static str] = &[
                #(
                    stringify!(Self::#variant_idents),
                )*
            ];
            NAMES[ix]
        }

        let mut ix = variant_index(self).unwrap();

        if ui.combo(
            label,
            &mut ix,
            VARIANTS,
            |v| std::borrow::Cow::Borrowed(variant_name(variant_index(v).unwrap())),
        ) {
            *self = VARIANTS[ix].clone();
        }
    }
}

/// Fill the `inspect` function body to derive `Inspect`
pub fn impl_inspect(
    ty_args: &args::TypeArgs,
    generics: Generics,
    inspect_body: TokenStream2,
) -> TokenStream2 {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ty_ident = &ty_args.ident;

    let imgui = imgui_path();
    let inspect = inspect_path();

    return quote! {
        impl #impl_generics #inspect for #ty_ident #ty_generics #where_clause
        {
            fn inspect(&mut self, ui: &#imgui::Ui, label: &str) {
                #inspect_body
            }
        }
    };
}

pub fn struct_inspect_generics(ty_args: &args::TypeArgs) -> Generics {
    let mut generics = ty_args.generics.clone();
    let inspect = inspect_path();

    let clause = generics.make_where_clause();

    if let Some(bounds) = ty_args.bounds.as_ref() {
        // add user's manual boundaries
        if !bounds.is_empty() {
            clause.predicates.extend(
                bounds
                    .split(",")
                    .map(|b| parse_str::<WherePredicate>(b).unwrap()),
            );
        }
    } else {
        // add `Field: Inspect` for each fiel
        clause.predicates.extend(
            ty_args
                .all_fields()
                .iter()
                .filter(|f| !f.skip)
                .map(|f| &f.ty)
                .map::<WherePredicate, _>(|ty| parse_quote! { #ty: #inspect }),
        );
    }

    generics
}

pub fn enum_inspect_generics(ty_args: &args::TypeArgs) -> Generics {
    let mut generics = ty_args.generics.clone();
    let inspect = inspect_path();

    let clause = generics.make_where_clause();

    if let Some(bounds) = ty_args.bounds.as_ref() {
        // add user's manual boundaries
        if !bounds.is_empty() {
            clause.predicates.extend(
                bounds
                    .split(",")
                    .map(|b| parse_str::<WherePredicate>(b).unwrap()),
            );
        }
    } else {
        // add `Field: Inspect + Default` for each fiel
        clause.predicates.extend(
            ty_args
                .all_fields()
                .iter()
                .filter(|f| !f.skip)
                .map(|f| &f.ty)
                .map::<WherePredicate, _>(|ty| parse_quote! { #ty: #inspect + Default }),
        );
    }

    generics
}
