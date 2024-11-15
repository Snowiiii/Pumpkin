#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// #![warn(clippy::restriction)]
#![deny(clippy::cargo)]
// REMOVE SOME WHEN RELEASE
#![expect(clippy::cargo_common_metadata)]
#![expect(clippy::multiple_crate_versions)]
#![expect(clippy::single_call_fn)]
#![expect(clippy::cast_sign_loss)]
#![expect(clippy::cast_possible_truncation)]
#![expect(clippy::cast_possible_wrap)]
#![expect(clippy::missing_panics_doc)]
#![expect(clippy::missing_errors_doc)]
#![expect(clippy::module_name_repetitions)]
#![expect(clippy::struct_excessive_bools)]

use tokio::io;

#[cfg(target_os = "wasi")]
compile_error!("Compiling for WASI targets is not supported!");

#[tokio::main]
async fn main() -> io::Result<()> {
    pumpkin::main().await
}
