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

/// `self.field.inspect(ui, label);`
pub fn field_inspectors<'a>(
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
                    // Convert into `Index` type (e.g. `self.0`, not `self.0usize` for example)
                    let field_ident = Index::from(field_index);
                    (quote!(#field_ident), format!("{}", field_index))
                }
                ast::Style::Unit => unreachable!(),
            };

            if let Some(as_) = field.as_.as_ref() {
                // #[inspect(as = "type")]

                let as_ = parse_str::<Type>(as_).unwrap();
                quote! {
                    let mut x: #as_ = (*self.#field_ident).into();
                    #inspect::inspect(&mut x, ui, label);
                    *self.#field_ident = x.into();
                }
            } else if let Some(with) = field.with.as_ref() {
                // #[inspect(with = "function")]

                if let Ok(with) = parse_str::<ExprPath>(with) {
                    quote! {
                        (#with)(&mut self.#field_ident);
                    }
                } else {
                    panic!("invalid #[inspect(with)] argument; be sure to give path");
                }
            } else {
                // inspect the value as-is
                quote! {
                    self.#field_ident.inspect(ui, #label);
                }
            }
        })
}

/// Fill the `inspect` function body to derive `Inspect`
pub fn generate_inspect_impl(args: &args::TypeArgs, inspect_body: TokenStream2) -> TokenStream2 {
    let generics = self::create_impl_generics(args);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ty_ident = &args.ident;

    let imgui = imgui_path();
    let inspect = inspect_path();

    quote! {
        impl #impl_generics #inspect for #ty_ident #ty_generics #where_clause
        {
            fn inspect(&mut self, ui: &#imgui::Ui, label: &str) {
                #inspect_body
            }
        }
    }
}

/// Creates `where` clause for `Inspect` impl
fn create_impl_generics(args: &args::TypeArgs) -> Generics {
    let mut generics = args.generics.clone();
    let inspect = inspect_path();

    let clause = generics.make_where_clause();

    if let Some(bounds) = args.bounds.as_ref() {
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
            args.all_fields()
                .iter()
                .filter(|f| !f.skip)
                .map(|f| &f.ty)
                .map::<WherePredicate, _>(|ty| parse_quote! { #ty: #inspect }),
        );
    }

    generics
}
