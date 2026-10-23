// ! View de edição de metadados

use gtk4::prelude::*;
use libadwaita as adw;

/// Dialog de edição de metadados
pub struct TagEditorDialog {
    pub widget: adw::Window,
}

impl TagEditorDialog {
    /// Cria um novo dialog de edição
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
