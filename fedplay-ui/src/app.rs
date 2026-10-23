//! Aplicação principal do FedPlay.

use crate::views::{EmptyView, LibraryView, TagEditorDialog};
use crate::widgets::{PlayerBar, Sidebar};

use fedplay_audio::GStreamerPlayer;
use fedplay_core::{Track, TrackMetadata, read_metadata};

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

/// Aplicação principal do FedPlay.
pub struct FedplayApp {
    app: adw::Application,
    player: Rc<RefCell<GStreamerPlayer>>,
}

impl FedplayApp {
    /// Cria uma nova instância da aplicação.
    pub fn new(app: &adw::Application) -> Self {
        let player =
            GStreamerPlayer::new().expect("Não foi possível inicializar o leitor de áudio");

        Self {
            app: app.clone(),
            player: Rc::new(RefCell::new(player)),
        }
    }

    /// Constrói e apresenta a interface principal.
    pub fn build_ui(&self) {
        self.load_css();

        let window = self.build_main_window();
        let toast_overlay = adw::ToastOverlay::new();

        // Componentes principais.
        let player_bar = Rc::new(PlayerBar::new(Rc::clone(&self.player)));
        let sidebar = Sidebar::new();

        let content_stack = self.build_content_stack();
        let header = adw::HeaderBar::new();

        self.configure_header(&header);

        // Configurar os botões.
        self.setup_add_button(&header, &window, &player_bar);

        self.setup_edit_button(&header, &window, &player_bar, &toast_overlay);

        self.setup_header_actions(&header);

        // Área de conteúdo.
        let content_toolbar = adw::ToolbarView::new();
        content_toolbar.add_top_bar(&header);
        content_toolbar.set_content(Some(&content_stack));

        // Navegação.
        let split_view = self.build_split_view(&sidebar, &content_toolbar);

        // Layout principal.
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        main_box.append(&split_view);
        main_box.append(&player_bar.widget);

        toast_overlay.set_child(Some(&main_box));
        window.set_content(Some(&toast_overlay));

        window.present();
    }

    // --------------------------------------------------
    // Janela principal
    // --------------------------------------------------

    fn build_main_window(&self) -> adw::ApplicationWindow {
        let window = adw::ApplicationWindow::builder()
            .application(&self.app)
            .title("FedPlay")
            .default_width(1200)
            .default_height(800)
            .build();

        window.add_css_class("fedplay-window");

        window
    }

    // --------------------------------------------------
    // Cabeçalho
    // --------------------------------------------------

    fn configure_header(&self, header: &adw::HeaderBar) {
        let title = adw::WindowTitle::new("Todas as músicas", "Biblioteca de músicas do FedPlay");

        header.set_title_widget(Some(&title));
    }

    fn setup_header_actions(&self, header: &adw::HeaderBar) {
        // Ordenação.
        let sort_button = gtk4::MenuButton::new();
        sort_button.set_icon_name("view-sort-ascending-symbolic");
        sort_button.set_tooltip_text(Some("Ordenar músicas"));
        sort_button.add_css_class("flat");

        header.pack_start(&sort_button);

        // Pesquisa.
        let search_button = gtk4::ToggleButton::new();
        search_button.set_icon_name("system-search-symbolic");
        search_button.set_tooltip_text(Some("Pesquisar músicas"));
        search_button.add_css_class("flat");

        header.pack_end(&search_button);

        // Menu principal.
        let menu_button = gtk4::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");
        menu_button.set_tooltip_text(Some("Menu principal"));
        menu_button.add_css_class("flat");

        header.pack_end(&menu_button);
    }

    // --------------------------------------------------
    // Navegação e conteúdo
    // --------------------------------------------------

    fn build_content_stack(&self) -> gtk4::Stack {
        let stack = gtk4::Stack::new();

        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

        stack.set_vexpand(true);
        stack.set_hexpand(true);

        let library_view = LibraryView::new();
        let empty_view = EmptyView::new();

        stack.add_named(&library_view.widget, Some("library"));

        stack.add_named(&empty_view.widget, Some("empty"));

        // A biblioteca é a vista inicial.
        stack.set_visible_child_name("library");

        stack
    }

    fn build_split_view(
        &self,
        sidebar: &Sidebar,
        content: &adw::ToolbarView,
    ) -> adw::NavigationSplitView {
        let split_view = adw::NavigationSplitView::new();

        split_view.set_max_sidebar_width(280.0);
        split_view.set_min_sidebar_width(240.0);

        let sidebar_page = adw::NavigationPage::builder()
            .title("FedPlay")
            .child(&sidebar.widget)
            .build();

        split_view.set_sidebar(Some(&sidebar_page));

        let content_page = adw::NavigationPage::builder()
            .title("Biblioteca")
            .child(content)
            .build();

        split_view.set_content(Some(&content_page));

        split_view
    }

    // --------------------------------------------------
    // Adicionar música
    // --------------------------------------------------

    fn setup_add_button(
        &self,
        header: &adw::HeaderBar,
        window: &adw::ApplicationWindow,
        player_bar: &Rc<PlayerBar>,
    ) {
        let add_button = gtk4::Button::from_icon_name("list-add-symbolic");

        add_button.add_css_class("flat");

        add_button.set_tooltip_text(Some("Adicionar ficheiro de áudio"));

        let player = Rc::clone(&self.player);
        let player_bar = Rc::clone(player_bar);
        let parent_window = window.clone();

        add_button.connect_clicked(move |_| {
            let dialog = gtk4::FileDialog::new();

            dialog.set_title("Selecionar ficheiro de áudio");

            let filter = gtk4::FileFilter::new();
            filter.set_name(Some("Ficheiros de áudio"));

            for mime_type in ["audio/mpeg", "audio/mp4", "audio/x-m4a"] {
                filter.add_mime_type(mime_type);
            }

            dialog.set_default_filter(Some(&filter));

            let player = Rc::clone(&player);
            let player_bar = Rc::clone(&player_bar);

            dialog.open(
                None::<&gtk4::Window>,
                None::<&gtk4::gio::Cancellable>,
                move |result| {
                    let file = match result {
                        Ok(file) => file,
                        Err(error) => {
                            // Silencia o erro se o usuário apenas fechou a janela
                            if !error.message().contains("Dismissed")
                                && !error.message().contains("cancelado")
                            {
                                eprintln!("Erro ao selecionar arquivo: {}", error);
                            }
                            return;
                        }
                    };

                    let Some(path) = file.path() else {
                        eprintln!("Não foi possível obter o caminho do ficheiro.");
                        return;
                    };

                    if let Err(error) = load_track(&player, &player_bar, &path) {
                        eprintln!(
                            "Não foi possível reproduzir '{}': {}",
                            path.display(),
                            error
                        );
                    }
                },
            );
        });

        header.pack_end(&add_button);
    }

    // --------------------------------------------------
    // Editar metadados
    // --------------------------------------------------

    fn setup_edit_button(
        &self,
        header: &adw::HeaderBar,
        window: &adw::ApplicationWindow,
        player_bar: &Rc<PlayerBar>,
        toast_overlay: &adw::ToastOverlay,
    ) {
        let edit_button = gtk4::Button::from_icon_name("document-edit-symbolic");

        edit_button.add_css_class("flat");

        edit_button.set_tooltip_text(Some("Editar metadados da faixa atual"));

        let player_bar = Rc::clone(player_bar);
        let toast_overlay = toast_overlay.clone();
        let parent_window = window.clone();

        edit_button.connect_clicked(move |_| {
            let Some(track) = player_bar.get_current_track() else {
                show_toast(&toast_overlay, "Nenhuma faixa selecionada para editar");

                return;
            };

            let player_bar_for_saved = Rc::clone(&player_bar);
            let toast_for_saved = toast_overlay.clone();

            let on_saved = Rc::new(move |updated_track: Track| {
                // Atualizar a faixa apresentada no leitor.
                player_bar_for_saved.set_track(&updated_track);

                show_toast(&toast_for_saved, "Metadados guardados com sucesso");
            });

            let dialog = TagEditorDialog::new(&track, on_saved);

            dialog.widget.set_transient_for(Some(&parent_window));

            dialog.widget.present();
        });

        header.pack_end(&edit_button);
    }

    // --------------------------------------------------
    // CSS
    // --------------------------------------------------

    fn load_css(&self) {
        let css_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/style.css");

        let provider = gtk4::CssProvider::new();

        if css_path.is_file() {
            provider.load_from_path(&css_path);

            println!("CSS carregado: {}", css_path.display());
        } else {
            eprintln!("CSS não encontrado: {}", css_path.display());

            provider.load_from_data(
                r#"
                .fedplay-window {
                    background-color: #242424;
                }

                .player-bar {
                    background-color: #1e1e1e;
                }

                .track-row:hover {
                    background-color: rgba(53, 132, 228, 0.1);
                }
                "#,
            );
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

// ------------------------------------------------------
// Funções auxiliares
// ------------------------------------------------------

/// Carrega uma faixa no leitor e inicia a reprodução.
fn load_track(
    player: &Rc<RefCell<GStreamerPlayer>>,
    player_bar: &Rc<PlayerBar>,
    path: &Path,
) -> Result<(), String> {
    if !path.is_file() {
        return Err("O caminho selecionado não corresponde a um ficheiro.".to_string());
    }

    // Tentar ler os metadados existentes.
    let mut metadata = read_metadata(path).unwrap_or_else(|_| TrackMetadata::default());

    // Usar o nome do ficheiro como título, se necessário.
    if metadata.title.is_none() {
        metadata.title = path
            .file_stem()
            .and_then(|name| name.to_str())
            .map(str::to_string);
    }

    // TODO: substituir o ID fixo por um identificador único
    // quando a biblioteca de músicas estiver integrada.
    let track = Track::new(1, path.to_path_buf(), metadata);

    {
        let mut audio_player = player
            .try_borrow_mut()
            .map_err(|error| format!("Não foi possível aceder ao leitor: {error}"))?;

        audio_player
            .load(&track)
            .map_err(|error| format!("Erro ao carregar a faixa: {error}"))?;

        audio_player
            .play()
            .map_err(|error| format!("Erro ao iniciar a reprodução: {error}"))?;
    }

    // A atualização da interface ocorre depois de libertar
    // o empréstimo mutável do leitor.
    player_bar.set_track(&track);

    Ok(())
}

/// Apresenta uma notificação temporária.
fn show_toast(overlay: &adw::ToastOverlay, message: &str) {
    let toast = adw::Toast::new(message);

    toast.set_timeout(3);

    overlay.add_toast(toast);
}
