//! Empty state view

use gtk4::prelude::*;
use libadwaita as adw;

/// View de biblioteca vazia
pub struct EmptyView {
    pub widget: adw::Clamp, // ← MUDAR DE Box PARA Clamp
}

impl EmptyView {
    /// Cria uma nova empty view
    pub fn new() -> Self {
        let clamp = adw::Clamp::new();
        clamp.set_maximum_size(600);

        let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        box_.add_css_class("empty-state");
        box_.set_valign(gtk4::Align::Center);
        box_.set_vexpand(true);

        // Ícone grande
        let icon = gtk4::Image::from_icon_name("folder-open-symbolic");
        icon.set_pixel_size(128);
        icon.add_css_class("empty-icon");

        // Título
        let title = gtk4::Label::new(Some("Sua biblioteca está vazia"));
        title.add_css_class("empty-title");

        // Subtítulo
        let subtitle = gtk4::Label::new(Some(
            "Adicione suas pastas de música ou arquivos MP3/M4A para começar.",
        ));
        subtitle.add_css_class("empty-subtitle");
        subtitle.set_justify(gtk4::Justification::Center);
        subtitle.set_wrap(true);

        // Botão principal
        let add_button = gtk4::Button::with_label("Adicionar Pasta de Música");
        add_button.add_css_class("suggested-action");
        add_button.add_css_class("pill");

        // Texto de drag & drop
        let drag_label = gtk4::Label::new(Some("Ou arraste arquivos do Nautilus aqui"));
        drag_label.add_css_class("dim-label");
        drag_label.set_margin_top(16);

        let formats_label = gtk4::Label::new(Some("Suporta FLAC, MP3, AAC, OGG Opus e WAV"));
        formats_label.add_css_class("caption");
        formats_label.set_margin_top(4);

        // Card de dicas
        let tips_card = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        tips_card.add_css_class("card-surface");
        tips_card.set_margin_top(32);
        tips_card.set_margin_start(16);
        tips_card.set_margin_end(16);
        tips_card.set_margin_bottom(16);

        let tips_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let tips_icon = gtk4::Image::from_icon_name("info-symbolic");
        let tips_title = gtk4::Label::new(Some("Dica do Fedplay"));
        tips_title.add_css_class("title-4");
        tips_header.append(&tips_icon);
        tips_header.append(&tips_title);

        let tip1 = gtk4::Label::new(Some(
            "🎵 Lossless: A integração nativa com o PipeWire garante reprodução Bit-Perfect com amostragem direta.",
        ));
        tip1.add_css_class("dim-label");
        tip1.set_wrap(true);
        tip1.set_halign(gtk4::Align::Start);

        let tip2 = gtk4::Label::new(Some(
            "🎛️ Equalizador 10-Bandas  •  🔧 ReplayGain  •  ⚡ Fedora 40 Ready",
        ));
        tip2.add_css_class("dim-label");
        tip2.set_wrap(true);
        tip2.set_halign(gtk4::Align::Start);

        tips_card.append(&tips_header);
        tips_card.append(&tip1);
        tips_card.append(&tip2);

        box_.append(&icon);
        box_.append(&title);
        box_.append(&subtitle);
        box_.append(&add_button);
        box_.append(&drag_label);
        box_.append(&formats_label);
        box_.append(&tips_card);

        clamp.set_child(Some(&box_));

        Self { widget: clamp } // ← Retornar o Clamp, não o Box
    }
}
