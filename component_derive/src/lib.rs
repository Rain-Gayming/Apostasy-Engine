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

            fn clone_box(&self) -> Box<dyn Component> where Self: Clone {
                Box::new(self.clone())
            }
        }

        impl #impl_generics PartialEq for #name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                if self.type_id_dyn() != other.type_id_dyn() {
                    return false;
                }
                self.type_id_dyn() == other.type_id_dyn()
            }
        }

        impl #impl_generics ::std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct("type")
                    .field("type_id", &::std::any::TypeId::of::<Self>())
                    .finish()
            }
        }
    };

    TokenStream::from(expanded)
}
