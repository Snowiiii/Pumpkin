use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::sync::Mutex;
use syn::{parse_macro_input, ItemFn, ItemStruct};

static PLUGIN_METHODS: Lazy<Mutex<HashMap<String, Vec<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static PLUGIN_HOOKS: Lazy<Mutex<HashMap<String, Vec<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static PLUGIN_EVENT_NAMES: Lazy<Mutex<HashMap<String, Vec<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[proc_macro_attribute]
pub fn plugin_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_body = &input_fn.block;

    let struct_name = if attr.is_empty() {
        "MyPlugin".to_string()
    } else {
        attr.to_string().trim().to_string()
    };

    let method = quote! {
        #[allow(unused_mut)]
        fn #fn_name(#fn_inputs) #fn_output {
            #fn_body
        }
    }
    .to_string();

    PLUGIN_METHODS
        .lock()
        .unwrap()
        .entry(struct_name)
        .or_default()
        .push(method);

    TokenStream::new()
}

#[proc_macro_attribute]
pub fn plugin_event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_body = &input_fn.block;

    let struct_name = if attr.is_empty() {
        "MyPlugin".to_string()
    } else {
        attr.to_string().trim().to_string()
    };

    let method = quote! {
        #[allow(unused_mut)]
        fn #fn_name(#fn_inputs) #fn_output {
            #fn_body
        }
    }
    .to_string();

    PLUGIN_HOOKS
        .lock()
        .unwrap()
        .entry(struct_name.clone())
        .or_default()
        .push(method);

    PLUGIN_EVENT_NAMES
        .lock()
        .unwrap()
        .entry(struct_name)
        .or_default()
        .push("\"".to_owned() + &fn_name.to_string().trim_start_matches("on_").to_string() + "\"");

    TokenStream::new()
}

#[proc_macro_attribute]
pub fn plugin_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input struct
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_ident = &input_struct.ident;

    // Get the custom name from attribute or use the struct's name
    let struct_name = if attr.is_empty() {
        struct_ident.clone()
    } else {
        let attr_str = attr.to_string();
        quote::format_ident!("{}", attr_str.trim())
    };

    let methods = PLUGIN_METHODS
        .lock()
        .unwrap()
        .remove(&struct_name.to_string())
        .unwrap_or_default();

    let methods: Vec<proc_macro2::TokenStream> = methods
        .iter()
        .filter_map(|method_str| method_str.parse().ok())
        .collect();

    let hooks = PLUGIN_HOOKS
        .lock()
        .unwrap()
        .remove(&struct_name.to_string())
        .unwrap_or_default();

    let hooks: Vec<proc_macro2::TokenStream> = hooks
        .iter()
        .filter_map(|method_str| method_str.parse().ok())
        .collect();

    let events = PLUGIN_EVENT_NAMES
        .lock()
        .unwrap()
        .remove(&struct_name.to_string())
        .unwrap_or_default();

    let events: Vec<proc_macro2::TokenStream> = events
        .iter()
        .filter_map(|method_str| method_str.parse().ok())
        .collect();

    let event_names = events
        .iter()
        .map(|event| quote::quote!(#event))
        .collect::<Vec<_>>();

    // Combine the original struct definition with the impl block and plugin() function
    let expanded = quote! {
        #input_struct

        impl pumpkin::plugin::Plugin for #struct_ident {
            #(#methods)*
        }

        impl pumpkin::plugin::Hooks for #struct_ident {
            fn registered_events(&self) -> Result<&'static [&'static str], String> {
                static EVENTS: &[&str] = &[#(#event_names),*];
                Ok(EVENTS)
            }

            #(#hooks)*
        }

        #[no_mangle]
        pub fn plugin() -> Box<dyn pumpkin::plugin::Plugin> {
            Box::new(#struct_ident {})
        }

        #[no_mangle]
        pub fn hooks() -> Box<dyn pumpkin::plugin::Hooks> {
            Box::new(#struct_ident {})
        }
    };

    TokenStream::from(expanded)
}
