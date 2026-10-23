//! Fedplay - Reprodutor de áudio para Fedora
//!
//! Aplicação principal com interface GTK4 + Libadwaita

mod app;
mod views;
mod widgets;

use app::FedplayApp;
use gtk4::prelude::*;
use libadwaita as adw;

fn main() {
    // Criar aplicação Adwaita
    let app = adw::Application::builder()
        .application_id("com.github.fedplay.Fedplay")
        .build();

    app.connect_activate(|app| {
        let fedplay_app = FedplayApp::new(app);
        fedplay_app.build_ui();
    });

    app.run();
}
