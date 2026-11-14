//! Fedplay - Reprodutor de áudio para Fedora
//!
//! Aplicação principal com interface GTK4 + Libadwaita

mod app;
mod views;
mod widgets;

use app::FedplayApp;
use gtk4::gio::prelude::*;
use libadwaita as adw;

fn main() {
    // Criar aplicação Adwaita
    let app = adw::Application::builder()
        .application_id("com.github.fedplay.Fedplay")
        .build();

    app.connect_activate(|app| match FedplayApp::new(app) {
        Ok(fedplay_app) => fedplay_app.build_ui(),
        Err(error) => {
            eprintln!("Erro ao iniciar FedPlay: {error}");
        }
    });
}
