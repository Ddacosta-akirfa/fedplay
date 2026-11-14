use crate::views::{EmptyView, LibraryView, PlaylistsView, TagEditorDialog};
use crate::widgets::{PlayerBar, Sidebar};

use fedplay_audio::GStreamerPlayer;
use fedplay_core::{Database, LibraryManager, Track, TrackMetadata, read_metadata};

use gtk4::glib::{self, ControlFlow};
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

/// Aplicação principal do FedPlay.
pub struct FedplayApp {
    app: adw::Application,
    player: Rc<RefCell<GStreamerPlayer>>,
    library: Arc<Mutex<LibraryManager>>,
}

impl FedplayApp {
    pub fn new(app: &adw::Application) -> Result<Self, String> {
        let player = GStreamerPlayer::new()
            .map_err(|error| format!("Não foi possível inicializar o leitor: {error}"))?;

        let path = database_path()?;

        let database = Database::new(&path)
            .map_err(|error| format!("Não foi possível inicializar a base de dados: {error}"))?;

        let library = LibraryManager::new(database);

        Ok(Self {
            app: app.clone(),
            player: Rc::new(RefCell::new(player)),
            library: Arc::new(Mutex::new(library)),
        })
    }

    pub fn build_ui(&self) {
        self.load_css();

        let window = self.build_main_window();
        let toast_overlay = adw::ToastOverlay::new();

        let player_bar = Rc::new(PlayerBar::new(Rc::clone(&self.player)));
        let sidebar = Sidebar::new();

        let title = adw::WindowTitle::new("Todas as Músicas", "0 faixas");

        // Cabeçalho com título dinâmico.
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&title));

        let content_stack =
            self.build_content_stack(&player_bar, &window, &toast_overlay, &title, &header);

        // Configurar os botões do cabeçalho.
        self.setup_add_button(&header, &player_bar);
        self.setup_edit_button(&header, &window, &player_bar, &toast_overlay);
        self.setup_header_actions(&header);

        // Área de conteúdo.
        let content_toolbar = adw::ToolbarView::new();
        content_toolbar.add_top_bar(&header);
        content_toolbar.set_content(Some(&content_stack));

        // Navegação da sidebar e atualização do título.
        let content_stack_clone = content_stack.clone();
        let title_clone = title.clone();

        sidebar.list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                match row.index() {
                    0 => {
                        content_stack_clone.set_visible_child_name("library");
                        title_clone.set_title("Todas as Músicas");
                        title_clone.set_subtitle("148 faixas");
                    }
                    1 => {
                        content_stack_clone.set_visible_child_name("library");
                        title_clone.set_title("Artistas");
                        title_clone.set_subtitle("42 artistas");
                    }
                    2 => {
                        content_stack_clone.set_visible_child_name("library");
                        title_clone.set_title("Álbuns");
                        title_clone.set_subtitle("28 álbuns");
                    }
                    3 => {
                        content_stack_clone.set_visible_child_name("playlists");
                        title_clone.set_title("Listas de Reprodução");
                        title_clone.set_subtitle("8 listas salvas");
                    }
                    _ => {}
                }
            }
        });

        // Navegação principal.
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

    fn build_content_stack(
        &self,
        player_bar: &Rc<PlayerBar>,
        window: &adw::ApplicationWindow,
        toast_overlay: &adw::ToastOverlay,
        title: &adw::WindowTitle,
        header: &adw::HeaderBar,
    ) -> gtk4::Stack {
        let stack = gtk4::Stack::new();

        // Views
        let library_view = Rc::new(LibraryView::new());
        let empty_view = EmptyView::new();

        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

        stack.set_vexpand(true);
        stack.set_hexpand(true);

        let playlists_view = PlaylistsView::new();

        // Seleção de uma pasta de música.
        let parent_window = window.clone();

        let library = Arc::clone(&self.library);
        let library_view_for_empty = Rc::clone(&library_view);
        let stack_for_empty = stack.clone();
        let title_for_empty = title.clone();
        let toast_for_empty = toast_overlay.clone();

        empty_view.connect_add_folder(move || {
            open_folder_import_dialog(
                parent_window.clone(),
                Arc::clone(&library),
                toast_for_empty.clone(),
                Rc::clone(&library_view_for_empty),
                stack_for_empty.clone(),
                title_for_empty.clone(),
            );
        });

        stack.add_named(&library_view.widget, Some("library"));
        stack.add_named(&empty_view.widget, Some("empty"));
        stack.add_named(&playlists_view.widget, Some("playlists"));

        let import_folder_button = gtk4::Button::from_icon_name("folder-open-symbolic");

        import_folder_button.add_css_class("flat");
        import_folder_button.set_tooltip_text(Some("Importar pasta de música"));

        header.pack_end(&import_folder_button);

        let parent_window = window.clone();
        let library = Arc::clone(&self.library);
        let library_view_for_button = Rc::clone(&library_view);
        let stack_for_button = stack.clone();
        let title_for_button = title.clone();
        let toast_for_button = toast_overlay.clone();

        import_folder_button.connect_clicked(move |_| {
            open_folder_import_dialog(
                parent_window.clone(),
                Arc::clone(&library),
                toast_for_button.clone(),
                Rc::clone(&library_view_for_button),
                stack_for_button.clone(),
                title_for_button.clone(),
            );
        });

        let initial_track_count = match self.library.lock() {
            Ok(manager) => match manager.get_tracks() {
                Ok(tracks) => {
                    let count = tracks.len();
                    library_view.set_tracks(tracks);
                    count
                }
                Err(error) => {
                    eprintln!("Erro ao carregar biblioteca: {error}");
                    0
                }
            },
            Err(error) => {
                eprintln!("Erro ao aceder à biblioteca: {error}");
                0
            }
        };

        title.set_subtitle(&format!("{initial_track_count} faixas"));

        stack.set_visible_child_name(if initial_track_count == 0 {
            "empty"
        } else {
            "library"
        });

        // Conectar o duplo-clique nas faixas da biblioteca
        let player_clone_library = self.player.clone();
        let player_bar_clone_library = Rc::clone(player_bar);

        library_view.connect_track_activated(move |track| {
            let mut player = player_clone_library.borrow_mut();

            if player.load(track).is_ok() && player.play().is_ok() {
                player_bar_clone_library.set_track(track);
            }
        });

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

    fn setup_add_button(&self, header: &adw::HeaderBar, player_bar: &Rc<PlayerBar>) {
        let add_button = gtk4::Button::from_icon_name("list-add-symbolic");

        add_button.add_css_class("flat");

        add_button.set_tooltip_text(Some("Adicionar ficheiro de áudio"));

        let player = Rc::clone(&self.player);
        let player_bar = Rc::clone(player_bar);

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

fn start_directory_import(
    library: Arc<Mutex<LibraryManager>>,
    path: PathBuf,
    toast_overlay: adw::ToastOverlay,
    library_view: Rc<LibraryView>,
    stack: gtk4::Stack,
    title: adw::WindowTitle,
) {
    show_toast(&toast_overlay, "A importar músicas...");

    let (sender, receiver) = mpsc::channel();

    std::thread::spawn(move || {
        let result = match library.lock() {
            Ok(manager) => match manager.import_directory(&path) {
                Ok(summary) => manager
                    .get_tracks()
                    .map(|tracks| (summary, tracks))
                    .map_err(|error| error.to_string()),

                Err(error) => Err(error.to_string()),
            },

            Err(error) => Err(format!("Não foi possível aceder à biblioteca: {error}")),
        };

        let _ = sender.send(result);
    });

    glib::timeout_add_local(Duration::from_millis(100), move || {
        match receiver.try_recv() {
            Ok(Ok((summary, tracks))) => {
                let count = tracks.len();

                // Atualizar os dados e as linhas da biblioteca.
                library_view.set_tracks(tracks);

                // Atualizar a contagem do cabeçalho.
                title.set_subtitle(&format!("{count} faixas"));

                // Mostrar a view adequada.
                stack.set_visible_child_name(if count == 0 { "empty" } else { "library" });

                show_toast(
                    &toast_overlay,
                    &format!(
                        "Importação concluída: {} encontradas, {} adicionadas, {} atualizadas.",
                        summary.tracks_found, summary.tracks_added, summary.tracks_updated,
                    ),
                );

                ControlFlow::Break
            }

            Ok(Err(error)) => {
                eprintln!("Erro durante a importação: {error}");

                show_toast(
                    &toast_overlay,
                    &format!("Erro ao importar músicas: {error}"),
                );

                ControlFlow::Break
            }

            Err(mpsc::TryRecvError::Empty) => ControlFlow::Continue,

            Err(mpsc::TryRecvError::Disconnected) => {
                show_toast(
                    &toast_overlay,
                    "A importação foi interrompida inesperadamente.",
                );

                ControlFlow::Break
            }
        }
    });
}

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

fn open_folder_import_dialog(
    parent_window: adw::ApplicationWindow,
    library: Arc<Mutex<LibraryManager>>,
    toast_overlay: adw::ToastOverlay,
    library_view: Rc<LibraryView>,
    stack: gtk4::Stack,
    title: adw::WindowTitle,
) {
    let dialog = gtk4::FileDialog::new();
    dialog.set_title("Selecionar pasta de música");

    dialog.select_folder(
        Some(&parent_window),
        None::<&gtk4::gio::Cancellable>,
        move |result| {
            let folder = match result {
                Ok(folder) => folder,
                Err(error) => {
                    eprintln!("Erro ao selecionar pasta: {error}");
                    return;
                }
            };

            let Some(path) = folder.path() else {
                show_toast(&toast_overlay, "Não foi possível obter o caminho da pasta.");
                return;
            };

            start_directory_import(library, path, toast_overlay, library_view, stack, title);
        },
    );
}

/// Apresenta uma notificação temporária.
fn show_toast(overlay: &adw::ToastOverlay, message: &str) {
    let toast = adw::Toast::new(message);

    toast.set_timeout(3);

    overlay.add_toast(toast);
}

fn database_path() -> Result<PathBuf, String> {
    let data_dir = match std::env::var_os("XDG_DATA_HOME") {
        Some(path) => PathBuf::from(path),
        None => {
            let home =
                std::env::var_os("HOME").ok_or("Não foi possível localizar o diretório HOME")?;

            PathBuf::from(home).join(".local/share")
        }
    };

    let fedplay_dir = data_dir.join("fedplay");

    fs::create_dir_all(&fedplay_dir)
        .map_err(|error| format!("Erro ao criar diretório de dados: {error}"))?;

    Ok(fedplay_dir.join("library.db"))
}
