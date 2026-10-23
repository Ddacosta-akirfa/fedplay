//! Aplicação principal do Fedplay

use crate::views::{EmptyView, LibraryView};
use crate::widgets::{PlayerBar, Sidebar};
use fedplay_audio::GStreamerPlayer;
use fedplay_core::Track;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc; // Necessário para o Rc::clone

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
        // Carregar CSS custom
        self.load_css();

        // Criar janela principal
        let window = adw::ApplicationWindow::builder()
            .application(&self.app)
            .title("Fedplay")
            .default_width(1200)
            .default_height(800)
            .build();
        window.add_css_class("fedplay-window");

        // Toast overlay (para notificações)
        let toast_overlay = adw::ToastOverlay::new();

        // Player bar (bottom) - Envolvido em Rc para permitir clone no closure
        let player_bar = Rc::new(PlayerBar::new(self.player.clone()));

        // Split view (sidebar + content)
        let split_view = adw::NavigationSplitView::new();
        split_view.set_max_sidebar_width(280.0);
        split_view.set_min_sidebar_width(240.0);

        // Sidebar
        let sidebar = Sidebar::new();
        let sidebar_page = adw::NavigationPage::builder()
            .title("Fedplay")
            .child(&sidebar.widget)
            .build();
        split_view.set_sidebar(Some(&sidebar_page));

        // Content area (será trocado dinamicamente)
        let content_stack = gtk4::Stack::new();
        content_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        content_stack.set_vexpand(true);

        // Views
        let library_view = LibraryView::new();
        let empty_view = EmptyView::new();

        content_stack.add_named(&library_view.widget, Some("library"));
        content_stack.add_named(&empty_view.widget, Some("empty"));

        // Por enquanto, mostrar biblioteca
        content_stack.set_visible_child_name("library");

        // ToolbarView para o content
        let content_toolbar = adw::ToolbarView::new();

        // Header bar
        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Todas as Músicas", "Biblioteca de músicas do Fedplay");
        header.set_title_widget(Some(&title));

        // ==========================================
        // Botão de Adicionar Música (Funcionalidade M1 restaurada)
        // ==========================================
        let add_button = gtk4::Button::from_icon_name("list-add-symbolic");
        add_button.add_css_class("flat");
        add_button.set_tooltip_text(Some("Adicionar arquivo de áudio"));

        let player_clone = Rc::clone(&self.player);
        let player_bar_clone = Rc::clone(&player_bar);

        add_button.connect_clicked(move |_| {
            // Criar clones para esta operação de seleção.
            // A closure do botão continua reutilizável.
            let player = Rc::clone(&player_clone);
            let player_bar = Rc::clone(&player_bar_clone);

            let dialog = gtk4::FileDialog::new();
            dialog.set_title("Selecionar Arquivo de Áudio");

            let filter = gtk4::FileFilter::new();
            filter.set_name(Some("Arquivos de Áudio"));
            filter.add_mime_type("audio/mpeg");
            filter.add_mime_type("audio/mp4");
            filter.add_mime_type("audio/x-m4a");
            filter.add_mime_type("audio/flac");
            dialog.set_default_filter(Some(&filter));

            dialog.open(
                None::<&gtk4::Window>,
                None::<&gtk4::gio::Cancellable>,
                move |result| {
                    let file = match result {
                        Ok(file) => file,
                        Err(error) => {
                            eprintln!("Erro ao selecionar ficheiro: {error}");
                            return;
                        }
                    };

                    let path = match file.path() {
                        Some(path) => path,
                        None => {
                            eprintln!("Não foi possível obter o caminho do ficheiro.");
                            return;
                        }
                    };

                    let track = Track::new(
                        1,
                        path.clone(),
                        fedplay_core::TrackMetadata {
                            title: path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .map(str::to_string),
                            ..Default::default()
                        },
                    );

                    let mut player = player.borrow_mut();

                    if let Err(error) = player.load(&track) {
                        eprintln!("Erro ao carregar faixa: {error}");
                        return;
                    }

                    if let Err(error) = player.play() {
                        eprintln!("Erro ao iniciar reprodução: {error}");
                        return;
                    }

                    // Atualizar a interface após iniciar a reprodução.
                    player_bar.set_track(&track);
                },
            );
        });

        header.pack_end(&add_button);
        // ==========================================

        // Botões do header
        let sort_button = gtk4::MenuButton::new();
        sort_button.set_icon_name("view-sort-ascending-symbolic");
        sort_button.add_css_class("flat");
        header.pack_start(&sort_button);

        let search_button = gtk4::ToggleButton::new();
        search_button.set_icon_name("system-search-symbolic");
        search_button.add_css_class("flat");
        header.pack_end(&search_button);

        let menu_button = gtk4::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");
        menu_button.add_css_class("flat");
        header.pack_end(&menu_button);

        content_toolbar.add_top_bar(&header);
        content_toolbar.set_content(Some(&content_stack));

        // Content page
        let content_page = adw::NavigationPage::builder()
            .title("Biblioteca")
            .child(&content_toolbar)
            .build();
        split_view.set_content(Some(&content_page));

        // Layout principal (split + player bar)
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.append(&split_view);

        // Rc faz deref automaticamente, então &player_bar.widget funciona perfeitamente
        main_box.append(&player_bar.widget);

        toast_overlay.set_child(Some(&main_box));
        window.set_content(Some(&toast_overlay));
        window.present();
    }

    /// Carrega o CSS custom do arquivo do sistema de arquivos
    /// Carrega o CSS custom do arquivo do sistema de arquivos
    fn load_css(&self) {
        // CARGO_MANIFEST_DIR aponta sempre para a raiz do crate fedplay-ui
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let css_path = std::path::Path::new(manifest_dir).join("data/style.css");

        let provider = gtk4::CssProvider::new();

        let fallback_css = r#"
            .fedplay-window { background-color: #242424; }
            .player-bar { background-color: #1e1e1e; }
            .track-row:hover { background-color: rgba(53, 132, 228, 0.1); }
        "#;

        if css_path.is_file() {
            println!("✓ CSS carregado de: {}", css_path.display());
            let _ = provider.load_from_path(&css_path);
        } else {
            eprintln!(
                "⚠ CSS não encontrado em: {}. A utilizar estilos alternativos.",
                css_path.display()
            );
            let _ = provider.load_from_data(fallback_css);
        }

        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        } else {
            eprintln!("Não foi possível obter o display GTK.");
        }
    }
}
