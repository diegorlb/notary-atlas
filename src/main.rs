mod app;

use eframe::{NativeOptions, Result, egui::ViewportBuilder, run_native};

use crate::app::NotaryAtlas;

const APP_NAME: &str = "Atlas de Escribanos";

fn main() -> Result {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    run_native(
        APP_NAME,
        options,
        Box::new(|_| Ok(Box::new(NotaryAtlas::new()))),
    )
}
