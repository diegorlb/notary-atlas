mod ui;

use std::{
    sync::{
        Arc,
        mpsc::{Receiver, channel},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use eframe::{
    App, CreationContext, Frame,
    egui::{Align2, Area, CentralPanel, Color32, Context, Id, Key, Rect, RichText, Ui, Vec2},
};
use image::{ImageBuffer, Rgba, imageops};
use tokio::{
    runtime::Handle,
    sync::mpsc::{UnboundedSender, unbounded_channel},
};
use walkers::{HttpTiles, MapMemory, lat_lon, sources::OpenStreetMap};

use crate::{
    app::ui::{DebugPanel, FilterBar, MapAction, MapView, Sidebar},
    domain::{Filters, GeocodedNotary, NotariesGroupedIndices, apply_filters, group_notaries},
    services::Geocoder,
};

#[derive(Clone)]
pub enum State {
    Loading,

    Loaded {
        data: Arc<Vec<GeocodedNotary>>,
        groups: Vec<NotariesGroupedIndices>,
        unfiltered: Arc<Vec<NotariesGroupedIndices>>,
    },

    Failed(String),
}

pub enum Payload {
    NotariesLoaded {
        data: Arc<Vec<GeocodedNotary>>,
        groups: Vec<NotariesGroupedIndices>,
        unfiltered: Arc<Vec<NotariesGroupedIndices>>,
    },

    FilteredResults(Vec<NotariesGroupedIndices>),
    ClearFilters,

    NotariesFailed(String),
}

pub enum Command {
    LoadInitialData,
    RefreshData,
    ApplyFilters(Filters),
}

pub struct MapData {
    rect: Option<Rect>,
    tiles: HttpTiles,
    memory: MapMemory,
}

impl MapData {
    pub fn new(ctx: &CreationContext<'_>) -> Self {
        let mut memory = MapMemory::default();
        let _ = memory.set_zoom(8.0);

        let tiles = HttpTiles::new(OpenStreetMap, ctx.egui_ctx.clone());

        Self {
            rect: None,
            tiles,
            memory,
        }
    }

    pub fn zoom_in(&mut self) {
        let _ = self.memory.zoom_in();
    }

    pub fn zoom_out(&mut self) {
        let _ = self.memory.zoom_out();
    }

    pub fn center_at(&mut self, lat: f64, lon: f64) {
        self.memory.center_at(lat_lon(lat, lon));
    }

    pub fn move_map(&mut self, delta_x: f64, delta_y: f64) {
        if delta_x == 0.0 && delta_y == 0.0 {
            return;
        }

        let zoom = self.memory.zoom();
        let scale = f64::powf(2.0, zoom);
        let speed = 10.0 / scale;

        let current_position = self
            .memory
            .detached()
            .unwrap_or_else(|| lat_lon(-34.6037, -58.3816));

        let (lon, lat) = current_position.x_y();

        self.memory.center_at(lat_lon(
            f64::mul_add(delta_y, speed, lat),
            f64::mul_add(delta_x, speed, lon),
        ));
    }
}

pub struct NotaryAtlas {
    state: State,
    debug: bool,

    current_filters: Filters,
    applied_filters: Filters,
    last_edit: Instant,

    selected_group: Option<usize>,

    screenshot_pending: bool,

    ui_rx: Receiver<Payload>,
    worker_tx: UnboundedSender<Command>,

    map_data: MapData,
}

impl NotaryAtlas {
    pub fn new(ctx: &CreationContext<'_>, handle: &Handle) -> Self {
        let (ui_tx, ui_rx) = channel();
        let (worker_tx, mut worker_rx) = unbounded_channel();

        let egui_ctx = ctx.egui_ctx.clone();
        handle.spawn(async move {
            let mut geocoder = Geocoder::new();
            geocoder.load_caches().await;

            let mut cached_geocoded_notaries = Arc::default();

            while let Some(command) = worker_rx.recv().await {
                let payload = match command {
                    Command::LoadInitialData | Command::RefreshData => {
                        let force_refresh = matches!(command, Command::RefreshData);
                        match geocoder.geocode(force_refresh).await {
                            Ok(notaries) => {
                                let data = Arc::new(notaries);
                                let groups = group_notaries(&data, 0..data.len());
                                let unfiltered = Arc::new(groups.clone());

                                cached_geocoded_notaries = Arc::clone(&data);

                                Payload::NotariesLoaded {
                                    data,
                                    groups,
                                    unfiltered,
                                }
                            }

                            Err(error) => Payload::NotariesFailed(error.to_string()),
                        }
                    }

                    Command::ApplyFilters(filters) => {
                        if filters.is_empty() {
                            Payload::ClearFilters
                        } else {
                            let filtered = apply_filters(&cached_geocoded_notaries, &filters);

                            Payload::FilteredResults(filtered)
                        }
                    }
                };

                let _ = ui_tx.send(payload);

                egui_ctx.request_repaint();
            }
        });

        let _ = worker_tx.send(Command::LoadInitialData);

        Self {
            state: State::Loading,
            debug: false,

            current_filters: Filters::default(),
            applied_filters: Filters::default(),
            last_edit: Instant::now(),

            selected_group: None,

            screenshot_pending: false,

            ui_rx,
            worker_tx,

            map_data: MapData::new(ctx),
        }
    }

    #[expect(clippy::unused_self)]
    fn render_loading(&self, ui: &mut Ui) {
        CentralPanel::default().show(ui, |ui| {
            Area::new(Id::unique("loading_area"))
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.spinner();
                        ui.add_space(15.0);
                        ui.label(
                            RichText::new("Cargando y geocodificando escribanos...")
                                .size(18.0)
                                .color(Color32::LIGHT_GRAY),
                        );
                    });
                });
        });
    }

    fn render_loaded(&mut self, ui: &mut Ui) {
        let previous_filters = self.current_filters.clone();

        ui.add(FilterBar {
            filters: &mut self.current_filters,
        });

        if self.current_filters != previous_filters {
            self.last_edit = Instant::now();
        }

        if self.applied_filters != self.current_filters
            && self.last_edit.elapsed().as_millis() >= 400
        {
            self.applied_filters = self.current_filters.clone();
            let _ = self
                .worker_tx
                .send(Command::ApplyFilters(self.current_filters.clone()));
        }

        let State::Loaded { data, groups, .. } = &self.state else {
            return;
        };

        ui.add(DebugPanel {
            total: data.len(),
            groups,
            debug: &mut self.debug,
        });

        ui.add(Sidebar {
            data,
            groups,
            selected_group: &mut self.selected_group,
            map_data: &mut self.map_data,
        });

        let mut map_action = MapAction::None;
        ui.add(MapView {
            data,
            groups,
            selected_group: &mut self.selected_group,

            is_capturing: self.screenshot_pending,
            map_action: &mut map_action,

            map_data: &mut self.map_data,
        });

        self.handle_map_action(ui.ctx(), &map_action);
    }

    fn render_failed(&mut self, ui: &mut Ui) {
        let State::Failed(error_message) = self.state.clone() else {
            return;
        };

        CentralPanel::default().show(ui, |ui| {
            Area::new(Id::unique("loading_area"))
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(error_message)
                                .strong()
                                .color(Color32::DARK_RED),
                        );

                        ui.add_space(15.0);

                        if ui.button("Reintentar").clicked() {
                            self.state = State::Loading;
                            let _ = self.worker_tx.send(Command::RefreshData);
                        }
                    });
                });
        });
    }

    fn read_receiver(&mut self) {
        while let Ok(payload) = self.ui_rx.try_recv() {
            match payload {
                Payload::NotariesLoaded {
                    data,
                    groups,
                    unfiltered,
                } => {
                    self.state = State::Loaded {
                        data,
                        groups,
                        unfiltered,
                    };

                    self.selected_group = None;
                }

                Payload::FilteredResults(groups) => {
                    if let State::Loaded {
                        data, unfiltered, ..
                    } = &mut self.state
                    {
                        self.state = State::Loaded {
                            data: Arc::clone(data),
                            groups,
                            unfiltered: Arc::clone(unfiltered),
                        };
                    }

                    self.selected_group = None;
                }

                Payload::ClearFilters => {
                    if let State::Loaded {
                        data, unfiltered, ..
                    } = &mut self.state
                    {
                        self.state = State::Loaded {
                            data: Arc::clone(data),
                            groups: (**unfiltered).clone(),
                            unfiltered: Arc::clone(unfiltered),
                        };
                    }

                    self.selected_group = None;
                }

                Payload::NotariesFailed(error) => self.state = State::Failed(error),
            }
        }
    }

    fn handle_map_action(&mut self, context: &Context, map_action: &MapAction) {
        if self.screenshot_pending {
            self.handle_map_screenshot(context);
            self.screenshot_pending = false;
        }

        match map_action {
            MapAction::None => {}

            MapAction::SaveScreenshot => {
                self.screenshot_pending = true;
                context.request_repaint();
            }
        }
    }

    fn handle_map_screenshot(&self, context: &Context) {
        let Some(rect) = self.map_data.rect else {
            return;
        };

        let ppp = context.pixels_per_point();

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        context.request_screenshot(move |screenshot| {
            let full_width = u32::try_from(screenshot.width()).unwrap_or_default();
            let full_height = u32::try_from(screenshot.height()).unwrap_or_default();

            let min_x = f32::max(0.0, rect.min.x * ppp) as u32;
            let min_y = f32::max(0.0, rect.min.y * ppp) as u32;

            let max_x = f32::min(full_width as f32, rect.max.x * ppp) as u32;
            let max_y = f32::max(full_height as f32, rect.max.y * ppp) as u32;

            let width = u32::saturating_sub(max_x, min_x);
            let height = u32::saturating_sub(max_y, min_y);

            if width == 0 || height == 0 {
                return;
            }

            let raw_image = screenshot.as_raw().to_vec();

            std::thread::spawn(move || {
                let buffer =
                    ImageBuffer::<Rgba<_>, _>::from_raw(full_width, full_height, raw_image);
                let Some(mut buffer) = buffer else {
                    return;
                };

                let cropped_buffer = imageops::crop(&mut buffer, min_x, min_y, width, height);
                let cropped_image = cropped_buffer.to_image();

                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let filename = format!("map_{timestamp}.png");

                match cropped_image.save(&filename) {
                    Ok(()) => {
                        log::info!("Map successfully saved as {filename}");
                    }

                    Err(error) => {
                        log::error!("Failed to save map snapshot: {error}");
                    }
                }
            });
        });
    }
}

impl App for NotaryAtlas {
    fn ui(&mut self, ui: &mut Ui, _: &mut Frame) {
        match self.state {
            State::Loading => self.render_loading(ui),
            State::Loaded { .. } => self.render_loaded(ui),
            State::Failed(_) => self.render_failed(ui),
        }
    }

    fn logic(&mut self, ctx: &Context, _: &mut Frame) {
        let wants_keyboard = ctx.egui_wants_keyboard_input();

        ctx.input(|input| {
            if input.key_pressed(Key::F3) {
                self.debug = !self.debug;
            }

            if !wants_keyboard {
                if input.key_pressed(Key::Plus) {
                    self.map_data.zoom_in();
                }

                if input.key_pressed(Key::Minus) {
                    self.map_data.zoom_out();
                }

                let (mut delta_x, mut delta_y) = (0.0, 0.0);

                if input.key_down(Key::ArrowUp) {
                    delta_y += 1.0;
                }

                if input.key_down(Key::ArrowDown) {
                    delta_y -= 1.0;
                }

                if input.key_down(Key::ArrowLeft) {
                    delta_x -= 1.0;
                }

                if input.key_down(Key::ArrowRight) {
                    delta_x += 1.0;
                }

                self.map_data.move_map(delta_x, delta_y);
            }
        });

        if !wants_keyboard {
            let is_panning = ctx.input(|input| {
                input.key_down(Key::ArrowUp)
                    || input.key_down(Key::ArrowDown)
                    || input.key_down(Key::ArrowLeft)
                    || input.key_down(Key::ArrowRight)
            });

            if is_panning {
                ctx.request_repaint();
            }
        }

        self.read_receiver();
    }
}
