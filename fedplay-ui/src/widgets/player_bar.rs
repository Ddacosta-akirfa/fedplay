//! Player bar persistente do Fedplay

use fedplay_audio::GStreamerPlayer;
use fedplay_core::Track;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Player bar na parte inferior da janela
pub struct PlayerBar {
    pub widget: gtk4::Box,
    title_label: gtk4::Label,
    artist_label: gtk4::Label,
    play_button: gtk4::Button,
    current_track: Rc<RefCell<Option<Track>>>,
}

impl PlayerBar {
    /// Cria uma nova player bar
    pub fn new(player: Rc<RefCell<GStreamerPlayer>>) -> Self {
        let current_track = Rc::new(RefCell::new(None));

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

        let title_label = gtk4::Label::new(Some("Nenhuma faixa selecionada"));
        title_label.add_css_class("body-medium");
        title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title_label.set_halign(gtk4::Align::Start);

        let artist_label = gtk4::Label::new(Some("Fedplay"));
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

        let play_button = gtk4::Button::from_icon_name("media-playback-start-symbolic");
        play_button.add_css_class("play-button");

        let next_button = gtk4::Button::from_icon_name("media-skip-forward-symbolic");
        next_button.add_css_class("control-button");

        let player_play = player.clone();
        let play_btn_clone = play_button.clone();
        play_button.connect_clicked(move |_| {
            let state = player_play.borrow().get_state();
            let _ = if matches!(state.playback, fedplay_core::PlaybackState::Playing) {
                play_btn_clone.set_icon_name("media-playback-start-symbolic");
                player_play.borrow().pause()
            } else {
                play_btn_clone.set_icon_name("media-playback-pause-symbolic");
                player_play.borrow().play()
            };
        });

        buttons_box.append(&prev_button);
        buttons_box.append(&play_button);
        buttons_box.append(&next_button);

        // Slider de progresso (placeholder visual por enquanto)
        let progress_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        progress_box.set_halign(gtk4::Align::Center);

        let current_time = gtk4::Label::new(Some("0:00"));
        current_time.add_css_class("numeric-mono");

        let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        scale.set_draw_value(false);
        scale.set_size_request(300, -1);

        let total_time = gtk4::Label::new(Some("0:00"));
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

        let volume_button = gtk4::Button::from_icon_name("audio-volume-high-symbolic");
        volume_button.add_css_class("control-button");

        actions_box.append(&volume_button);

        bar.append(&info_box);
        bar.append(&controls_box);
        bar.append(&actions_box);

        Self {
            widget: bar,
            title_label,
            artist_label,
            play_button,
            current_track: current_track.clone(),
        }
    }

    /// Obtém a faixa atualmente selecionada.
    pub fn get_current_track(&self) -> Option<Track> {
        self.current_track.borrow().clone()
    }

    // Adicione um método para obter o callback de edição:
    pub fn get_edit_callback(&self) -> Option<Rc<dyn Fn() + 'static>> {
        let track_ref = self.current_track.clone();

        Some(Rc::new(move || {
            if let Some(ref track) = *track_ref.borrow() {
                println!("Solicitando edição para: {}", track.display_title());
            }
        }))
    }

    /// Atualiza as informações da faixa atual.
    pub fn set_track(&self, track: &Track) {
        let title = track
            .metadata
            .title
            .as_deref()
            .unwrap_or("Título desconhecido");

        let artist = track
            .metadata
            .artist
            .as_deref()
            .unwrap_or("Artista desconhecido");

        self.title_label.set_text(title);
        self.artist_label.set_text(artist);

        *self.current_track.borrow_mut() = Some(track.clone());

        self.play_button
            .set_icon_name("media-playback-pause-symbolic");
    }
}
