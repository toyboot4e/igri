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

/// `self.field.inspect(ui, label);`
pub fn struct_field_inspectors<'a>(
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
                    // `self.0`, not `self.0usize` for example
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
                    panic!("invalid #[inspect(with)] argument");
                }
            } else {
                // inspect the value as-is
                quote! {
                    self.#field_ident.inspect(ui, #label);
                }
            }
        })
}
