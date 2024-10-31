use pumpkin_egui::TemplateApp;

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
