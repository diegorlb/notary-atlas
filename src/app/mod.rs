use eframe::{
    App, Frame,
    egui::{Color32, RichText, Ui},
};

pub struct NotaryAtlas {}

impl NotaryAtlas {
    pub const fn new() -> Self {
        Self {}
    }
}

impl App for NotaryAtlas {
    fn ui(&mut self, ui: &mut Ui, _: &mut Frame) {
        ui.label(RichText::new("Notary Atlas").color(Color32::WHITE));
    }
}
