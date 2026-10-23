//! View da biblioteca de músicas

use gtk4::prelude::*;

/// View da biblioteca de músicas
pub struct LibraryView {
    pub widget: gtk4::ScrolledWindow,
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
        list_box.set_selection_mode(gtk4::SelectionMode::None);

        // Faixas de exemplo (mock data)
        let tracks = vec![
            (
                "Midnight City",
                "M83 • Hurry Up, We're Dreaming",
                "4:04",
                "FLAC",
            ),
            ("Resonance", "HOME • Odyssey", "3:32", ""),
            (
                "Dayvan Cowboy",
                "Boards of Canada • The Campfire Headphase",
                "5:00",
                "",
            ),
            (
                "Instant Crush",
                "Daft Punk ft. Julian Casablancas • RAM",
                "5:37",
                "Hi-Res",
            ),
            (
                "An Ending (Ascent)",
                "Brian Eno • Apollo: Atmospheres",
                "4:26",
                "",
            ),
            ("Windowlicker", "Aphex Twin • Windowlicker EP", "6:07", ""),
        ];

        for (title, artist, duration, badge) in tracks {
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

            // Badge (se houver)
            let right_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            right_box.set_valign(gtk4::Align::Center);

            if !badge.is_empty() {
                let badge_label = gtk4::Label::new(Some(badge));
                badge_label.add_css_class("badge");
                if badge == "Hi-Res" {
                    badge_label.add_css_class("badge-hires");
                }
                right_box.append(&badge_label);
            }

            // Duração
            let duration_label = gtk4::Label::new(Some(duration));
            duration_label.add_css_class("numeric-mono");
            right_box.append(&duration_label);

            // Menu button
            let menu_button = gtk4::MenuButton::new();
            menu_button.set_icon_name("more-vert-symbolic");
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

        Self { widget: scrolled }
    }
}
