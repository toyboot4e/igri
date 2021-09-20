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

fn inspect_struct(args: &args::TypeArgs, fields: &ast::Fields<args::FieldArgs>) -> TokenStream2 {
    let imgui = imgui_path();
    let inspect = inspect_path();

    let inspect = if let Some(as_) = args.as_.as_ref() {
        // case 1. #[inspect(as = "type")]
        utils::impl_inspect_as(quote!(self), as_)
    } else if let Some(with) = args.with.as_ref() {
        // case 2. #[inspect(with = "function")]
        utils::impl_inspect_with(quote!(self), with)
    } else {
        // FIXME: more permissive transparent inspection
        let is_transparent =
            fields.style == ast::Style::Tuple && fields.iter().filter(|x| !x.skip).count() == 1;
        if is_transparent {
            // case 3. Transparent inspection
            quote! {
                use #inspect;
                self.0.inspect(ui, label);
            }
        } else if args.in_place {
            // case 4. Flatten
            let field_inspectors = utils::field_inspectors(quote! { self }, &fields);

            quote! {
                #(#field_inspectors)*
            }
        } else {
            // case 5. Nest tree node
            let field_inspectors = utils::field_inspectors(quote! { self }, &fields);

            let open = args.open;
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
    };

    utils::generate_inspect_impl(args, inspect)
}

fn inspect_enum(args: &args::TypeArgs, variants: &[args::VariantArgs]) -> TokenStream2 {
    if variants.iter().all(|v| v.fields.is_empty()) {
        self::inspect_plain_enum(args, variants)
    } else {
        self::inspect_complex_enum(args, variants)
    }
}

/// Show menu to choose one of the variants
fn inspect_plain_enum(ty_args: &args::TypeArgs, ty_variants: &[args::VariantArgs]) -> TokenStream2 {
    let tag_selector = utils::enum_tag_selector(ty_args, ty_variants);

    utils::generate_inspect_impl(
        ty_args,
        quote! {
            #tag_selector
        },
    )
}

/// Inspect the variant's fields
fn inspect_complex_enum(args: &args::TypeArgs, variants: &[args::VariantArgs]) -> TokenStream2 {
    let inspect = inspect_path();

    let matchers = variants.iter().map(|v| {
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

                let labels = v
                    .fields
                    .iter()
                    .map(|f| format!("{}", f.ident.as_ref().unwrap()));

                quote! {
                    Self::#v_ident { #(#f_idents),* } => {
                        #(
                            #inspect::inspect(#f_idents, ui, #labels);
                        )*
                    }
                }
            }
            ast::Style::Tuple => {
                let f_idents = (0..v.fields.len())
                    .map(|i| format_ident!("f{}", i))
                    .collect::<Vec<_>>();

                let labels = (0..v.fields.len()).map(|i| format!("{}", i));

                quote! {
                    Self::#v_ident(#(#f_idents),*) => {
                        #(
                            #inspect::inspect(#f_idents, ui, #labels);
                        )*
                    }
                }
            }
            ast::Style::Unit => quote! {
                Self::#v_ident => {
                    ui.label_text(label, stringify!(#v_ident));
                }
            },
        }
    });

    utils::generate_inspect_impl(
        args,
        quote! {
            match self {
                #(#matchers,)*
            }
        },
    )
}
