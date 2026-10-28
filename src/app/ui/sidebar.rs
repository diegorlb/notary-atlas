use eframe::egui::{Button, Color32, Panel, Response, RichText, ScrollArea, Ui, Widget};
use walkers::lat_lon;

use crate::{
    app::MapData,
    domain::{GeocodedNotary, NotariesGroupedIndices},
};

const MAX_DISPLAY: usize = 50;

pub struct Sidebar<'a> {
    pub data: &'a [GeocodedNotary],
    pub groups: &'a [NotariesGroupedIndices],
    pub selected_group: &'a mut Option<usize>,

    pub map_data: &'a mut MapData,
}

impl Widget for Sidebar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        Panel::left("notaries_sidebar")
            .default_size(280.0)
            .min_size(220.0)
            .show(ui, |ui| {
                let filtered_count: usize =
                    self.groups.iter().map(|group| group.indices.len()).sum();

                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("Escribanos ({filtered_count})"))
                        .strong()
                        .size(16.0),
                );
                ui.add_space(4.0);

                ui.separator();

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if self.groups.is_empty() {
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new("No se encontraron escribanos")
                                    .italics()
                                    .color(Color32::GRAY),
                            );

                            return;
                        }

                        let filtered = self.groups.iter().take(MAX_DISPLAY).enumerate();
                        for (group_index, group) in filtered {
                            ui.group(|ui| {
                                ui.set_min_width(ui.available_width());

                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(&group.location.formatted)
                                            .strong()
                                            .size(14.0),
                                    );

                                    ui.add_space(2.0);

                                    for &index in &group.indices {
                                        let notary = &self.data[index].notary;

                                        ui.label(
                                            RichText::new(format!(
                                                "- {}, {}",
                                                notary.last_name, notary.first_name
                                            ))
                                            .size(12.0),
                                        );
                                    }

                                    ui.add_space(6.0);

                                    let button = ui.add_sized(
                                        [ui.available_width(), 0.0],
                                        Button::new("Ver detalles"),
                                    );

                                    if button.clicked() {
                                        *self.selected_group = Some(group_index);

                                        self.map_data.memory.center_at(lat_lon(
                                            group.location.lat,
                                            group.location.lon,
                                        ));
                                    }
                                });
                            });
                        }

                        if self.groups.len() > MAX_DISPLAY {
                            ui.add_space(10.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "+ {} direcciones ocultas.",
                                        self.groups.len() - MAX_DISPLAY
                                    ))
                                    .small()
                                    .color(Color32::GRAY),
                                );

                                ui.label(
                                    RichText::new("Usá los filtros para refinar la búsqueda.")
                                        .small()
                                        .color(Color32::GRAY),
                                );
                            });
                            ui.add_space(10.0);
                        }
                    });
            })
            .response
    }
}
