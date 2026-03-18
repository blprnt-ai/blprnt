use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

/// Derive macro to implement SurrealValue for enums that support Display and FromStr
///
/// Usage:
/// ```ignore
/// #[derive(Clone, Debug, Display, FromStr, SurrealEnumValue)]
/// #[serde(rename_all = "snake_case")]
/// pub enum MyEnum {
///   Variant1,
///   Variant2,
/// }
/// ```
#[proc_macro_derive(SurrealEnumValue)]
pub fn derive_surreal_value_enum(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  let expanded = quote! {
    impl #impl_generics surrealdb_types::SurrealValue for #name #ty_generics #where_clause {
      fn into_value(self) -> surrealdb_types::Value {
        self.to_string().into_value()
      }

      fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb_types::Error>
      where
        Self: Sized,
      {
        let s = String::from_value(value)?;
        std::str::FromStr::from_str(&s).map_err(|_e| {
          surrealdb_types::Error::serialization(
            format!("Failed to parse {} from '{}'", stringify!(#name), s),
            Some(surrealdb_types::SerializationError::Deserialization),
          )
        })
      }

      fn kind_of() -> surrealdb_types::Kind {
        surrealdb_types::Kind::String
      }
    }
  };

  TokenStream::from(expanded)
}
