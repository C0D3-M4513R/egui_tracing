use egui_tracing::egui;
#[cfg(target_arch = "wasm32")]
use egui_tracing::tracing_subscriber;
#[cfg(target_arch = "wasm32")]
use egui_tracing::tracing_subscriber::layer::SubscriberExt;
#[cfg(target_arch = "wasm32")]
use egui_tracing::tracing_subscriber::util::SubscriberInitExt;

#[cfg(target_arch = "wasm32")]
fn main() {
    let collector = egui_tracing::EventCollector::default();
    tracing_subscriber::registry()
        .with(collector.clone())
        .init();

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "eframe-canvas",
                web_options,
                Box::new(|_cc| Box::new(MyApp::new(egui_tracing::Logs::new(collector)))),
            )
            .await
            .expect("failed to start eframe");
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}

pub struct MyApp {
    collector: egui_tracing::Logs,
}

impl MyApp {
    #[cfg(target_arch = "wasm32")]
    fn new(collector: egui_tracing::Logs) -> Self {
        Self { collector }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&self.collector)
        });
    }
}
