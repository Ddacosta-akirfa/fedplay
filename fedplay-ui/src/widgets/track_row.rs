//! Widget de linha de faixa reutilizável

use gtk4::prelude::*;

/// Widget reutilizável para uma faixa na lista
pub struct TrackRow {
    pub widget: gtk4::ListBoxRow,
}

impl TrackRow {
    /// Cria uma nova track row
    pub fn new(title: &str, artist: &str, duration: &str, badge: Option<&str>) -> Self {
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
            if badge_text == "Hi-Res" {
                badge_label.add_css_class("badge-hires");
            }
            right_box.append(&badge_label);
        }

        let duration_label = gtk4::Label::new(Some(duration));
        duration_label.add_css_class("numeric-mono");
        right_box.append(&duration_label);

        let menu_button = gtk4::MenuButton::new();
        menu_button.set_icon_name("more-vert-symbolic");
        menu_button.add_css_class("flat");
        right_box.append(&menu_button);

        row_box.append(&cover);
        row_box.append(&text_box);
        row_box.append(&right_box);

        row.set_child(Some(&row_box));

        Self { widget: row }
    }
}
