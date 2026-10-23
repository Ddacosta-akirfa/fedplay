//! Fedplay Audio - Backend de áudio usando GStreamer
//! 
//! Este módulo fornece a implementação de reprodução de áudio
//! usando GStreamer como backend.

pub mod gstreamer_backend;

pub use gstreamer_backend::GStreamerPlayer;