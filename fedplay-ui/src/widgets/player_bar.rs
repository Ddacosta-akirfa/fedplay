//! Player bar persistente do Fedplay

use fedplay_audio::GStreamerPlayer;
use fedplay_core::{PlaybackState, Track};
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Modos de repetição da Player Bar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    None,
    RepeatAll,
    RepeatOne,
    RepeatUntilEnd,
}

impl RepeatMode {
    pub fn next(&self) -> Self {
        match self {
            RepeatMode::None => RepeatMode::RepeatAll,
            RepeatMode::RepeatAll => RepeatMode::RepeatOne,
            RepeatMode::RepeatOne => RepeatMode::RepeatUntilEnd,
            RepeatMode::RepeatUntilEnd => RepeatMode::None,
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            RepeatMode::None => "media-playlist-repeat-symbolic",
            RepeatMode::RepeatAll => "media-playlist-repeat-symbolic",
            RepeatMode::RepeatOne => "media-playlist-repeat-song-symbolic",
            RepeatMode::RepeatUntilEnd => "media-playlist-consecutive-symbolic",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            RepeatMode::None => "Repetição desativada",
            RepeatMode::RepeatAll => "Repetir todas as músicas",
            RepeatMode::RepeatOne => "Repetir música atual",
            RepeatMode::RepeatUntilEnd => "Tocar até ao fim da playlist",
        }
    }
}

/// Player bar na parte inferior da janela
pub struct PlayerBar {
    pub widget: gtk4::Box,
    title_label: gtk4::Label,
    artist_label: gtk4::Label,
    cover_image: gtk4::Image, // ← NOVO: Referência à imagem da capa
    play_button: gtk4::Button,
    shuffle_button: gtk4::Button,
    repeat_button: gtk4::Button,
    favorite_button: gtk4::Button,
    current_track: Rc<RefCell<Option<Track>>>,
    shuffle_enabled: Rc<RefCell<bool>>,
    repeat_mode: Rc<RefCell<RepeatMode>>,
    is_favorite: Rc<RefCell<bool>>,
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

        // Capa da música (agora guardamos a referência)
        let cover_image = gtk4::Image::from_icon_name("audio-x-generic-symbolic");
        cover_image.set_pixel_size(50);
        cover_image.add_css_class("cover");

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

        info_box.append(&cover_image);
        info_box.append(&text_box);

        // Coluna central: controles
        let controls_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        controls_box.set_halign(gtk4::Align::Center);

        let buttons_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        buttons_box.set_halign(gtk4::Align::Center);

        let shuffle_button = gtk4::Button::from_icon_name("media-playlist-shuffle-symbolic");
        shuffle_button.add_css_class("control-button");
        shuffle_button.set_tooltip_text(Some("Embaralhar reprodução"));

        let prev_button = gtk4::Button::from_icon_name("media-skip-backward-symbolic");
        prev_button.add_css_class("control-button");
        prev_button.set_tooltip_text(Some("Faixa anterior"));

        let play_button = gtk4::Button::from_icon_name("media-playback-start-symbolic");
        play_button.add_css_class("play-button");
        play_button.set_tooltip_text(Some("Reproduzir/Pausar"));

        let next_button = gtk4::Button::from_icon_name("media-skip-forward-symbolic");
        next_button.add_css_class("control-button");
        next_button.set_tooltip_text(Some("Próxima faixa"));

        let repeat_button = gtk4::Button::from_icon_name("media-playlist-repeat-symbolic");
        repeat_button.add_css_class("control-button");
        repeat_button.set_tooltip_text(Some("Repetição desativada"));

        let favorite_button = gtk4::Button::from_icon_name("non-starred-symbolic");
        favorite_button.add_css_class("control-button");
        favorite_button.set_tooltip_text(Some("Favoritar esta música"));

        // Lógica do Shuffle
        let shuffle_enabled = Rc::new(RefCell::new(false));
        let shuffle_btn_clone = shuffle_button.clone();
        let shuffle_state = shuffle_enabled.clone();
        shuffle_button.connect_clicked(move |_| {
            let mut state = shuffle_state.borrow_mut();
            *state = !*state;
            if *state {
                shuffle_btn_clone.add_css_class("active");
                shuffle_btn_clone.set_tooltip_text(Some("Reprodução embaralhada"));
            } else {
                shuffle_btn_clone.remove_css_class("active");
                shuffle_btn_clone.set_tooltip_text(Some("Embaralhar reprodução"));
            }
        });

        // Lógica do Play/Pause
        let player_play = player.clone();
        let play_btn_clone = play_button.clone();
        play_button.connect_clicked(move |_| {
            let state = player_play.borrow().get_state();
            let _ = if matches!(state.playback, PlaybackState::Playing) {
                play_btn_clone.set_icon_name("media-playback-start-symbolic");
                player_play.borrow().pause()
            } else {
                play_btn_clone.set_icon_name("media-playback-pause-symbolic");
                player_play.borrow().play()
            };
        });

        // Lógica do Repeat
        let repeat_mode = Rc::new(RefCell::new(RepeatMode::None));
        let repeat_btn_clone = repeat_button.clone();
        let repeat_state = repeat_mode.clone();
        repeat_button.connect_clicked(move |_| {
            let mut mode = repeat_state.borrow_mut();
            *mode = mode.next();
            repeat_btn_clone.set_icon_name(mode.icon_name());
            repeat_btn_clone.set_tooltip_text(Some(mode.tooltip()));

            if *mode != RepeatMode::None {
                repeat_btn_clone.add_css_class("active");
            } else {
                repeat_btn_clone.remove_css_class("active");
            }
        });

        // Lógica do Favorite
        let is_favorite = Rc::new(RefCell::new(false));
        let fav_btn_clone = favorite_button.clone();
        let fav_state = is_favorite.clone();
        favorite_button.connect_clicked(move |_| {
            let mut state = fav_state.borrow_mut();
            *state = !*state;
            if *state {
                fav_btn_clone.set_icon_name("starred-symbolic");
                fav_btn_clone.add_css_class("favorite-active");
                fav_btn_clone.set_tooltip_text(Some("Remover dos favoritos"));
            } else {
                fav_btn_clone.set_icon_name("non-starred-symbolic");
                fav_btn_clone.remove_css_class("favorite-active");
                fav_btn_clone.set_tooltip_text(Some("Favoritar esta música"));
            }
        });

        buttons_box.append(&shuffle_button);
        buttons_box.append(&prev_button);
        buttons_box.append(&play_button);
        buttons_box.append(&next_button);
        buttons_box.append(&repeat_button);
        buttons_box.append(&favorite_button);

        // Slider de progresso
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

        let current_track = Rc::new(RefCell::new(None));

        Self {
            widget: bar,
            title_label,
            artist_label,
            cover_image, // ← Guardar referência
            play_button,
            shuffle_button,
            repeat_button,
            favorite_button,
            current_track,
            shuffle_enabled,
            repeat_mode,
            is_favorite,
        }
    }

    /// Atualiza as informações da faixa na interface
    pub fn set_track(&self, track: &Track) {
        self.title_label.set_text(&track.display_title());
        self.artist_label.set_text(&track.display_artist());
        self.play_button
            .set_icon_name("media-playback-pause-symbolic");
        *self.current_track.borrow_mut() = Some(track.clone());

        // Resetar estado de favorito ao trocar de música
        *self.is_favorite.borrow_mut() = false;
        self.favorite_button.set_icon_name("non-starred-symbolic");
        self.favorite_button.remove_css_class("favorite-active");

        // Carregar a capa do ficheiro
        self.update_cover_art(track);
    }

    /// Extrai e atualiza a capa da música
    fn update_cover_art(&self, track: &Track) {
        use lofty::TaggedFileExt;

        // Mostrar o ícone padrão enquanto não houver capa.
        self.cover_image
            .set_icon_name(Some("audio-x-generic-symbolic"));

        // Ler os metadados do ficheiro.
        let Ok(tagged_file) = lofty::read_from_path(&track.path) else {
            return;
        };

        let Some(tag) = tagged_file.primary_tag() else {
            return;
        };

        // Obter a primeira imagem incorporada nos metadados.
        let Some(picture) = tag.pictures().first() else {
            return;
        };

        let loader = gtk4::gdk_pixbuf::PixbufLoader::new();

        if loader.write(picture.data()).is_err() {
            return;
        }

        if loader.close().is_err() {
            return;
        }

        let Some(pixbuf) = loader.pixbuf() else {
            return;
        };

        // Redimensionar a capa para 50 × 50 pixels.
        let Some(scaled) = pixbuf.scale_simple(50, 50, gtk4::gdk_pixbuf::InterpType::Bilinear)
        else {
            return;
        };

        // Converter para Texture e atualizar a interface.
        let texture = gtk4::gdk::Texture::for_pixbuf(&scaled);
        self.cover_image.set_paintable(Some(&texture));
    }

    /// Retorna a faixa atualmente carregada
    pub fn get_current_track(&self) -> Option<Track> {
        self.current_track.borrow().clone()
    }

    pub fn is_shuffle_enabled(&self) -> bool {
        *self.shuffle_enabled.borrow()
    }

    pub fn get_repeat_mode(&self) -> RepeatMode {
        *self.repeat_mode.borrow()
    }

    pub fn is_current_favorite(&self) -> bool {
        *self.is_favorite.borrow()
    }
}
