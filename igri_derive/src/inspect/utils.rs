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
        bridge.inspect(ui, label);
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

/// `<prefix>field.inspect(ui, label);`
pub fn field_inspectors<'a>(
    mut field_map: impl FnMut(TokenStream2) -> TokenStream2 + 'a,
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
                ast::Style::Unit => return quote! {},
            };

            let field_ident = field_map(field_ident);

            if let Some(as_) = field.as_.as_ref() {
                // #[inspect(as = "type")]

                let as_ = parse_str::<Type>(as_).unwrap();
                quote! {
                    let mut x: #as_ = (*#field_ident).into();
                    x.inspect(ui, label);
                    *#field_ident = x.into();
                }
            } else if let Some(with) = field.with.as_ref() {
                // #[inspect(with = "function")]

                if let Ok(with) = parse_str::<ExprPath>(with) {
                    quote! {
                        (#with)(&mut #field_ident);
                    }
                } else {
                    panic!("invalid #[inspect(with)] argument; be sure to give path");
                }
            } else {
                // inspect the value as-is
                quote! {
                    #field_ident.inspect(ui, #label);
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

    let ty_variants = ty_variants.iter().collect::<Vec<_>>();

    // List of `TypeName::Variant`
    let v_idents = ty_variants
        .iter()
        .map(|v| format_ident!("{}", v.ident))
        .collect::<Vec<_>>();

    let indices = (0..v_idents.len()).map(Index::from).collect::<Vec<_>>();
    let default_variants = self::default_variants(ty_args, &ty_variants).collect::<Vec<_>>();

    let index_matchers = ty_variants.iter().enumerate().map(|(index, v)| {
        let v_ident = &v.ident;

        match v.fields.style {
            ast::Style::Struct => {
                quote! {
                    #ty_ident::#v_ident { .. } => #index,
                }
            }
            ast::Style::Tuple => {
                quote! {
                    #ty_ident::#v_ident(..) => #index,
                }
            }
            ast::Style::Unit => {
                quote! {
                    #ty_ident::#v_ident => #index,
                }
            }
        }
    });

    quote! {
        const NAMES: &'static [&'static str] = &[
            #(
                stringify!(#v_idents),
            )*
        ];

        let mut ix = match self {
            #(#index_matchers)*
        };

        if ui.combo_simple_string(
            label,
            &mut ix,
            NAMES,
        ) {
            *self = match ix {
                #(
                    _ if ix == #indices => #default_variants,
                )*
                _ => unreachable!(),
            }
        }
    }
}

fn default_variants<'a>(
    ty_args: &'a args::TypeArgs,
    ty_variants: &'a [&'a args::VariantArgs],
) -> impl Iterator<Item = TokenStream2> + 'a {
    let ty_ident = &ty_args.ident;

    ty_variants.iter().map(move |v| {
        let v_ident = &v.ident;

        let fields = v.fields.iter().collect::<Vec<_>>();

        // default variant
        match v.fields.style {
            ast::Style::Struct => {
                let field_idents = fields.iter().map(|f| {
                    let ident = &f.ident;
                    quote! { #ident }
                });

                quote! {
                    #ty_ident::#v_ident {
                        #(
                            #field_idents: Default::default(),
                        )*
                    }
                }
            }
            ast::Style::Tuple => {
                let xs = (0..fields.len()).map(|_i| quote! { Default::default });

                quote! {
                    #ty_ident::#v_ident (
                        #(#xs,)*
                    )
                }
            }
            ast::Style::Unit => {
                quote! {
                    #ty_ident::#v_ident
                }
            }
        }
    })
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
        // add `Field: Inspect + Default` for each field
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
