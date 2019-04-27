use gtk::{prelude::*, Application, ApplicationWindow, Inhibit};
use relm::{Relm, Update, Widget};
use relm_derive::Msg;

#[derive(Msg)]
pub enum Msg {}

pub struct AppRoot {}

impl Update for AppRoot {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: Self::ModelParam) -> Self::Model {
        ()
    }

    fn update(&mut self, msg: Self::Msg) {}
}

impl Widget for AppRoot {
    type Root = ApplicationWindow;

    fn root(&self) -> Self::Root {
        unimplemented!()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        unimplemented!()
    }
}

fn main() {
    AppRoot::run(()).unwrap()
}
