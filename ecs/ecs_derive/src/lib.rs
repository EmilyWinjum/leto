use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let expanded = quote! {
        impl Component for #name {
            fn to_any(self: Box<Self>) -> Box<dyn std::any::Any> {
                self
            }
            fn to_store(self: Box<Self>) -> ComponentStore {
                ComponentStore::from(*self)
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
