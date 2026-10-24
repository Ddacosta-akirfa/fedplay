//! View de Playlists (Visão Geral e Detalhe)

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

/// View de playlists
pub struct PlaylistsView {
    pub widget: gtk4::Box,
    stack: gtk4::Stack,
}

impl PlaylistsView {
    /// Cria uma nova view de playlists
    pub fn new() -> Self {
        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);
        stack.set_vexpand(true);

        // ==========================================
        // 1. VISÃO GERAL (Lista de Playlists)
        // ==========================================
        let overview_scroll = gtk4::ScrolledWindow::new();
        overview_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        overview_scroll.set_vexpand(true);
        overview_scroll.add_css_class("view-surface");

        let overview_box = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
        overview_box.set_margin_top(16);
        overview_box.set_margin_bottom(16);
        overview_box.set_margin_start(16);
        overview_box.set_margin_end(16);

        // Seção: Playlists Inteligentes
        let smart_title = gtk4::Label::new(Some("Playlists Inteligentes"));
        smart_title.add_css_class("title-2");
        smart_title.set_halign(gtk4::Align::Start);
        overview_box.append(&smart_title);

        let smart_list = gtk4::ListBox::new();
        smart_list.add_css_class("boxed-list");
        smart_list.set_selection_mode(gtk4::SelectionMode::None);

        let smart_playlists = [
            ("starred-symbolic", "Favoritas", "38 faixas", ""),
            ("history-symbolic", "Tocadas Recentemente", "50 faixas", ""),
            ("fire-symbolic", "Mais Tocadas", "25 faixas", ""),
            (
                "headphones-symbolic",
                "Lossless / Hi-Res Only",
                "64 faixas",
                "badge-hires",
            ),
        ];

        for (icon, name, count, badge_class) in smart_playlists {
            let row = adw::ActionRow::new();
            row.set_title(name);
            row.set_subtitle(count);

            let icon_widget = gtk4::Image::from_icon_name(icon);
            row.add_prefix(&icon_widget);

            if !badge_class.is_empty() {
                let badge = gtk4::Label::new(Some("Hi-Res"));
                badge.add_css_class("badge");
                badge.add_css_class(badge_class);
                row.add_suffix(&badge);
            }
            smart_list.append(&row);
        }
        overview_box.append(&smart_list);

        // Seção: Minhas Listas
        let my_header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        my_header_box.set_halign(gtk4::Align::Fill);
        my_header_box.set_margin_top(16);

        let my_title = gtk4::Label::new(Some("Minhas Listas"));
        my_title.add_css_class("title-2");
        my_title.set_halign(gtk4::Align::Start);
        my_title.set_hexpand(true);
        my_header_box.append(&my_title);

        overview_box.append(&my_header_box);

        // Botão Nova Playlist (azul Fedora)
        let new_playlist_btn = gtk4::Button::new();

        let button_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

        let icon = gtk4::Image::from_icon_name("list-add-symbolic");
        let label = gtk4::Label::new(Some("Nova Playlist"));

        button_content.append(&icon);
        button_content.append(&label);

        new_playlist_btn.set_child(Some(&button_content));
        new_playlist_btn.add_css_class("suggested-action");
        new_playlist_btn.add_css_class("pill");
        new_playlist_btn.set_tooltip_text(Some("Criar nova playlist"));

        // TODO: Conectar ao diálogo de criação de playlist
        my_header_box.append(&new_playlist_btn);

        let my_list = gtk4::ListBox::new();
        my_list.add_css_class("boxed-list");
        my_list.set_selection_mode(gtk4::SelectionMode::None);

        let my_playlists = [
            (
                "Synthwave Night Drive",
                "18 faixas • 1h 14m",
                "Fedora 40 FLAC",
            ),
            ("Foco &amp; Programação", "32 faixas • 2h 45m", ""),
            ("Ambient Soundscapes", "14 faixas • 1h 02m", ""),
        ];

        for (name, meta, tag) in my_playlists {
            let row = adw::ActionRow::new();
            row.set_title(name);
            row.set_subtitle(meta);

            let icon_widget = gtk4::Image::from_icon_name("queue-music-symbolic");
            row.add_prefix(&icon_widget);

            if !tag.is_empty() {
                let badge = gtk4::Label::new(Some(tag));
                badge.add_css_class("badge");
                row.add_suffix(&badge);
            }

            let stack_clone = stack.clone();
            row.connect_activated(move |_| {
                stack_clone.set_visible_child_name("detail");
            });

            my_list.append(&row);
        }
        overview_box.append(&my_list);

        overview_scroll.set_child(Some(&overview_box));

        // ==========================================
        // 2. VISÃO DE DETALHE (Playlist Específica)
        // ==========================================
        let detail_scroll = gtk4::ScrolledWindow::new();
        detail_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        detail_scroll.set_vexpand(true);
        detail_scroll.add_css_class("view-surface");

        let detail_clamp = adw::Clamp::new();
        detail_clamp.set_maximum_size(900);

        let detail_box = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        detail_box.set_margin_top(16);
        detail_box.set_margin_bottom(16);
        detail_box.set_margin_start(16);
        detail_box.set_margin_end(16);

        // Botão Voltar
        let back_btn = gtk4::Button::from_icon_name("go-previous-symbolic");
        back_btn.add_css_class("flat");
        back_btn.set_label("Voltar");
        back_btn.set_halign(gtk4::Align::Start);
        let stack_clone_back = stack.clone();
        back_btn.connect_clicked(move |_| {
            stack_clone_back.set_visible_child_name("overview");
        });
        detail_box.append(&back_btn);

        // Título da Playlist
        let playlist_title = gtk4::Label::new(Some("Synthwave Night Drive"));
        playlist_title.add_css_class("title-1");
        playlist_title.set_halign(gtk4::Align::Start);
        detail_box.append(&playlist_title);

        let playlist_meta =
            gtk4::Label::new(Some("Criado localmente • 18 músicas, 1h 14m • 840 MB FLAC"));
        playlist_meta.add_css_class("dim-label");
        playlist_meta.set_halign(gtk4::Align::Start);
        detail_box.append(&playlist_meta);

        // Badges e Info do Topo
        let info_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        info_box.set_halign(gtk4::Align::Start);

        let hires_badge = gtk4::Label::new(Some("Lossless 24-bit / 96kHz"));
        hires_badge.add_css_class("badge");
        hires_badge.add_css_class("badge-hires");

        let folder_badge = gtk4::Label::new(Some("~/Música/Playlists/synthwave.m3u8"));
        folder_badge.add_css_class("dim-label");

        info_box.append(&hires_badge);
        info_box.append(&folder_badge);
        detail_box.append(&info_box);

        // Lista de Faixas da Playlist
        let track_list = gtk4::ListBox::new();
        track_list.add_css_class("boxed-list");
        track_list.set_selection_mode(gtk4::SelectionMode::None);

        let tracks = [
            (1, "Resonance", "HOME", "Odyssey", "3:32", "FLAC", false),
            (
                2,
                "Midnight City",
                "M83",
                "Hurry Up, We're Dreaming",
                "4:04",
                "24/96",
                true,
            ),
            (
                3,
                "Days of Thunder",
                "The Midnight",
                "Days of Thunder EP",
                "5:48",
                "FLAC",
                false,
            ),
            (4, "Tech Noir", "Gunship", "Gunship", "4:57", "FLAC", false),
        ];

        for (num, title, artist, album, duration, format, is_playing) in tracks {
            let row = gtk4::ListBoxRow::new();
            row.add_css_class("track-row");
            if is_playing {
                row.add_css_class("playing");
            }

            let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
            row_box.set_margin_start(12);
            row_box.set_margin_end(12);
            row_box.set_margin_top(8);
            row_box.set_margin_bottom(8);

            let drag_icon = gtk4::Image::from_icon_name("open-menu-symbolic");
            drag_icon.add_css_class("dim-label");
            row_box.append(&drag_icon);

            let num_label = if is_playing {
                let img = gtk4::Image::from_icon_name("graphic-equalizer-symbolic");
                img.add_css_class("badge-hires");
                img.set_pixel_size(16);
                gtk4::Widget::from(img)
            } else {
                let lbl = gtk4::Label::new(Some(&num.to_string()));
                lbl.add_css_class("numeric-mono");
                lbl.set_width_chars(2);
                gtk4::Widget::from(lbl)
            };
            row_box.append(&num_label);

            let cover = gtk4::Image::from_icon_name("audio-x-generic-symbolic");
            cover.set_pixel_size(40);
            cover.add_css_class("cover");
            row_box.append(&cover);

            let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
            text_box.set_vexpand(true);
            text_box.set_valign(gtk4::Align::Center);
            text_box.set_hexpand(true);

            let title_lbl = gtk4::Label::new(Some(title));
            title_lbl.add_css_class("track-title");
            title_lbl.set_halign(gtk4::Align::Start);

            let artist_lbl = gtk4::Label::new(Some(&format!("{} • {}", artist, album)));
            artist_lbl.add_css_class("track-artist");
            artist_lbl.set_halign(gtk4::Align::Start);

            text_box.append(&title_lbl);
            text_box.append(&artist_lbl);
            row_box.append(&text_box);

            let right_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            right_box.set_valign(gtk4::Align::Center);

            let format_lbl = gtk4::Label::new(Some(format));
            format_lbl.add_css_class("badge");
            if format == "24/96" {
                format_lbl.add_css_class("badge-hires");
            }
            right_box.append(&format_lbl);

            let dur_lbl = gtk4::Label::new(Some(duration));
            dur_lbl.add_css_class("numeric-mono");
            right_box.append(&dur_lbl);

            row_box.append(&right_box);
            row.set_child(Some(&row_box));
            track_list.append(&row);
        }
        detail_box.append(&track_list);

        let footer_info = gtk4::Label::new(Some(
            "Arraste uma faixa para reordenar  •  6 de 18 faixas listadas",
        ));
        footer_info.add_css_class("caption");
        footer_info.add_css_class("dim-label");
        footer_info.set_halign(gtk4::Align::Center);
        footer_info.set_margin_top(8);
        detail_box.append(&footer_info);

        detail_clamp.set_child(Some(&detail_box));
        detail_scroll.set_child(Some(&detail_clamp));

        // ==========================================
        // 3. MONTAGEM FINAL NO STACK
        // ==========================================
        stack.add_named(&overview_scroll, Some("overview"));
        stack.add_named(&detail_scroll, Some("detail"));
        stack.set_visible_child_name("overview");

        // Box principal que encapsula o stack
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.append(&stack);

        Self {
            widget: main_box,
            stack,
        }
    }
}
