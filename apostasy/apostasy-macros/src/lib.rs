use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemFn, parse_macro_input, parse_quote};

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);

    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Sized + Send + Sync + 'static });

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let output = quote! {
        unsafe impl #impl_generics apostasy::engine::ecs::component::Component for #struct_name #type_generics
        #where_clause
        {
            fn id() -> apostasy::engine::ecs::entity::Entity {
                #[linkme::distributed_slice(apostasy::engine::ecs::component::COMPONENT_ENTRIES)]
                static ENTRY: apostasy::engine::ecs::component::ComponentEntry = #struct_name::init;
                let begin = apostasy::engine::ecs::component::COMPONENT_ENTRIES[..].as_ptr() as u32;
                let end = &raw const ENTRY as u32;
                unsafe {
                    apostasy::engine::ecs::entity::Entity::from_offset(
                        (end - begin) / size_of::<apostasy::engine::ecs::component::ComponentEntry>() as u32,
                    )
                }
            }

            fn init(world: &apostasy::engine::ecs::World) {
                // println!("initalizing for: {}", std::any::type_name::<#struct_name>());
                world.entity(#struct_name::id()).insert(#struct_name::info());
            }

            fn info() -> apostasy::engine::ecs::component::ComponentInfo {
                unsafe {
                    apostasy::engine::ecs::component::ComponentInfo {
                        name: std::any::type_name::<#struct_name>(),
                        align: std::mem::align_of::<#struct_name>(),
                        size: std::mem::size_of::<#struct_name>(),
                        id: #struct_name::id(),
                        drop: #struct_name::erased_drop,
                        clone: #struct_name::get_erased_clone(),
                        default: #struct_name::get_erased_default(),
                        on_insert: #struct_name::get_on_insert(),
                        on_remove: #struct_name::get_on_remove(),
                    }
                }
            }
        }
    };

    output.into()
}

#[proc_macro_derive(Resource)]
pub fn resource_derive(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);

    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Sized + Send + Sync + 'static });

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let output = quote! {
        unsafe impl #impl_generics apostasy::engine::ecs::resource::Resource for #struct_name #type_generics
        #where_clause
        {
            fn id() -> apostasy::engine::ecs::entity::Entity {
                #[linkme::distributed_slice(apostasy::engine::ecs::resource::RESOURCE_ENTRIES)]
                static ENTRY: apostasy::engine::ecs::resource::ResourceEntry = #struct_name::init;
                let begin = apostasy::engine::ecs::resource::RESOURCE_ENTRIES[..].as_ptr() as u32;
                let end = &raw const ENTRY as u32;
                unsafe {
                    apostasy::engine::ecs::entity::Entity::from_offset(
                        (end - begin) / size_of::<apostasy::engine::ecs::resource::ResourceEntry>() as u32,
                    )
                }
            }

            fn name() -> &'static str {
                std::any::type_name::<#struct_name>()
            }

            fn init(world: &mut apostasy::engine::ecs::World) {
            }
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn update(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;

    // Generate an inventory registration
    let expanded = quote! {
        #input_fn

        inventory::submit! {
            apostasy::engine::ecs::system::UpdateSystem {
                name: stringify!(#fn_name),
                func: #fn_name,
            }
        }
    };

    TokenStream::from(expanded)
}
