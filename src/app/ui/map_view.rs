use eframe::egui::{Align2, Area, CentralPanel, Frame, Id, Response, RichText, Ui, Vec2, Widget};
use walkers::{Map, Plugin, lat_lon};

use crate::{
    app::{MapData, ui::map_marker::MapMarker},
    domain::{GeocodedNotary, NotariesGroupedIndices},
};

pub enum MapAction {
    None,
    SaveScreenshot,
}

pub struct MapView<'a> {
    pub data: &'a [GeocodedNotary],
    pub groups: &'a [NotariesGroupedIndices],
    pub selected_group: &'a mut Option<usize>,

    pub is_capturing: bool,
    pub map_action: &'a mut MapAction,

    pub map_data: &'a mut MapData,
}

impl Widget for MapView<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        CentralPanel::default()
            .show(ui, |ui| {
                let map_response = ui.allocate_ui(ui.available_size(), |ui| {
                    ui.add(
                        Map::new(
                            Some(&mut self.map_data.tiles),
                            &mut self.map_data.memory,
                            lat_lon(-34.6037, -58.3816),
                        )
                        .double_click_to_zoom(true)
                        .with_plugin(MapViewPlugin {
                            data: self.data,
                            groups: self.groups,
                            selected_group: self.selected_group,
                        }),
                    );
                });

                self.map_data.rect = Some(map_response.response.rect);

                if !self.is_capturing {
                    Area::new(Id::unique("map_controls"))
                        .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-16.0, -16.0))
                        .show(ui.ctx(), |ui| {
                            Frame::default().show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    if ui.button(RichText::new("+").size(24.0)).clicked() {
                                        self.map_data.zoom_in();
                                    }

                                    if ui.button(RichText::new("-").size(24.0)).clicked() {
                                        self.map_data.zoom_out();
                                    }

                                    ui.separator();

                                    if ui.button(RichText::new("Guardar")).clicked() {
                                        *self.map_action = MapAction::SaveScreenshot;
                                    }
                                });
                            });
                        });
                }
            })
            .response
    }
}

struct MapViewPlugin<'a> {
    pub data: &'a [GeocodedNotary],
    pub groups: &'a [NotariesGroupedIndices],
    pub selected_group: &'a mut Option<usize>,
}

impl Plugin for MapViewPlugin<'_> {
    fn run(
        self: Box<Self>,
        ui: &mut Ui,
        response: &Response,
        projector: &walkers::Projector,
        _: &walkers::MapMemory,
    ) {
        let mut any_marker_clicked = false;

        let cull_rect = response.rect.shrink(10.0);

        for (index, grouped_notaries) in self.groups.iter().enumerate() {
            let location = &grouped_notaries.location;
            let screen_position = projector
                .project(lat_lon(location.lat, location.lon))
                .to_pos2();

            if !cull_rect.contains(screen_position) {
                continue;
            }

            let indices = &grouped_notaries.indices;
            let marker = MapMarker {
                index,
                data: self.data,
                indices,
                location,
                is_selected: *self.selected_group == Some(index),
            };

            let is_clicked = marker.draw(ui.ctx(), response, ui.painter(), screen_position);
            if is_clicked {
                *self.selected_group = Some(marker.index);
                any_marker_clicked = true;
            }
        }

        if !any_marker_clicked
            && response.hovered()
            && ui.input(|input| input.pointer.primary_clicked())
        {
            *self.selected_group = None;
        }
    }
}
