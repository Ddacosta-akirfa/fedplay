// fedplay-ui/src/views/playlists_view.rs
use gtk4::prelude::*;

pub struct PlaylistsView {
    pub widget: gtk4::Box,
}

impl PlaylistsView {
    pub fn new() -> Self {
        let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let label = gtk4::Label::new(Some("Playlists (em desenvolvimento)"));
        box_.append(&label);
        Self { widget: box_ }
    }
}
