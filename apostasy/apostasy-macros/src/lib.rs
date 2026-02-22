use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, ItemFn, LitInt, parse_macro_input, parse_quote};

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Clone + Send + Sync + 'static });
    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();
    let output = quote! {
        impl #impl_generics apostasy::engine::nodes::component::Component for #struct_name #type_generics
        #where_clause
        {
            fn name() -> &'static str where Self: Sized {
                std::any::type_name::<#struct_name>()
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
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

/// Registers a ui system, UI systems run every frame, allowing for custom UI elements
#[proc_macro_attribute]
pub fn ui(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as SystemArgs);
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let priority = args.priority.unwrap_or(0);

    let expanded = quote! {
        #input_fn
        inventory::submit! {
            apostasy::engine::rendering::renderer::UIFunction{
                name: stringify!(#fn_name),
                func: #fn_name,
                priority: #priority,
            }
        }
    };
    TokenStream::from(expanded)
}

/// Registers a console command
#[proc_macro_attribute]
pub fn console_command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn
        inventory::submit! {
            apostasy::engine::editor::console_commands::ConsoleCommand{
                name: stringify!(#fn_name),
                func: #fn_name,
            }
        }
    };
    TokenStream::from(expanded)
}
