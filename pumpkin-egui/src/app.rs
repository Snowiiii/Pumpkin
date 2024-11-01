use std::{
    io,
    sync::{Arc, LazyLock},
};

use egui::mutex::Mutex;
// Simplest workaround for keyboar invocation
//#[cfg(target_os = "android")]
use j4rs::{InvocationArg, Jvm};
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

                //#[cfg(target_os = "android")]
                {
                    let textedit = ui.add_sized(
                        ui.available_size() - egui::vec2(43.0, 0.0),
                        egui::TextEdit::singleline(&mut self.command),
                    );
    
                    if textedit.gained_focus() {
                        let jvm = Jvm::attach_thread().unwrap();
                        let instance = jvm.create_instance(
                            "pumpkin_egui_android.MainActivity",
                            InvocationArg::empty()
                        ).unwrap();
                        jvm.invoke(
                            &instance,
                            "openKeyboard",
                            InvocationArg::empty()
                        ).unwrap();
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
