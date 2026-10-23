//! Diálogo de edição de metadados de uma faixa.

use fedplay_core::{Track, TrackMetadata, read_metadata, write_metadata};
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::rc::Rc;

/// Diálogo para visualizar e editar os metadados de uma faixa.
pub struct TagEditorDialog {
    /// Janela do editor de metadados.
    pub widget: adw::Window,
}

impl TagEditorDialog {
    /// Cria um diálogo preenchido com os metadados da faixa.
    ///
    /// `on_saved` é executado quando os metadados são guardados
    /// com sucesso.
    pub fn new(track: &Track, on_saved: Rc<dyn Fn(Track) + 'static>) -> Self {
        // Janela principal do diálogo.
        let window = adw::Window::builder()
            .title("Editar Metadados")
            .default_width(500)
            .default_height(600)
            .modal(true)
            .build();

        // Carregar os metadados existentes.
        let current_meta = read_metadata(&track.path).unwrap_or_else(|_| track.metadata.clone());

        // Conteúdo principal.
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        main_box.set_margin_top(16);
        main_box.set_margin_bottom(24);
        main_box.set_margin_start(24);
        main_box.set_margin_end(24);

        // --------------------------------------------------
        // Cabeçalho
        // --------------------------------------------------

        let header = adw::HeaderBar::new();

        let cancel_button = gtk4::Button::with_label("Cancelar");
        cancel_button.set_tooltip_text(Some("Cancelar a edição"));

        let window_for_cancel = window.clone();

        cancel_button.connect_clicked(move |_| {
            window_for_cancel.close();
        });

        header.pack_start(&cancel_button);

        let window_title = adw::WindowTitle::new("Editar Metadados", "");
        header.set_title_widget(Some(&window_title));

        let save_button = gtk4::Button::with_label("Guardar");
        save_button.add_css_class("suggested-action");
        save_button.set_tooltip_text(Some("Guardar os metadados"));

        header.pack_end(&save_button);

        main_box.append(&header);

        // --------------------------------------------------
        // Capa do álbum
        // --------------------------------------------------

        let cover_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        cover_box.set_halign(gtk4::Align::Center);
        cover_box.set_margin_bottom(8);

        let cover_image = gtk4::Image::from_icon_name("audio-x-generic-symbolic");
        cover_image.set_pixel_size(112);
        cover_image.add_css_class("card");
        cover_image.set_accessible_role(gtk4::AccessibleRole::Img);
        cover_image.set_tooltip_text(Some("Capa do álbum"));

        let change_cover_button = gtk4::Button::with_label("Alterar capa");
        change_cover_button.add_css_class("flat");
        change_cover_button.set_tooltip_text(Some("A alteração da capa ainda não está disponível"));
        change_cover_button.set_sensitive(false);

        cover_box.append(&cover_image);
        cover_box.append(&change_cover_button);

        main_box.append(&cover_box);

        // --------------------------------------------------
        // Informações da faixa
        // --------------------------------------------------

        let preferences_group = adw::PreferencesGroup::new();
        preferences_group.set_title("Informações da faixa");
        preferences_group.set_description(Some("Edita os dados associados ao ficheiro de áudio."));

        // Título.
        let title_row = adw::EntryRow::new();
        title_row.set_title("Título");

        if let Some(title) = current_meta.title.as_deref() {
            title_row.set_text(title);
        }

        preferences_group.add(&title_row);

        // Artista.
        let artist_row = adw::EntryRow::new();
        artist_row.set_title("Artista");

        if let Some(artist) = current_meta.artist.as_deref() {
            artist_row.set_text(artist);
        }

        preferences_group.add(&artist_row);

        // Álbum.
        let album_row = adw::EntryRow::new();
        album_row.set_title("Álbum");

        if let Some(album) = current_meta.album.as_deref() {
            album_row.set_text(album);
        }

        preferences_group.add(&album_row);

        // Género.
        let genre_row = adw::EntryRow::new();
        genre_row.set_title("Género");

        if let Some(genre) = current_meta.genre.as_deref() {
            genre_row.set_text(genre);
        }

        preferences_group.add(&genre_row);

        // --------------------------------------------------
        // Número da faixa e ano
        // --------------------------------------------------

        let metadata_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        metadata_box.set_homogeneous(true);

        // Número da faixa: zero representa um valor não definido.
        let track_number_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);

        let track_number_label = gtk4::Label::new(Some("Número da faixa"));
        track_number_label.set_halign(gtk4::Align::Start);
        track_number_label.add_css_class("dim-label");

        let track_number_adjustment = gtk4::Adjustment::new(0.0, 0.0, 999.0, 1.0, 10.0, 0.0);

        let track_number_spin = gtk4::SpinButton::new(Some(&track_number_adjustment), 1.0, 0);

        track_number_spin.set_numeric(true);
        track_number_spin.set_tooltip_text(Some("Deixa em zero se o número não estiver definido"));

        if let Some(number) = current_meta.track_number {
            track_number_spin.set_value(number as f64);
        }

        track_number_box.append(&track_number_label);
        track_number_box.append(&track_number_spin);

        // Ano: zero representa um valor não definido.
        let year_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);

        let year_label = gtk4::Label::new(Some("Ano"));
        year_label.set_halign(gtk4::Align::Start);
        year_label.add_css_class("dim-label");

        let year_adjustment = gtk4::Adjustment::new(0.0, 0.0, 9999.0, 1.0, 10.0, 0.0);

        let year_spin = gtk4::SpinButton::new(Some(&year_adjustment), 1.0, 0);

        year_spin.set_numeric(true);
        year_spin.set_tooltip_text(Some("Deixa em zero se o ano não estiver definido"));

        if let Some(year) = current_meta.year {
            year_spin.set_value(year as f64);
        }

        year_box.append(&year_label);
        year_box.append(&year_spin);

        metadata_box.append(&track_number_box);
        metadata_box.append(&year_box);

        let metadata_row = adw::ActionRow::new();
        metadata_row.set_child(Some(&metadata_box));

        preferences_group.add(&metadata_row);
        main_box.append(&preferences_group);

        // --------------------------------------------------
        // Informações do ficheiro
        // --------------------------------------------------

        let file_name = track
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Ficheiro desconhecido");

        let file_size = std::fs::metadata(&track.path)
            .ok()
            .map(|metadata| format_file_size(metadata.len()))
            .unwrap_or_else(|| "Tamanho desconhecido".to_string());

        let file_info = gtk4::Label::new(Some(&format!("{file_name} · {file_size}")));

        file_info.set_halign(gtk4::Align::Start);
        file_info.set_wrap(true);
        file_info.add_css_class("dim-label");

        main_box.append(&file_info);

        // --------------------------------------------------
        // Área com scroll
        // --------------------------------------------------

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scroll.set_vexpand(true);
        scroll.set_child(Some(&main_box));

        window.set_content(Some(&scroll));

        // --------------------------------------------------
        // Guardar metadados
        // --------------------------------------------------

        let track_to_update = track.clone();
        let window_for_save = window.clone();

        save_button.connect_clicked(move |_| {
            let track_number = track_number_spin.value_as_int();
            let year = year_spin.value_as_int();

            let new_metadata = TrackMetadata {
                title: non_empty_text(title_row.text().as_str()),
                artist: non_empty_text(artist_row.text().as_str()),
                album: non_empty_text(album_row.text().as_str()),
                genre: non_empty_text(genre_row.text().as_str()),

                track_number: (track_number > 0).then_some(track_number as u32),

                year: (year > 0).then_some(year as u32),

                duration_ms: current_meta.duration_ms,
            };

            match write_metadata(&track_to_update.path, &new_metadata) {
                Ok(()) => {
                    let mut updated_track = track_to_update.clone();
                    updated_track.metadata = new_metadata;

                    on_saved(updated_track);
                    window_for_save.close();
                }

                Err(error) => {
                    eprintln!(
                        "Erro ao guardar metadados de '{}': {}",
                        track_to_update.path.display(),
                        error
                    );

                    // Mostra um Toast de erro vermelho (destrutivo) para o utilizador
                    let toast = adw::Toast::new(&format!("Erro ao guardar: {}", error));
                    // Opcional: definir uma cor ou ícone de erro se o seu CSS suportar,
                    // ou apenas confiar na mensagem clara.
                    toast.set_timeout(3); // 3 segundos para dar tempo de ler

                    // Nota: Precisa de ter uma referência ao ToastOverlay aqui.
                    // Se não tiver, pode ignorar esta parte e manter apenas o eprintln! por enquanto.
                    // toast_overlay.add_toast(toast);
                }
            }
        });

        Self { widget: window }
    }
}

/// Converte texto vazio em `None` e remove espaços exteriores.
fn non_empty_text(value: &str) -> Option<String> {
    let value = value.trim();

    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Formata o tamanho de um ficheiro para apresentação.
fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let size = bytes as f64;

    if size >= GB {
        format!("{:.2} GB", size / GB)
    } else if size >= MB {
        format!("{:.2} MB", size / MB)
    } else if size >= KB {
        format!("{:.2} KB", size / KB)
    } else {
        format!("{bytes} bytes")
    }
}
