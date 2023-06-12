use core::panic;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let expanded = quote! {
        impl Component for #name {
            fn to_any(self: Box<Self>) -> Box<dyn std::any::Any> {
                self
            }
            fn to_store(self: Box<Self>) -> ComponentStore {
                (*self).into()
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let fields = match input.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(fields) => fields.named,
            _ => panic!("Expected to be a named struct"),
        },
        _ => panic!("Expected to be a struct"),
    };

    let mut field_types: Vec<_> = Vec::new();

    let mut ref_names: Vec<_> = Vec::new();
    let mut mut_names: Vec<_> = Vec::new();

    let mut ref_elems: Vec<_> = Vec::new();
    let mut mut_elems: Vec<_> = Vec::new();

    for field in fields.iter() {
        if let syn::Type::Reference(ty) = &field.ty {
            field_types.push(&ty.elem);

            if ty.mutability.is_none() {
                ref_names.push(&field.ident);
                ref_elems.push(&ty.elem);
            } else {
                mut_names.push(&field.ident);
                mut_elems.push(&ty.elem);
            }
        }
    }

    let ref_idx: Vec<_> = ref_elems.iter().enumerate().map(|(idx, _)| idx).collect();
    let mut_idx: Vec<_> = mut_elems.iter().enumerate().map(|(idx, _)| idx).collect();

    let expanded = quote! {
        impl Model for #name<'_> {
            type Row<'r> = #name<'r>;

            fn get_types() -> ecs::bundle::TypeBundle {
                ecs::bundle::TypeBundle::from([#(std::any::TypeId::of::<#field_types>()), *].as_slice())
            }

            fn get_reads(at: &ecs::archetype::Archetype) -> Vec<ecs::component::ReadGuard> {
                vec![#(at.get_storage(std::any::TypeId::of::<#ref_elems>()).unwrap().inner()), *]
            }

            fn get_writes(at: &ecs::archetype::Archetype) -> Vec<ecs::component::WriteGuard> {
                vec![#(at.get_storage(std::any::TypeId::of::<#mut_elems>()).unwrap().inner_mut()), *]
            }

            fn process<F>(
                reads: Vec<ecs::component::ReadGuard>,
                mut writes: Vec<ecs::component::WriteGuard>,
                row: usize,
                system: &mut F,
            ) where
                for<'f> F: FnMut(Self::Row<'f>),
            {
                #(let #ref_names: &#ref_elems = reads[#ref_idx]
                    .to_any()
                    .downcast_ref::<Vec<#ref_elems>>()
                    .unwrap()
                    .get(row)
                    .unwrap();)
                *

                #(let #mut_names: &mut #mut_elems = writes[#mut_idx]
                    .to_any_mut()
                    .downcast_mut::<Vec<#mut_elems>>()
                    .unwrap()
                    .get_mut(row)
                    .unwrap();)
                *

                let row: Self::Row<'_> = #name { #(#ref_names), *, #(#mut_names), * };

                system(row);
            }

        }
    };

    proc_macro::TokenStream::from(expanded)
}
