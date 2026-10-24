// fedplay-ui/src/views/tag_editor_view.rs
use gtk4::prelude::*;
use libadwaita as adw;

pub struct TagEditorDialog {
    pub widget: adw::Window,
}

impl TagEditorDialog {
    pub fn new() -> Self {
        let window = adw::Window::builder()
            .title("Editar Metadados")
            .default_width(500)
            .default_height(600)
            .modal(true)
            .build();
        Self { widget: window }
    }
}
