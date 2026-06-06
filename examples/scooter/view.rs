use buoyant::{match_view, view::prelude::*};

use crate::state::State;

use super::colour;

pub mod home;
pub mod info;
pub mod locked;
pub mod settings;

use super::state::Page;

#[must_use]
pub fn root_view(state: &State) -> impl View<colour::ColorFormat, State> + use<> {
    match_view!(state.page, {
        Page::Home => home::view(state),
        Page::Settings => EmptyView,
        Page::Info => info::view(state),
    })
    .padding(Edges::All, 5)
    .background_color(colour::BACKGROUND, RoundedRectangle::new(8))
}
