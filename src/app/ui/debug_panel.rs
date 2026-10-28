use eframe::egui::{Align2, Response, RichText, Ui, Vec2, Widget, Window};

use crate::domain::NotariesGroupedIndices;

const FONT_SIZE: f32 = 12.0;

pub struct DebugPanel<'a> {
    pub total: usize,
    pub groups: &'a [NotariesGroupedIndices],

    pub debug: &'a mut bool,
}

impl Widget for DebugPanel<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.allocate_ui(ui.available_size(), |ui| {
            if !*self.debug {
                return;
            }

            Window::new("Información Extra")
                .anchor(Align2::LEFT_BOTTOM, Vec2::new(12.0, -12.0))
                .show(ui.ctx(), |ui| {
                    let fps = 1.0 / ui.input(|input| input.unstable_dt).max(0.0001);
                    let total_addresses = self.groups.len();
                    let total_filtered_notaries = self
                        .groups
                        .iter()
                        .map(|group| group.indices.len())
                        .sum::<usize>();

                    ui.label(RichText::new(format!("FPS: {fps:.0}")).size(FONT_SIZE));

                    ui.label(
                        RichText::new(format!("Total Escribanos: {}", self.total)).size(FONT_SIZE),
                    );

                    ui.label(
                        RichText::new(format!("Escribanos Filtrados: {total_filtered_notaries}"))
                            .size(FONT_SIZE),
                    );

                    ui.label(
                        RichText::new(format!("Direcciones Agrupadas: {total_addresses}"))
                            .size(FONT_SIZE),
                    );

                    if let Some(mouse_posistion) = ui.ctx().pointer_hover_pos() {
                        ui.label(
                            RichText::new(format!(
                                "Mouse: [{:.0}, {:.0}]",
                                mouse_posistion.x, mouse_posistion.y
                            ))
                            .size(FONT_SIZE),
                        );
                    } else {
                        ui.label(RichText::new("Mouse: Fuera de los límites").size(FONT_SIZE));
                    }
                });
        })
        .response
    }
}
