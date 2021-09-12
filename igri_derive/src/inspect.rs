mod args;
mod utils;

use darling::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::*;

use utils::{imgui_path, inspect_path};

/// Implements `Inspect`
pub fn impl_inspect(ast: syn::DeriveInput) -> TokenStream2 {
    let args = args::TypeArgs::from_derive_input(&ast).unwrap();

    match args.data {
        ast::Data::Struct(ref fields) => self::inspect_struct(&args, fields),
        ast::Data::Enum(ref fields) => self::inspect_enum(&args, fields),
    }
}

fn create_impl_generics(args: &args::TypeArgs) -> Generics {
    let mut generics = args.generics.clone();
    let inspect = inspect_path();

    let clause = generics.make_where_clause();

    if let Some(bounds) = args.bounds.as_ref() {
        // add user's manually boundary
        if !bounds.is_empty() {
            clause.predicates.extend(
                bounds
                    .split(",")
                    .map(|b| parse_str::<WherePredicate>(b).unwrap()),
            );
        }
    } else {
        // crate boundary for each field
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

/// Fill the `inspect` function body to derive `Inspect`
fn generate_inspect_impl(args: &args::TypeArgs, inspect_body: TokenStream2) -> TokenStream2 {
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

fn inspect_struct(args: &args::TypeArgs, fields: &ast::Fields<args::FieldArgs>) -> TokenStream2 {
    let imgui = imgui_path();
    let inspect = inspect_path();

    assert!(!(args.with.is_some() && args.as_.is_some()));

    let inspect = if let Some(as_) = args.as_.as_ref() {
        // #[inspect(as = "type")]

        let as_ = parse_str::<Type>(as_).unwrap();
        quote! {
            let mut x: #as_ = (*self).into();
            #inspect::inspect(&mut x, ui, label);
            *self = x.into();
        }
    } else if let Some(with) = args.with.as_ref() {
        // #[inspect(with = "function")]

        if let Ok(with) = parse_str::<ExprPath>(with) {
            quote! {
                #with(self, ui, label);
            }
        } else {
            panic!("invalid #[inspect(with)] argument");
        }
    } else {
        // FIXME: more permissive transparent inspection
        let is_transparent =
            fields.style == ast::Style::Tuple && fields.iter().filter(|x| !x.skip).count() == 1;
        if is_transparent {
            // delegate the inspection to the only field
            quote! {
                use #inspect;
                self.0.inspect(ui, label);
            }
        } else if args.in_place {
            // inspect each field
            let field_inspectors = utils::struct_field_inspectors(&fields);

            quote! {
                #(#field_inspectors)*
            }
        } else {
            // insert tree and inspect each field
            let field_inspectors = utils::struct_field_inspectors(&fields);

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

    self::generate_inspect_impl(args, inspect)
}

fn inspect_enum(args: &args::TypeArgs, variants: &[args::VariantArgs]) -> TokenStream2 {
    if variants.iter().all(|v| v.fields.is_empty()) {
        self::inspect_unit_enum(args, variants)
    } else {
        self::inspect_complex_enum(args, variants)
    }
}

/// Inspect the variant's fields
fn inspect_complex_enum(args: &args::TypeArgs, variants: &[args::VariantArgs]) -> TokenStream2 {
    let ty_ident = &args.ident;
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
                    #ty_ident::#v_ident { #(#f_idents),* } => {
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
                    #ty_ident::#v_ident(#(#f_idents),*) => {
                        #(
                            #inspect::inspect(#f_idents, ui, #labels);
                        )*
                    }
                }
            }
            ast::Style::Unit => quote! {
                #ty_ident::#v_ident
            },
        }
    });

    self::generate_inspect_impl(
        args,
        quote! {{
            match self {
                #(#matchers,)*
            }
        }},
    )
}

/// Show menu to choose one of the variants
fn inspect_unit_enum(args: &args::TypeArgs, variants: &[args::VariantArgs]) -> TokenStream2 {
    let ty_ident = &args.ident;

    // create `[TypeName::A, TypeName::B]`
    let variant_idents = variants
        .iter()
        .map(|v| format_ident!("{}", v.ident))
        .collect::<Vec<_>>();

    self::generate_inspect_impl(
        args,
        quote! {{
            const VARIANTS: &[#ty_ident] = &[#(#ty_ident::#variant_idents,)*];

            fn item_ix(variant: &#ty_ident) -> Option<usize> {
                VARIANTS
                    .iter()
                    .enumerate()
                    .find_map(|(i, v)| if v == variant { Some(i) } else { None })
            }

            let imgui_names: &[&'static str] = &[
                #(
                    stringify!(Self::#variant_idents),
                )*
            ];

            let mut ix = item_ix(self).unwrap();
            let index = ix.clone();

            if ui.combo(
                label,
                &mut ix,
                VARIANTS,
                // label function
                |v| {
                    let i = item_ix(v).unwrap();
                    std::borrow::Cow::Borrowed(imgui_names[i])
                },
            ) {
                    *self = VARIANTS[ix].clone();
            }
        }},
    )
}
