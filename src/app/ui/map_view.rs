use eframe::egui::{
    Align2, Area, Button, CentralPanel, Color32, CursorIcon, Frame, Id, Response, RichText, Ui,
    Vec2, Widget,
};
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
                            Frame::default()
                                .fill(Color32::from_black_alpha(180))
                                .corner_radius(8.0)
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        let button_size = Vec2::new(20.0, 20.0);

                                        let zoom_in_button = ui
                                            .add_sized(
                                                button_size,
                                                Button::new(RichText::new("+").size(20.0)),
                                            )
                                            .on_hover_text("Acercar")
                                            .on_hover_cursor(CursorIcon::PointingHand);

                                        ui.add_space(4.0);

                                        let zoom_out_button = ui
                                            .add_sized(
                                                button_size,
                                                Button::new(RichText::new("-").size(20.0)),
                                            )
                                            .on_hover_text("Alejar")
                                            .on_hover_cursor(CursorIcon::PointingHand);

                                        ui.add_space(4.0);

                                        let center_map_button = ui
                                            .add_sized(
                                                button_size,
                                                Button::new(RichText::new("⌂").size(18.0)),
                                            )
                                            .on_hover_text("Centrar mapa")
                                            .on_hover_cursor(CursorIcon::PointingHand);

                                        ui.separator();

                                        let save_map_button = ui
                                            .button(RichText::new("Guardar").size(14.0))
                                            .on_hover_text("Guardar captura del mapa")
                                            .on_hover_cursor(CursorIcon::PointingHand);

                                        if zoom_in_button.clicked() {
                                            self.map_data.zoom_in();
                                        }

                                        if zoom_out_button.clicked() {
                                            self.map_data.zoom_out();
                                        }

                                        if center_map_button.clicked() {
                                            self.map_data.center_at(-34.6037, -58.3816);
                                            let _ = self.map_data.memory.set_zoom(8.0);
                                        }

                                        if save_map_button.clicked() {
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
                *self.selected_group = if *self.selected_group == Some(marker.index) {
                    None
                } else {
                    Some(marker.index)
                };

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
