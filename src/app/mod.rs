mod ui;

use std::{
    sync::{
        Arc,
        mpsc::{Receiver, channel},
    },
    time::Instant,
};

use eframe::{
    App, CreationContext, Frame,
    egui::{Align2, Area, CentralPanel, Color32, Context, Id, Key, Rect, RichText, Ui, Vec2},
};
use tokio::{
    runtime::Handle,
    sync::mpsc::{UnboundedSender, unbounded_channel},
};
use walkers::{HttpTiles, MapMemory, sources::OpenStreetMap};

use crate::{
    app::ui::{DebugPanel, FilterBar, MapView, Sidebar},
    domain::{Filters, GeocodedNotary, NotariesGroupedIndices, apply_filters, group_notaries},
    services::Geocoder,
};

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
    pub rect: Option<Rect>,
    pub tiles: HttpTiles,
    pub memory: MapMemory,
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
}

pub struct NotaryAtlas {
    state: State,
    debug: bool,

    current_filters: Filters,
    applied_filters: Filters,
    last_edit: Instant,

    selected_group: Option<usize>,

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
            let mut geocoder = Geocoder::default();
            geocoder.load_caches().await;

            let mut cached_geocoded_notaries = Arc::default();

            while let Some(command) = worker_rx.recv().await {
                let payload = match command {
                    Command::LoadInitialData | Command::RefreshData => {
                        match geocoder.geocode().await {
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
            && self.last_edit.elapsed().as_millis() >= 150
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

        ui.add(MapView {
            data,
            groups,
            selected_group: &mut self.selected_group,
            map_data: &mut self.map_data,
        });
    }

    fn render_failed(&self, ui: &mut Ui) {
        let State::Failed(error_message) = &self.state else {
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
                    });
                });
        });
    }

    fn read_receiver(&mut self) {
        let Ok(payload) = self.ui_rx.try_recv() else {
            return;
        };

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

impl App for NotaryAtlas {
    fn ui(&mut self, ui: &mut Ui, _: &mut Frame) {
        match self.state {
            State::Loading => self.render_loading(ui),
            State::Loaded { .. } => self.render_loaded(ui),
            State::Failed(_) => self.render_failed(ui),
        }
    }

    fn logic(&mut self, ctx: &Context, _: &mut Frame) {
        ctx.input(|input| {
            if input.key_pressed(Key::F3) {
                self.debug = !self.debug;
            }

            if input.key_pressed(Key::Escape) {
                // TODO: implement quit on escape
            }

            if input.key_pressed(Key::Plus) {
                let _ = self.map_data.memory.zoom_in();
            }

            if input.key_pressed(Key::Minus) {
                let _ = self.map_data.memory.zoom_out();
            }
        });

        self.read_receiver();
    }
}
