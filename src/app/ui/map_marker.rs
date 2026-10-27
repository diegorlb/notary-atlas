use eframe::egui::{
    Align, Color32, Context, CursorIcon, Grid, Layout, Painter, Pos2, Rect, Response, RichText,
    ScrollArea, Stroke, Ui, Vec2, Window,
};

use crate::domain::{Location, Notary, Precision};

const MARKER_RADIUS: f32 = 6.0;
const MARKER_SIZE: Vec2 = Vec2::splat(2.0 * MARKER_RADIUS + 4.0);
const POSITION_OFFSET: Vec2 = Vec2::new(MARKER_RADIUS + 4.0, MARKER_RADIUS + 4.0);

pub struct MapMarker<'a> {
    pub index: usize,

    pub location: &'a Location,
    pub notaries: &'a [Notary],

    pub is_selected: bool,
}

impl MapMarker<'_> {
    pub fn draw(
        &self,
        ctx: &Context,
        response: &Response,
        painter: &Painter,
        screen_position: Pos2,
    ) -> bool {
        let marker_rect = Rect::from_center_size(screen_position, MARKER_SIZE);

        let is_hovered = response
            .hover_pos()
            .is_some_and(|mouse_position| marker_rect.contains(mouse_position));

        let is_clicked = is_hovered && ctx.input(|input| input.pointer.primary_clicked());

        if is_hovered {
            ctx.set_cursor_icon(CursorIcon::PointingHand);
        }

        if self.is_selected {
            let position = screen_position + POSITION_OFFSET;

            Window::new(format!("address_marker_id_{}", self.index))
                .id(response.id.with(("address_toggled_window", self.index)))
                .fixed_pos(position)
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| self.render_tooltip(ui));
        }

        let base_color = match self.location.precision {
            Precision::Exact => Color32::from_rgb(230, 57, 70),
            Precision::Centroid => Color32::from_rgb(150, 150, 150),
        };

        let fill_color = match (self.is_selected, is_hovered) {
            (true, _) => Color32::from_rgb(29, 110, 220),
            (false, true) => Color32::from_rgb(255, 120, 130),
            (false, false) => base_color,
        };

        painter.circle_filled(screen_position, MARKER_RADIUS, fill_color);

        painter.circle_stroke(
            screen_position,
            MARKER_RADIUS,
            Stroke::new(1.5, Color32::WHITE),
        );

        is_clicked
    }

    fn render_tooltip(&self, ui: &mut Ui) {
        ui.label(RichText::new(&self.location.formatted).size(22.0).strong());

        if self.location.precision == Precision::Centroid {
            ui.label(
                RichText::new("Ubicación aproximada")
                    .italics()
                    .color(Color32::from_rgb(255, 165, 0)),
            );
        }

        ui.add_space(8.0);

        ui.label(
            RichText::new(format!(
                "Hay {} escribano{} en esta dirección",
                self.notaries.len(),
                if self.notaries.len() == 1 { "" } else { "s" }
            ))
            .strong()
            .color(Color32::LIGHT_GRAY),
        );

        ui.add_space(4.0);
        ui.separator();

        ScrollArea::vertical()
            .max_height(350.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (index, notary) in self.notaries.iter().enumerate() {
                    if index > 0 {
                        ui.add_space(8.0);
                    }

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{} {}",
                                    notary.first_name, notary.last_name
                                ))
                                .strong()
                                .size(16.0),
                            );

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format!("Reg: {}", notary.registry))
                                        .strong()
                                        .color(Color32::from_rgb(100, 100, 255)),
                                );
                            });
                        });

                        ui.add_space(4.0);
                        ui.label(
                            RichText::from(format!("{}, {}", notary.address, notary.city))
                                .small()
                                .color(Color32::LIGHT_GRAY),
                        );

                        ui.add_space(6.0);
                        Grid::new(format!("contact_grid_{index}"))
                            .num_columns(2)
                            .spacing([12.0, 4.0])
                            .show(ui, |ui| {
                                if let Some(telephone) = &notary.telephones.primary {
                                    ui.label(RichText::new("Tel:").color(Color32::LIGHT_GRAY));
                                    ui.label(RichText::new(telephone));
                                    ui.end_row();
                                }

                                if let Some(telephone) = &notary.telephones.secondary {
                                    ui.label(RichText::new("Alt:").color(Color32::LIGHT_GRAY));
                                    ui.label(RichText::new(telephone));
                                    ui.end_row();
                                }

                                if let Some(email) = &notary.emails.public {
                                    ui.label(
                                        RichText::new("Email Pub.:").color(Color32::LIGHT_GRAY),
                                    );

                                    ui.label(RichText::new(email));

                                    ui.end_row();
                                }

                                if let Some(email) = &notary.emails.alternative {
                                    ui.label(
                                        RichText::new("Email Alt.:").color(Color32::LIGHT_GRAY),
                                    );

                                    ui.label(RichText::new(email));

                                    ui.end_row();
                                }

                                if let Some(email) = &notary.emails.institutional {
                                    ui.label(
                                        RichText::new("Email Inst.:").color(Color32::LIGHT_GRAY),
                                    );

                                    ui.label(RichText::new(email));

                                    ui.end_row();
                                }
                            });
                    });
                }
            });
    }
}
