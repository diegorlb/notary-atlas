use eframe::egui::{
    Align, Color32, ComboBox, Frame, Layout, Panel, Response, RichText, TextEdit, Ui, Vec2, Widget,
};

use crate::domain::{Filters, Precision};

const FONT_SIZE: f32 = 18.0;
const FONT_COLOR: Color32 = Color32::WHITE;

pub struct FilterBar<'a> {
    pub filters: &'a mut Filters,
}

impl Widget for FilterBar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        Panel::top("filters")
            .frame(
                Frame::default()
                    .fill(Color32::from_gray(32))
                    .inner_margin(12.0),
            )
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(16.0, 0.0);

                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Nombre / Apellido")
                                .strong()
                                .size(FONT_SIZE)
                                .color(FONT_COLOR),
                        );

                        ui.add(
                            TextEdit::singleline(&mut self.filters.name)
                                .desired_width(180.0)
                                .hint_text("Ej. Juan Perez"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("N° Registro")
                                .strong()
                                .size(FONT_SIZE)
                                .color(FONT_COLOR),
                        );

                        ui.add(
                            TextEdit::singleline(&mut self.filters.registry)
                                .desired_width(180.0)
                                .hint_text("1234"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Dirección / Localidad")
                                .strong()
                                .size(FONT_SIZE)
                                .color(FONT_COLOR),
                        );

                        ui.add(
                            TextEdit::singleline(&mut self.filters.location)
                                .desired_width(180.0)
                                .hint_text("Ej. La Plata"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Ubicación")
                                .strong()
                                .size(FONT_SIZE)
                                .color(FONT_COLOR),
                        );

                        ComboBox::from_id_salt("precision_combo")
                            .width(180.0)
                            .selected_text(match self.filters.precision {
                                Some(Precision::Exact) => "Exacta",
                                Some(Precision::Centroid) => "Aproximada",

                                None => "Todos",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.filters.precision,
                                    Some(Precision::Exact),
                                    "Exacta",
                                );

                                ui.selectable_value(
                                    &mut self.filters.precision,
                                    Some(Precision::Centroid),
                                    "Aproximada",
                                );

                                ui.selectable_value(&mut self.filters.precision, None, "Todos");
                            });
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(RichText::new("Limpiar").size(FONT_SIZE))
                            .clicked()
                        {
                            *self.filters = Filters::default();
                        }
                    });
                })
            })
            .response
    }
}
