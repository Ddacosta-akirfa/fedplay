//! Player bar persistente do Fedplay

use fedplay_audio::GStreamerPlayer;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Player bar na parte inferior da janela
pub struct PlayerBar {
    pub widget: gtk4::Box,
}

impl PlayerBar {
    /// Cria uma nova player bar
    pub fn new(player: Rc<RefCell<GStreamerPlayer>>) -> Self {
        let bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        bar.add_css_class("player-bar");
        bar.set_margin_start(16);
        bar.set_margin_end(16);
        bar.set_margin_top(8);
        bar.set_margin_bottom(8);

        // Coluna esquerda: capa + info
        let info_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        info_box.set_hexpand(true);
        info_box.set_halign(gtk4::Align::Start);

        let cover = gtk4::Image::from_icon_name("audio-x-generic-symbolic");
        cover.set_pixel_size(50);
        cover.add_css_class("cover");

        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        text_box.set_valign(gtk4::Align::Center);

        let title_label = gtk4::Label::new(Some("Fedora Soundscapes"));
        title_label.add_css_class("body-medium");
        title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title_label.set_halign(gtk4::Align::Start);

        let artist_label = gtk4::Label::new(Some("Libadwaita Orchestra"));
        artist_label.add_css_class("dim-label");
        artist_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        artist_label.set_halign(gtk4::Align::Start);

        text_box.append(&title_label);
        text_box.append(&artist_label);

        info_box.append(&cover);
        info_box.append(&text_box);

        // Coluna central: controles
        let controls_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        controls_box.set_halign(gtk4::Align::Center);

        let buttons_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        buttons_box.set_halign(gtk4::Align::Center);

        let prev_button = gtk4::Button::from_icon_name("media-skip-backward-symbolic");
        prev_button.add_css_class("control-button");

        let play_button = gtk4::Button::from_icon_name("play-arrow-symbolic");
        play_button.add_css_class("play-button");

        let next_button = gtk4::Button::from_icon_name("media-skip-forward-symbolic");
        next_button.add_css_class("control-button");

        let player_play = player.clone();
        play_button.connect_clicked(move |_| {
            let _ = player_play.borrow().play();
        });

        buttons_box.append(&prev_button);
        buttons_box.append(&play_button);
        buttons_box.append(&next_button);

        // Slider de progresso
        let progress_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        progress_box.set_halign(gtk4::Align::Center);

        let current_time = gtk4::Label::new(Some("1:20"));
        current_time.add_css_class("numeric-mono");

        let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        scale.set_draw_value(false);
        scale.set_size_request(300, -1);

        let total_time = gtk4::Label::new(Some("4:04"));
        total_time.add_css_class("numeric-mono");

        progress_box.append(&current_time);
        progress_box.append(&scale);
        progress_box.append(&total_time);

        controls_box.append(&buttons_box);
        controls_box.append(&progress_box);

        // Coluna direita: ações
        let actions_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        actions_box.set_halign(gtk4::Align::End);
        actions_box.set_hexpand(true);

        let volume_button = gtk4::Button::from_icon_name("volume-up-symbolic");
        volume_button.add_css_class("control-button");

        let queue_button = gtk4::Button::from_icon_name("queue-music-symbolic");
        queue_button.add_css_class("control-button");

        let edit_button = gtk4::Button::from_icon_name("edit-note-symbolic");
        edit_button.add_css_class("control-button");

        actions_box.append(&volume_button);
        actions_box.append(&queue_button);
        actions_box.append(&edit_button);

        bar.append(&info_box);
        bar.append(&controls_box);
        bar.append(&actions_box);

        Self { widget: bar }
    }
}
