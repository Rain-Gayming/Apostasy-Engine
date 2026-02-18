use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, ItemFn, LitInt, parse_macro_input, parse_quote};

/// Registers a component, Components are used to store data that is in an entity
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
                        fmt: #struct_name::get_erased_fmt(),
                    }
                }
            }
        }
    };

    output.into()
}

/// Registers a resource, Resources are used to store data that is shared between systems
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

struct SystemArgs {
    priority: Option<u32>,
}

/// Parser for the attribute arguments
impl Parse for SystemArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(SystemArgs { priority: None });
        }

        let name: syn::Ident = input.parse()?;
        if name != "priority" {
            return Err(syn::Error::new_spanned(name, "expected `priority`"));
        }

        input.parse::<syn::Token![=]>()?;
        let priority_lit: LitInt = input.parse()?;
        let priority: u32 = priority_lit.base10_parse()?;

        Ok(SystemArgs {
            priority: Some(priority),
        })
    }
}

/// Registers a start system, Start systems run once at the start of the game
#[proc_macro_attribute]
pub fn start(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as SystemArgs);
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let priority = args.priority.unwrap_or(0);

    let expanded = quote! {
        #input_fn
        inventory::submit! {
            apostasy::engine::ecs::system::StartSystem{
                name: stringify!(#fn_name),
                func: #fn_name,
                priority: #priority,
            }
        }
    };
    TokenStream::from(expanded)
}
/// Registers a fixed update system, Fixed updates run a specific amount of times per second
#[proc_macro_attribute]
pub fn fixed_update(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let args = parse_macro_input!(attr as SystemArgs);
    let fn_name = &input_fn.sig.ident;
    let priority = args.priority.unwrap_or(0);

    // Generate an inventory registration
    let expanded = quote! {
        #input_fn

        inventory::submit! {
            apostasy::engine::ecs::system::FixedUpdateSystem {
                name: stringify!(#fn_name),
                func: #fn_name,
                priority: #priority,
            }
        }
    };

    TokenStream::from(expanded)
}

/// Registers an update system, Update systems run every frame
#[proc_macro_attribute]
pub fn update(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let args = parse_macro_input!(attr as SystemArgs);
    let priority = args.priority.unwrap_or(0);

    // Generate an inventory registration
    let expanded = quote! {
        #input_fn

        inventory::submit! {
            apostasy::engine::ecs::system::UpdateSystem {
                name: stringify!(#fn_name),
                func: #fn_name,
                priority: #priority,
            }
        }
    };

    TokenStream::from(expanded)
}

/// Registers an late update system, Update systems run at the end of every frame
#[proc_macro_attribute]
pub fn late_update(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let args = parse_macro_input!(attr as SystemArgs);
    let priority = args.priority.unwrap_or(0);

    // Generate an inventory registration
    let expanded = quote! {
        #input_fn

        inventory::submit! {
            apostasy::engine::ecs::system::LateUpdateSystem {
                name: stringify!(#fn_name),
                func: #fn_name,
                priority: #priority,
            }
        }
    };

    TokenStream::from(expanded)
}

/// Registers a start system, Start systems run once at the start of the game
#[proc_macro_attribute]
pub fn ui(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as SystemArgs);
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let priority = args.priority.unwrap_or(0);

    let expanded = quote! {
        #input_fn
        inventory::submit! {
            apostasy::engine::ecs::system::UIFunction{
                name: stringify!(#fn_name),
                func: #fn_name,
                priority: #priority,
            }
        }
    };
    TokenStream::from(expanded)
}
