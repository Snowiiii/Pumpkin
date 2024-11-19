use git_version::git_version;
use std::env;

fn main() {
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    {
        let mut res = tauri_winres::WindowsResource::new();
        res.set_icon("../assets/icon.ico");
        res.set_language(0x0009); // English
        res.compile().unwrap();
    }

    let version = git_version!(fallback = "unknown");
    let git_version = match version {
        "unknown" => env::var("GIT_VERSION").unwrap_or("unknown".to_string()),
        _ => version.to_string(),
    };
    println!("cargo:rustc-env=GIT_VERSION={}", git_version);
}
