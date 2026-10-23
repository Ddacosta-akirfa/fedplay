//! View de playlists

use gtk4::prelude::*;

/// View de playlists
pub struct PlaylistsView {
    pub widget: gtk4::Box,
}

impl PlaylistsView {
    /// Cria uma nova view de playlists
    pub fn new() -> Self {
        let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        box_.set_vexpand(true);

        let label = gtk4::Label::new(Some("Playlists (em desenvolvimento)"));
        label.add_css_class("title-2");
        label.set_vexpand(true);
        label.set_valign(gtk4::Align::Center);

        box_.append(&label);

        Self { widget: box_ }
    }
}
