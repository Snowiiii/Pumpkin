use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ItemStruct};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static PLUGIN_METHODS: Lazy<Mutex<HashMap<String, Vec<String>>>> = 
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
    }.to_string();
    
    PLUGIN_METHODS.lock().unwrap()
        .entry(struct_name)
        .or_default()
        .push(method);
    
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
    
    let methods = PLUGIN_METHODS.lock().unwrap()
        .remove(&struct_name.to_string())
        .unwrap_or_default();
    
    let methods: Vec<proc_macro2::TokenStream> = methods.iter()
        .filter_map(|method_str| method_str.parse().ok())
        .collect();
    
    // Combine the original struct definition with the impl block and plugin() function
    let expanded = quote! {
        #input_struct

        impl pumpkin_api::Plugin for #struct_ident {
            #(#methods)*
        }

        #[no_mangle]
        pub fn plugin() -> Box<dyn pumpkin_api::Plugin> {
            Box::new(#struct_ident {})
        }
    };
    
    TokenStream::from(expanded)
}