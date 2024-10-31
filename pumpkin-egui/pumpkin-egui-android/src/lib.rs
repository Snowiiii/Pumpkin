use std::{fs::{self, File}, io::Write, path::Path};

use pumpkin_egui::TemplateApp;

const DEFAULT_CONF: &str = include_str!("../../../configuration.toml");
const DEFAULT_ADV_CONF: &str = include_str!("../../../features.toml");

use j4rs::{jni_sys::{jint, JavaVM, JNI_VERSION_1_6}, prelude::jobject};

#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
pub extern fn JNI_OnLoad(env: *mut JavaVM, _reserved: jobject) -> jint {
    j4rs::set_java_vm(env);
    JNI_VERSION_1_6
}

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(
    app: egui_winit::winit::platform::android::activity::AndroidApp,
) -> Result<(), Box<dyn std::error::Error>> {
    use egui_winit::winit::platform::android::EventLoopBuilderExtAndroid;
    pumpkin_egui_logger::builder().init().unwrap();
    /*android_logger::init_once(
        android_logger::Config::default()
            .with_tag("pumpkin_egui_android")
            .with_max_level(log::LevelFilter::Info),
    );*/

    if !Path::new("/storage/emulated/0/Documents/Pumpkin").exists() {
        fs::create_dir("/storage/emulated/0/Documents/Pumpkin").unwrap();
        let mut f = File::create("/storage/emulated/0/Documents/Pumpkin/configuration.toml").unwrap();
        f.write_all(DEFAULT_CONF.as_bytes()).unwrap();
        let mut f = File::create("/storage/emulated/0/Documents/Pumpkin/features.toml").unwrap();
        f.write_all(DEFAULT_ADV_CONF.as_bytes()).unwrap();
    }

    let mut options = eframe::NativeOptions::default();
    options.renderer = eframe::Renderer::Glow;
    options.event_loop_builder = Some(Box::new(move |builder| {
        builder.with_android_app(app);
    }));
    eframe::run_native(
        "pumpkin_egui_android",
        options,
        Box::new(|cc| Ok(Box::new(TemplateApp::new(cc)))),
    )?;

    Ok(())
}
