use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(DeriveComponent)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Component for #name #ty_generics #where_clause {
            fn type_id_dyn(&self) -> std::any::TypeId {
                std::any::TypeId::of::<Self>()
            }
        }
        impl #impl_generics PartialEq for #name #ty_generics #where_clause{
            fn eq(&self, other: &Self) -> bool {
                if self.type_id_dyn() != other.type_id_dyn() {
                    return false;
                }
                self.type_id_dyn() == other.type_id_dyn()
            }
        }
    };

    TokenStream::from(expanded)
}
