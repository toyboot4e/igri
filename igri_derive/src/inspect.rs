mod args;
mod utils;

use darling::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;

use self::utils::{imgui_path, inspect_path};

/// Implements `Inspect`
pub fn impl_inspect(ast: syn::DeriveInput) -> TokenStream2 {
    // The derive input is parsed into `darling` types defined in `args` module.
    // (`darling` lets us parse `#[attribute(..)]` in declartive style, while `syn` does not).
    let args = args::TypeArgs::from_derive_input(&ast).unwrap();

    assert!(
        !(args.with.is_some() && args.as_.is_some()),
        "tried to use both #[inspect(with = ..)] and #[inspect(as = ..)]"
    );

    match args.data {
        ast::Data::Struct(ref fields) => self::inspect_struct(&args, fields),
        ast::Data::Enum(ref fields) => self::inspect_enum(&args, fields),
    }
}

fn inspect_struct(
    ty_args: &args::TypeArgs,
    field_args: &ast::Fields<args::FieldArgs>,
) -> TokenStream2 {
    let imgui = imgui_path();
    let inspect = inspect_path();

    let inspect = if let Some(as_) = ty_args.as_.as_ref() {
        // case 1. #[inspect(as = "type")]
        utils::impl_inspect_as(quote!(self), as_)
    } else if let Some(with) = ty_args.with.as_ref() {
        // case 2. #[inspect(with = "function")]
        utils::impl_inspect_with(quote!(self), with)
    } else {
        // FIXME: more permissive transparent inspection
        let is_transparent = field_args.style == ast::Style::Tuple
            && field_args.iter().filter(|x| !x.skip).count() == 1;
        if is_transparent {
            // case 3. Transparent inspection
            quote! {
                use #inspect;
                self.0.inspect(ui, label);
            }
        } else if ty_args.in_place {
            // case 4. Flatten
            let field_inspectors =
                utils::field_inspectors(|field| quote! { (&mut self.#field) }, &field_args);

            quote! {
                #(#field_inspectors)*
            }
        } else {
            // case 5. Nest tree node
            let mut field_inspectors =
                utils::field_inspectors(|field| quote! { (&mut self.#field) }, &field_args)
                    .peekable();

            if field_inspectors.peek().is_none() {
                // unit struct, no field
                let ty_ident = &ty_args.ident;
                quote! {
                    ui.label_text(label, concat!("<", stringify!(#ty_ident), ">"));
                }
            } else {
                // tuple or named fields
                let open = ty_args.open;
                quote! {
                    let _ = #imgui::TreeNode::new(label)
                        .flags(
                            #imgui::TreeNodeFlags::OPEN_ON_ARROW |
                            #imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK
                        )
                        .default_open(#open)
                        .build(ui, ||
                               {
                                   #(#field_inspectors)*
                               });
                }
            }
        }
    };

    utils::impl_inspect(ty_args, utils::struct_inspect_generics(ty_args), inspect)
}

fn inspect_enum(ty_args: &args::TypeArgs, variant_args: &[args::VariantArgs]) -> TokenStream2 {
    let tag_selector = utils::enum_tag_selector(ty_args, variant_args);

    // collect field inspectors
    let matchers = variant_args.iter().map(|v| {
        let v_ident = &v.ident;

        match v.fields.style {
            ast::Style::Struct => {
                let f_idents = v
                    .fields
                    .iter()
                    .map(|f| {
                        let ident = &f.ident;
                        quote!(#ident)
                    })
                    .collect::<Vec<_>>();

                let field_inspectors = utils::field_inspectors(|field| field, &v.fields);

                quote! {
                    Self::#v_ident { #(#f_idents),* } => {
                        #(#field_inspectors)*
                    }
                }
            }
            ast::Style::Tuple => {
                let f_idents = (0..v.fields.len())
                    .map(|i| format_ident!("f{}", syn::Index::from(i)))
                    .collect::<Vec<_>>();

                let field_inspectors = utils::field_inspectors(
                    |field| {
                        use quote::*;
                        use syn::*;

                        let x: Index = parse_quote! {#field};
                        let f_ident = format_ident!("f{}", x);

                        quote! { #f_ident }
                    },
                    &v.fields,
                );

                quote! {
                    Self::#v_ident(#(#f_idents),*) => {
                        #(#field_inspectors)*
                    }
                }
            }
            ast::Style::Unit => quote! {
                Self::#v_ident => {}
            },
        }
    });

    let imgui = utils::imgui_path();

    let body = if variant_args.iter().all(|v| v.fields.is_empty()) {
        // plain enum: tag selector only
        quote! {
            #tag_selector
        }
    } else {
        quote! {
            if let Some(()) = #imgui::TreeNode::new(label)
                .opened(true, #imgui::Condition::FirstUseEver)
                .flags(#imgui::TreeNodeFlags::OPEN_ON_ARROW | #imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK)
                .build(ui, || {
                    #tag_selector

                    match self {
                        #(#matchers,)*
                    }
                })
            {}
        }
    };

    utils::impl_inspect(ty_args, utils::enum_inspect_generics(ty_args), body)
}
