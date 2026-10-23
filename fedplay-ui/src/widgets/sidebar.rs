//! Sidebar de navegação do Fedplay

use gtk4::prelude::*;
use libadwaita as adw;

/// Item de navegação da sidebar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavItem {
    Music,
    Artists,
    Albums,
    Playlists,
}

impl NavItem {
    /// Converte o índice da linha do ListBox em um NavItem
    pub fn from_index(index: i32) -> Option<Self> {
        match index {
            0 => Some(NavItem::Music),
            1 => Some(NavItem::Artists),
            2 => Some(NavItem::Albums),
            3 => Some(NavItem::Playlists),
            _ => None,
        }
    }
}

/// Sidebar do Fedplay
pub struct Sidebar {
    pub widget: adw::ToolbarView,
    pub list_box: gtk4::ListBox,
}

impl Sidebar {
    /// Cria uma nova sidebar
    pub fn new() -> Self {
        let list_box = gtk4::ListBox::new();
        list_box.add_css_class("fedplay-sidebar");
        list_box.add_css_class("navigation-sidebar");
        list_box.set_selection_mode(gtk4::SelectionMode::Single);

        // Header com logo
        let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        header_box.set_margin_start(16);
        header_box.set_margin_end(16);
        header_box.set_margin_top(16);
        header_box.set_margin_bottom(8);

        let logo_icon = gtk4::Image::from_icon_name("audio-x-generic-symbolic");
        logo_icon.set_pixel_size(32);

        let logo_label = gtk4::Label::new(Some("Fedplay"));
        logo_label.add_css_class("title-2");
        logo_label.set_halign(gtk4::Align::Start);
        logo_label.set_hexpand(true);

        header_box.append(&logo_icon);
        header_box.append(&logo_label);

        // Search entry
        let search = gtk4::SearchEntry::new();
        search.set_placeholder_text(Some("Buscar na biblioteca"));
        search.set_margin_start(12);
        search.set_margin_end(12);
        search.set_margin_bottom(12);

        // Itens de navegação (ordem define o índice: 0=Músicas, 1=Artistas, etc.)
        let nav_items = [
            ("music-note-symbolic", "Músicas"),
            ("person-symbolic", "Artistas"),
            ("album-symbolic", "Álbuns"),
            ("queue-music-symbolic", "Listas de Reprodução"),
        ];

        for (icon_name, label) in nav_items {
            let row = gtk4::ListBoxRow::new();
            row.add_css_class("nav-item");

            let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
            row_box.set_margin_start(8);
            row_box.set_margin_end(8);
            row_box.set_margin_top(6);
            row_box.set_margin_bottom(6);

            let icon = gtk4::Image::from_icon_name(icon_name);
            icon.set_pixel_size(20);

            let label_widget = gtk4::Label::new(Some(label));
            label_widget.set_halign(gtk4::Align::Start);
            label_widget.set_hexpand(true);
            label_widget.add_css_class("body-medium");

            row_box.append(&icon);
            row_box.append(&label_widget);
            row.set_child(Some(&row_box));

            list_box.append(&row);
        }

        // Selecionar primeiro item por padrão
        if let Some(first_row) = list_box.row_at_index(0) {
            list_box.select_row(Some(&first_row));
        }

        // Status card (PipeWire Engine)
        let status_card = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        status_card.add_css_class("status-card");
        status_card.set_margin_start(8);
        status_card.set_margin_end(8);
        status_card.set_margin_top(16);

        let status_title = gtk4::Label::new(Some("PipeWire Engine"));
        status_title.add_css_class("status-title");
        status_title.set_halign(gtk4::Align::Start);

        let status_value = gtk4::Label::new(Some("BIT-PERFECT 24/96"));
        status_value.add_css_class("status-value");
        status_value.set_halign(gtk4::Align::Start);

        status_card.append(&status_title);
        status_card.append(&status_value);

        // Montar layout vertical
        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        content_box.append(&header_box);
        content_box.append(&search);
        content_box.append(&list_box);
        content_box.append(&status_card);
        content_box.set_vexpand(true);

        // ToolbarView
        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.set_content(Some(&content_box));

        Self {
            widget: toolbar_view,
            list_box,
        }
    }
}
