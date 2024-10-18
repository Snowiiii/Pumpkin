use std::{io::Cursor, sync::LazyLock};

use base64::{engine::general_purpose, Engine as _};
use proc_macro::TokenStream;
use quote::quote;

// TODO: This is the same as pumpkin/src/server/connection_cache.rs, but we cannot reference that in
// this crate
fn load_icon(data: &[u8]) -> String {
    let icon = png::Decoder::new(Cursor::new(data));
    let reader = icon.read_info().unwrap();
    let info = reader.info();
    assert!(info.width == 64, "Icon width must be 64");
    assert!(info.height == 64, "Icon height must be 64");

    // Once we validate the dimensions, we can encode the image as-is
    let mut result = "data:image/png;base64,".to_owned();
    general_purpose::STANDARD.encode_string(data, &mut result);
    result
}

static ICON: LazyLock<&[u8]> = LazyLock::new(|| include_bytes!("../../assets/default_icon.png"));

pub fn create_icon_impl() -> TokenStream {
    let encoded_icon = load_icon(&ICON);
    quote! {
        #encoded_icon
    }
    .into()
}
