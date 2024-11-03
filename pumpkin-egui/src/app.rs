use std::{
    io,
    sync::{Arc, LazyLock},
};

use egui::mutex::Mutex;
use pumpkin::{commands::CommandSender, server::Server};
use pumpkin_core::text::{color::NamedColor, TextComponent};
use tokio::{runtime, task::JoinHandle};

static SERVER: LazyLock<Mutex<Option<Arc<Server>>>> = LazyLock::new(|| Mutex::new(None));

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)]
    rt: runtime::Runtime,
    #[serde(skip)]
    started: bool,
    #[serde(skip)]
    server_handle: Option<JoinHandle<io::Result<()>>>,

    command: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            rt: runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            started: false,
            server_handle: None,
            command: String::new(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // ensure rayon is built outside of tokio scope
        rayon::ThreadPoolBuilder::new().build_global().unwrap();
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Pumpkin server");
            ui.separator();
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.started, egui::Button::new("Start"))
                    .clicked()
                {
                    log::info!("[GUI Client] Start button pressed");
                    self.server_handle = Some(self.rt.spawn(pumpkin::server_start(|server| {
                        *SERVER.lock() = Some(server.clone())
                    })));
                    self.started = !self.started;
                }
                if ui
                    .add_enabled(self.started, egui::Button::new("Stop"))
                    .clicked()
                {
                    log::warn!(
                        "{}",
                        TextComponent::text("Stop button pressed; stopping server...")
                            .color_named(NamedColor::Red)
                            .to_pretty_console()
                    );
                    *SERVER.lock() = None;
                    self.rt = runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    self.started = !self.started;
                }

                if self.started {
                    if let Some(handle) = &self.server_handle {
                        if handle.is_finished() {
                            self.started = !self.started
                        }
                    }
                }
            });
            pumpkin_egui_logger::logger_ui().show(ui);
            ui.horizontal(|ui| {
                #[cfg(not(target_os = "android"))]
                ui.add_sized(
                    ui.available_size() - egui::vec2(43.0, 0.0),
                    egui::TextEdit::singleline(&mut self.command),
                );

                #[cfg(target_os = "android")]
                {
                    let textedit = ui.add_sized(
                        ui.available_size() - egui::vec2(43.0, 0.0),
                        egui::TextEdit::singleline(&mut self.command),
                    );
    
                    if textedit.gained_focus() {
                        show_soft_input(true);
                    } else if textedit.lost_focus() {
                        show_soft_input(false);
                    }
                }

                if ui.button("Send").clicked() {
                    if self.started {
                        if !self.command.is_empty() {
                            let cmd = self.command.clone();
                            self.rt.spawn(async move {
                                if let Some(server) = SERVER.lock().as_ref() {
                                    server
                                        .command_dispatcher
                                        .handle_command(&mut CommandSender::Console, &server, &cmd)
                                        .await;
                                } else {
                                    panic!("Command send, but server not found");
                                }
                            });
                            self.command = String::new()
                        }
                    } else {
                        log::warn!("[GUI Client] Server not started for sending commands")
                    }
                }
            });

            powered_by_egui_and_eframe(ui);
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

// Simplest workaround for keyboar invocation
#[cfg(target_os = "android")]
pub fn show_soft_input(show: bool) -> bool {
    use jni::objects::JValue;

    let ctx = ndk_context::android_context();
    let vm = match unsafe { jni::JavaVM::from_raw(ctx.vm() as _) } {
        Ok(value) => value,
        Err(e) => {
            log::error!("vm not found: {e}");
            return false;
        }
    };
    let activity = unsafe { jni::objects::JObject::from_raw(ctx.context() as _) };
    let mut env = match vm.attach_current_thread() {
        Ok(value) => value,
        Err(e) => {
            log::error!("env not found: {e}");
            return false;
        }
    };

    let class_ctxt = match env.find_class("android/content/Context") {
        Ok(value) => value,
        Err(e) => {
            log::error!("context class not found: {e}");
            return false;
        }
    };
    let ims = match env.get_static_field(class_ctxt, "INPUT_METHOD_SERVICE", "Ljava/lang/String;") {
        Ok(value) => value,
        Err(e) => {
            log::error!("input method service not found: {e}");
            return false;
        }
    };

    let im_manager = match env
        .call_method(&activity, "getSystemService", "(Ljava/lang/String;)Ljava/lang/Object;", &[ims.borrow()])
        .unwrap()
        .l()
    {
        Ok(value) => value,
        Err(e) => {
            log::error!("input manager not found: {e}");
            return false;
        }
    };

    let jni_window = match env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[]).unwrap().l() {
        Ok(value) => value,
        Err(e) => {
            log::error!("window not found: {e}");
            return false;
        }
    };
    let view = match env.call_method(jni_window, "getDecorView", "()Landroid/view/View;", &[]).unwrap().l() {
        Ok(value) => value,
        Err(e) => {
            log::error!("virtual keyboard not found: {e}");
            return false;
        }
    };

    if show {
        let result = env
            .call_method(im_manager, "showSoftInput", "(Landroid/view/View;I)Z", &[JValue::Object(&view), 0i32.into()])
            .unwrap()
            .z()
            .unwrap();
        result
    } else {
        let window_token = env.call_method(view, "getWindowToken", "()Landroid/os/IBinder;", &[]).unwrap().l().unwrap();
        let jvalue_window_token = jni::objects::JValueGen::Object(&window_token);

        let result = env
            .call_method(
                im_manager,
                "hideSoftInputFromWindow",
                "(Landroid/os/IBinder;I)Z",
                &[jvalue_window_token, 0i32.into()],
            )
            .unwrap()
            .z()
            .unwrap();
        result
    }
}