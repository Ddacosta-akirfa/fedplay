//! Aplicação principal do Fedplay

use fedplay_audio::GStreamerPlayer;
use fedplay_core::{Track, TrackMetadata};
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Aplicação principal do Fedplay
pub struct FedplayApp {
    app: adw::Application,
    player: Rc<RefCell<GStreamerPlayer>>,
}

impl FedplayApp {
    /// Cria uma nova instância da aplicação
    pub fn new(app: &adw::Application) -> Self {
        let player = GStreamerPlayer::new().expect("Falha ao criar player");

        Self {
            app: app.clone(),
            player: Rc::new(RefCell::new(player)),
        }
    }

    /// Constrói a interface do usuário
    pub fn build_ui(&self) {
        // Criar janela principal
        let window = adw::ApplicationWindow::builder()
            .application(&self.app)
            .title("Fedplay")
            .default_width(800)
            .default_height(600)
            .build();

        // Criar layout principal
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        // Header bar
        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Fedplay", "None");
        header.set_title_widget(Some(&title));
        main_box.append(&header);

        // Área de conteúdo
        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        content_box.set_margin_top(24);
        content_box.set_margin_bottom(24);
        content_box.set_margin_start(24);
        content_box.set_margin_end(24);
        content_box.set_vexpand(true);

        // Label de status
        let status_label = gtk4::Label::new(Some("Bem-vindo ao Fedplay!"));
        status_label.add_css_class("title-1");
        content_box.append(&status_label);

        // Botão para carregar arquivo
        let load_button = gtk4::Button::with_label("Carregar Arquivo de Áudio");
        load_button.add_css_class("suggested-action");

        let player_clone = self.player.clone();
        let status_clone = status_label.clone();
        let window_clone = window.clone();

        load_button.connect_clicked(move |_| {
            // Criar diálogo de seleção de arquivo
            let dialog = gtk4::FileDialog::new();
            dialog.set_title("Selecionar Arquivo de Áudio");

            let player_clone2 = player_clone.clone();
            let status_clone2 = status_clone.clone();
            let window_clone2 = window_clone.clone();

            dialog.open(
                Some(&window_clone2),
                None::<&gtk4::gio::Cancellable>,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            // Criar track
                            let track = Track::new(
                                1,
                                path.clone(),
                                TrackMetadata {
                                    title: path
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .map(|s| s.to_string()),
                                    ..Default::default()
                                },
                            );

                            // Carregar e reproduzir
                            let mut player = player_clone2.borrow_mut();
                            if let Err(e) = player.load(&track) {
                                status_clone2.set_text(&format!("Erro: {}", e));
                            } else {
                                if let Err(e) = player.play() {
                                    status_clone2.set_text(&format!("Erro ao reproduzir: {}", e));
                                } else {
                                    status_clone2.set_text(&format!(
                                        "Reproduzindo: {}",
                                        track.display_title()
                                    ));
                                }
                            }
                        }
                    }
                },
            );
        });

        content_box.append(&load_button);

        // Controles de reprodução
        let controls_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        controls_box.set_halign(gtk4::Align::Center);
        controls_box.set_margin_top(24);

        let play_button = gtk4::Button::from_icon_name("media-playback-start-symbolic");
        play_button.add_css_class("flat");

        let pause_button = gtk4::Button::from_icon_name("media-playback-pause-symbolic");
        pause_button.add_css_class("flat");

        let stop_button = gtk4::Button::from_icon_name("media-playback-stop-symbolic");
        stop_button.add_css_class("flat");

        let player_play = self.player.clone();
        play_button.connect_clicked(move |_| {
            let _ = player_play.borrow().play();
        });

        let player_pause = self.player.clone();
        pause_button.connect_clicked(move |_| {
            let _ = player_pause.borrow().pause();
        });

        let player_stop = self.player.clone();
        stop_button.connect_clicked(move |_| {
            let _ = player_stop.borrow().stop();
        });

        controls_box.append(&play_button);
        controls_box.append(&pause_button);
        controls_box.append(&stop_button);
        content_box.append(&controls_box);

        main_box.append(&content_box);

        // Adicionar conteúdo à janela
        window.set_content(Some(&main_box));
        window.present();
    }
}
