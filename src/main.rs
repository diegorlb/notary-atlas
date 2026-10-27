mod app;
mod domain;
mod services;
mod utils;

use eframe::{NativeOptions, Result, egui::ViewportBuilder, run_native};
use tokio::runtime::Runtime;

use crate::app::NotaryAtlas;

const APP_NAME: &str = "Atlas de Escribanos";

fn main() -> Result {
    env_logger::init();

    let runtime = Runtime::new().expect("");
    let handle = runtime.handle().clone();

    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    run_native(
        APP_NAME,
        options,
        Box::new(|ctx| Ok(Box::new(NotaryAtlas::new(ctx, &handle)))),
    )
}
