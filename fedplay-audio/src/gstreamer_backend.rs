//! Implementação do player usando GStreamer

use fedplay_core::{PlaybackState, PlayerState, Track};
use gstreamer::prelude::*;
use gstreamer::{Element, ElementFactory, State as GstState};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Falha ao inicializar GStreamer")]
    InitError,

    #[error("Falha ao criar o elemento de reprodução")]
    PipelineError,

    #[error("Arquivo não encontrado ou inacessível: {0}")]
    FileNotFound(String),

    #[error("Formato de áudio não suportado")]
    UnsupportedFormat,
}

/// Player de áudio usando um único elemento playbin.
pub struct GStreamerPlayer {
    player: Element,
    state: Arc<Mutex<PlayerState>>,
}

impl GStreamerPlayer {
    /// Cria uma única instância do elemento de reprodução.
    pub fn new() -> Result<Self, AudioError> {
        gstreamer::init().map_err(|_| AudioError::InitError)?;

        let player = ElementFactory::make("playbin")
            .build()
            .map_err(|_| AudioError::PipelineError)?;

        let state = Arc::new(Mutex::new(PlayerState::default()));

        Ok(Self { player, state })
    }

    /// Interrompe a faixa anterior e carrega uma nova.
    pub fn load(&mut self, track: &Track) -> Result<(), AudioError> {
        if !track.path.is_file() {
            return Err(AudioError::FileNotFound(
                track.path.to_string_lossy().into_owned(),
            ));
        }

        let path = track
            .path
            .canonicalize()
            .map_err(|_| AudioError::FileNotFound(track.path.to_string_lossy().into_owned()))?;

        let uri =
            gstreamer::glib::filename_to_uri(&path, None).map_err(|_| AudioError::PipelineError)?;

        // Parar completamente a reprodução anterior antes de trocar a URI.
        self.player
            .set_state(GstState::Null)
            .map_err(|_| AudioError::PipelineError)?;

        // Definir o novo ficheiro no mesmo elemento playbin.
        self.player.set_property("uri", uri.as_str());

        // Sincronizar o estado interno com a nova faixa.
        let mut state = self.state.lock().map_err(|_| AudioError::PipelineError)?;

        state.current_track_id = Some(track.id);
        state.playback = PlaybackState::Stopped;
        state.position_ms = 0;

        Ok(())
    }

    /// Inicia a reprodução da faixa carregada.
    pub fn play(&self) -> Result<(), AudioError> {
        self.player
            .set_state(GstState::Playing)
            .map_err(|_| AudioError::PipelineError)?;

        let mut state = self.state.lock().map_err(|_| AudioError::PipelineError)?;

        state.playback = PlaybackState::Playing;

        Ok(())
    }

    /// Pausa a reprodução atual.
    pub fn pause(&self) -> Result<(), AudioError> {
        self.player
            .set_state(GstState::Paused)
            .map_err(|_| AudioError::PipelineError)?;

        let mut state = self.state.lock().map_err(|_| AudioError::PipelineError)?;

        state.playback = PlaybackState::Paused;

        Ok(())
    }

    /// Para a reprodução e coloca o elemento em estado Null.
    pub fn stop(&self) -> Result<(), AudioError> {
        self.player
            .set_state(GstState::Null)
            .map_err(|_| AudioError::PipelineError)?;

        let mut state = self.state.lock().map_err(|_| AudioError::PipelineError)?;

        state.playback = PlaybackState::Stopped;
        state.position_ms = 0;

        Ok(())
    }

    /// Define o volume entre 0.0 e 1.0.
    pub fn set_volume(&self, volume: f64) -> Result<(), AudioError> {
        let volume = volume.clamp(0.0, 1.0);

        self.player.set_property("volume", volume);

        let mut state = self.state.lock().map_err(|_| AudioError::PipelineError)?;

        state.volume = volume;

        Ok(())
    }

    /// Devolve uma cópia do estado interno do leitor.
    pub fn get_state(&self) -> PlayerState {
        self.state
            .lock()
            .map(|state| state.clone())
            .unwrap_or_default()
    }

    /// Procura uma posição específica, em milissegundos.
    pub fn seek(&self, position_ms: u64) -> Result<(), AudioError> {
        let position = gstreamer::ClockTime::from_mseconds(position_ms);

        self.player
            .seek_simple(
                gstreamer::SeekFlags::FLUSH | gstreamer::SeekFlags::KEY_UNIT,
                position,
            )
            .map_err(|_| AudioError::PipelineError)?;

        let mut state = self.state.lock().map_err(|_| AudioError::PipelineError)?;

        state.position_ms = position_ms;

        Ok(())
    }
}

impl Drop for GStreamerPlayer {
    fn drop(&mut self) {
        // Libertar os recursos de áudio ao destruir o leitor.
        let _ = self.player.set_state(GstState::Null);
    }
}
