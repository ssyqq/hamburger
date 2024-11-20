use eframe::egui;
use tracing::info;

mod chat;
mod llm;
mod ui;

use ui::App;

fn main() -> Result<(), eframe::Error> {
    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!("Starting LLM Client...");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    info!("Initializing GUI...");

    eframe::run_native(
        "LLM Client",
        options,
        Box::new(|cc| {
            info!("Creating application instance...");
            match App::new(cc) {
                Ok(app) => Ok(Box::new(app)),
                Err(e) => {
                    panic!("Failed to create app: {}", e);
                }
            }
        }),
    )
}
