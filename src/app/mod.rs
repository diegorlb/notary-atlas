use std::sync::mpsc::{Receiver, channel};

use eframe::{
    App, Frame,
    egui::{Context, Key, Ui},
};

pub enum State {
    Loading,
    Loaded,
    Failed,
}

pub enum Payload {}

pub struct NotaryAtlas {
    state: State,

    receiver: Receiver<Payload>,
}

impl NotaryAtlas {
    pub fn new() -> Self {
        let (_, receiver) = channel();

        Self {
            state: State::Loading,

            receiver,
        }
    }

    fn render_loading(&mut self, ui: &mut Ui) {}

    fn render_loaded(&mut self, ui: &mut Ui) {}

    fn render_failed(&mut self, ui: &mut Ui) {}
}

impl App for NotaryAtlas {
    fn ui(&mut self, ui: &mut Ui, _: &mut Frame) {
        match self.state {
            State::Loading => self.render_loading(ui),

            State::Loaded => self.render_loaded(ui),

            State::Failed => self.render_failed(ui),
        }
    }

    fn logic(&mut self, ctx: &Context, _: &mut Frame) {
        ctx.input(|input| {
            if input.key_pressed(Key::Plus) {
                // TODO: implement zoom-in
            }

            if input.key_pressed(Key::Minus) {
                // TODO: implement zoom-out
            }
        });
    }
}
