/*!
[`darling`] parses `syn::DeriveInput` and creates alternatives to `syn` types that contain attribute
information.
*/

use darling::*;
use syn::*;

#[derive(FromDeriveInput)]
#[darling(attributes(inspect), supports(struct_any, enum_any))]
pub struct TypeArgs {
    pub ident: Ident,
    pub generics: Generics,
    pub data: ast::Data<VariantArgs, FieldArgs>,
    /// Casts the target before inspection
    #[darling(default)]
    pub with: Option<String>,
    /// Convert by value via `Into` trait
    #[darling(default, rename = "as")]
    pub as_: Option<String>,
    /// Start field inspection without staring node
    #[darling(default)]
    pub in_place: bool,
    /// Open tree by default
    #[darling(default)]
    pub open: bool,
    /// Add type boundary manually
    #[darling(default)]
    pub bounds: Option<String>,
}

#[derive(FromField, Clone)]
#[darling(attributes(inspect))]
pub struct FieldArgs {
    pub ident: Option<Ident>,
    pub ty: Type,
    // ---
    /// `#[inspect(skip)]`
    ///
    /// Skip inspection and `where Field: Inspect` boundary
    #[darling(default)]
    pub skip: bool,
    /// `#[inspect(as = <ty>)]`
    ///
    /// Convert by value via `Into` trait
    #[darling(default, rename = "as")]
    pub as_: Option<String>,
    /// `#[inspect(with = "<path>")]`
    ///
    /// Casts the target before inspection
    #[darling(default)]
    pub with: Option<String>,
}

#[derive(FromVariant)]
#[darling(attributes(inspect))]
pub struct VariantArgs {
    pub ident: Ident,
    pub fields: ast::Fields<FieldArgs>,
}

impl TypeArgs {
    /// Enumerates all the fields of a struct or enum variants
    pub fn all_fields(&self) -> Vec<self::FieldArgs> {
        match &self.data {
            ast::Data::Struct(field_args) => field_args.fields.clone(),
            ast::Data::Enum(variants) => variants
                .iter()
                .flat_map(|variant| variant.fields.clone().into_iter())
                .collect::<Vec<_>>(),
        }
    }
}
