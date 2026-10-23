//! Aplicação principal do Fedplay

use crate::views::{EmptyView, LibraryView};
use crate::widgets::{PlayerBar, Sidebar};
use fedplay_audio::GStreamerPlayer;
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

        // Player bar (bottom)
        let player_bar = PlayerBar::new(self.player.clone());

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
        main_box.append(&player_bar.widget);

        toast_overlay.set_child(Some(&main_box));
        window.set_content(Some(&toast_overlay));
        window.present();
    }

    /// Carrega o CSS custom
    /// Carrega o CSS custom do arquivo do sistema de arquivos
    fn load_css(&self) {
        let css_path = std::path::Path::new("fedplay-ui/data/style.css");
        let provider = gtk4::CssProvider::new();

        let fallback_css = r#"
        .fedplay-window {
            background-color: #242424;
        }

        .player-bar {
            background-color: #1e1e1e;
        }

        .track-row:hover {
            background-color: rgba(53, 132, 228, 0.1);
        }
    "#;

        if css_path.is_file() {
            println!("A carregar CSS de: {}", css_path.display());

            provider.load_from_path(css_path);
        } else {
            eprintln!(
                "CSS não encontrado em: {}. A utilizar estilos alternativos.",
                css_path.display()
            );

            provider.load_from_data(fallback_css);
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
