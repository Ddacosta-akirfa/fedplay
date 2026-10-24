//! View da biblioteca de músicas

use fedplay_core::Track;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// View da biblioteca de músicas
pub struct LibraryView {
    pub widget: gtk4::ScrolledWindow,
    list_box: gtk4::ListBox,
    tracks: Rc<RefCell<Vec<Track>>>,
}

impl LibraryView {
    /// Cria uma nova view de biblioteca
    pub fn new() -> Self {
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled.set_vexpand(true);
        scrolled.add_css_class("view-surface");

        let list_box = gtk4::ListBox::new();
        list_box.add_css_class("boxed-list");
        list_box.set_margin_start(16);
        list_box.set_margin_end(16);
        list_box.set_margin_top(16);
        list_box.set_margin_bottom(16);
        list_box.set_selection_mode(gtk4::SelectionMode::Single);

        let tracks = Rc::new(RefCell::new(Vec::new()));

        // Faixas de exemplo (mock data)
        let mock_tracks = vec![
            (
                "Midnight City",
                "M83 • Hurry Up, We're Dreaming",
                "4:04",
                Some("FLAC"),
            ),
            ("Resonance", "HOME • Odyssey", "3:32", None),
            (
                "Dayvan Cowboy",
                "Boards of Canada • The Campfire Headphase",
                "5:00",
                None,
            ),
            (
                "Instant Crush",
                "Daft Punk ft. Julian Casablancas • RAM",
                "5:37",
                Some("Hi-Res"),
            ),
            (
                "An Ending (Ascent)",
                "Brian Eno • Apollo: Atmospheres",
                "4:26",
                None,
            ),
            ("Windowlicker", "Aphex Twin • Windowlicker EP", "6:07", None),
        ];

        for (i, (title, artist, duration, badge)) in mock_tracks.iter().enumerate() {
            let track = Track::new(
                i as u64 + 1,
                std::path::PathBuf::new(), // Placeholder
                fedplay_core::TrackMetadata {
                    title: Some(title.to_string()),
                    artist: Some(artist.split(" • ").next().unwrap_or("").to_string()),
                    album: Some(artist.split(" • ").nth(1).unwrap_or("").to_string()),
                    ..Default::default()
                },
            );
            tracks.borrow_mut().push(track);

            let row = gtk4::ListBoxRow::new();
            row.add_css_class("track-row");

            let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
            row_box.set_margin_start(12);
            row_box.set_margin_end(12);
            row_box.set_margin_top(8);
            row_box.set_margin_bottom(8);

            // Capa
            let cover = gtk4::Image::from_icon_name("audio-x-generic-symbolic");
            cover.set_pixel_size(48);
            cover.add_css_class("cover");

            // Texto
            let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
            text_box.set_vexpand(true);
            text_box.set_valign(gtk4::Align::Center);

            let title_label = gtk4::Label::new(Some(title));
            title_label.add_css_class("track-title");
            title_label.set_halign(gtk4::Align::Start);
            title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

            let artist_label = gtk4::Label::new(Some(artist));
            artist_label.add_css_class("track-artist");
            artist_label.set_halign(gtk4::Align::Start);
            artist_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

            text_box.append(&title_label);
            text_box.append(&artist_label);

            // Lado direito
            let right_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            right_box.set_valign(gtk4::Align::Center);

            if let Some(badge_text) = badge {
                let badge_label = gtk4::Label::new(Some(badge_text));
                badge_label.add_css_class("badge");
                if *badge_text == "Hi-Res" {
                    badge_label.add_css_class("badge-hires");
                }
                right_box.append(&badge_label);
            }

            let duration_label = gtk4::Label::new(Some(duration));
            duration_label.add_css_class("numeric-mono");
            right_box.append(&duration_label);

            let menu_button = gtk4::MenuButton::new();
            menu_button.set_icon_name("view-more-symbolic");
            menu_button.add_css_class("flat");
            right_box.append(&menu_button);

            row_box.append(&cover);
            row_box.append(&text_box);
            row_box.append(&right_box);

            row.set_child(Some(&row_box));
            list_box.append(&row);
        }

        // Storage card
        let storage_card = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        storage_card.add_css_class("storage-card");
        storage_card.set_margin_start(16);
        storage_card.set_margin_end(16);
        storage_card.set_margin_bottom(16);

        let storage_title = gtk4::Label::new(Some("Armazenamento Local"));
        storage_title.add_css_class("title-4");
        storage_title.set_halign(gtk4::Align::Start);

        let storage_info = gtk4::Label::new(Some("~/Música • 14.8 GB Lossless"));
        storage_info.add_css_class("dim-label");
        storage_info.set_halign(gtk4::Align::Start);

        let rescan_button = gtk4::Button::with_label("Reescanear");
        rescan_button.add_css_class("flat");
        rescan_button.set_halign(gtk4::Align::Start);

        storage_card.append(&storage_title);
        storage_card.append(&storage_info);
        storage_card.append(&rescan_button);

        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        content_box.append(&list_box);
        content_box.append(&storage_card);

        scrolled.set_child(Some(&content_box));

        Self {
            widget: scrolled,
            list_box,
            tracks,
        }
    }

    /// Retorna a faixa selecionada atualmente (se houver)
    pub fn get_selected_track(&self) -> Option<Track> {
        if let Some(row) = self.list_box.selected_row() {
            let index = row.index();
            let tracks = self.tracks.borrow();
            if (index as usize) < tracks.len() {
                return Some(tracks[index as usize].clone());
            }
        }
        None
    }

    /// Conecta um callback para quando uma faixa for ativada (duplo-clique ou Enter)
    pub fn connect_track_activated<F: Fn(&Track) + 'static>(&self, callback: F) {
        let tracks = self.tracks.clone();
        let callback = Rc::new(callback);

        self.list_box.connect_row_activated(move |_, row| {
            let index = row.index();
            let tracks = tracks.borrow();
            if (index as usize) < tracks.len() {
                callback(&tracks[index as usize]);
            }
        });
    }
}
