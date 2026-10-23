//! Implementação do player usando GStreamer

use fedplay_core::{PlaybackState, PlayerState, Track};
use gstreamer::prelude::*;
use gstreamer::{ElementFactory, Pipeline, State as GstState};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Falha ao inicializar GStreamer")]
    InitError,

    #[error("Falha ao criar pipeline")]
    PipelineError,

    #[error("Arquivo não encontrado: {0}")]
    FileNotFound(String),

    #[error("Formato não suportado")]
    UnsupportedFormat,
}

/// Player de áudio usando GStreamer
pub struct GStreamerPlayer {
    pipeline: Option<Pipeline>,
    state: Arc<Mutex<PlayerState>>,
}

impl GStreamerPlayer {
    /// Cria uma nova instância do player
    pub fn new() -> Result<Self, AudioError> {
        // Inicializar GStreamer
        gstreamer::init().map_err(|_| AudioError::InitError)?;

        Ok(Self {
            pipeline: None,
            state: Arc::new(Mutex::new(PlayerState::default())),
        })
    }

    /// Carrega uma faixa para reprodução
    pub fn load(&mut self, track: &Track) -> Result<(), AudioError> {
        // Verificar se o arquivo existe
        if !track.path.exists() {
            return Err(AudioError::FileNotFound(
                track.path.to_string_lossy().to_string(),
            ));
        }

        // Criar pipeline
        let pipeline = ElementFactory::make("playbin")
            .build()
            .map_err(|_| AudioError::PipelineError)?
            .downcast::<Pipeline>()
            .map_err(|_| AudioError::PipelineError)?;

        // Obter o caminho absoluto e convertê-lo para URI
        let uri = track
            .path
            .canonicalize()
            .map_err(|_| AudioError::PipelineError)?;

        let uri = url::Url::from_file_path(&uri).map_err(|_| AudioError::PipelineError)?;

        // Definir URI do arquivo
        pipeline.set_property("uri", uri.as_str());

        self.pipeline = Some(pipeline);

        // Atualizar estado
        let mut state = self.state.lock().unwrap();
        state.current_track_id = Some(track.id);
        state.playback = PlaybackState::Stopped;
        state.position_ms = 0;

        Ok(())
    }

    /// Inicia a reprodução
    pub fn play(&self) -> Result<(), AudioError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline
                .set_state(GstState::Playing)
                .map_err(|_| AudioError::PipelineError)?;

            let mut state = self.state.lock().unwrap();
            state.playback = PlaybackState::Playing;

            Ok(())
        } else {
            Err(AudioError::PipelineError)
        }
    }

    /// Pausa a reprodução
    pub fn pause(&self) -> Result<(), AudioError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline
                .set_state(GstState::Paused)
                .map_err(|_| AudioError::PipelineError)?;

            let mut state = self.state.lock().unwrap();
            state.playback = PlaybackState::Paused;

            Ok(())
        } else {
            Err(AudioError::PipelineError)
        }
    }

    /// Para a reprodução
    pub fn stop(&self) -> Result<(), AudioError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline
                .set_state(GstState::Null)
                .map_err(|_| AudioError::PipelineError)?;

            let mut state = self.state.lock().unwrap();
            state.playback = PlaybackState::Stopped;
            state.position_ms = 0;

            Ok(())
        } else {
            Err(AudioError::PipelineError)
        }
    }

    /// Define o volume (0.0 a 1.0)
    pub fn set_volume(&self, volume: f64) -> Result<(), AudioError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_property("volume", &volume.clamp(0.0, 1.0));

            let mut state = self.state.lock().unwrap();
            state.volume = volume.clamp(0.0, 1.0);

            Ok(())
        } else {
            Err(AudioError::PipelineError)
        }
    }

    /// Retorna o estado atual do player
    pub fn get_state(&self) -> PlayerState {
        self.state.lock().unwrap().clone()
    }

    /// Busca a posição atual da reprodução
    pub fn seek(&self, position_ms: u64) -> Result<(), AudioError> {
        if let Some(pipeline) = &self.pipeline {
            let position = gstreamer::ClockTime::from_mseconds(position_ms);
            pipeline
                .seek_simple(
                    gstreamer::SeekFlags::FLUSH | gstreamer::SeekFlags::KEY_UNIT,
                    position,
                )
                .map_err(|_| AudioError::PipelineError)?;

            let mut state = self.state.lock().unwrap();
            state.position_ms = position_ms;

            Ok(())
        } else {
            Err(AudioError::PipelineError)
        }
    }
}

impl Drop for GStreamerPlayer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = GStreamerPlayer::new();
        assert!(player.is_ok());
    }

    #[test]
    fn test_player_state_initial() {
        let player = GStreamerPlayer::new().unwrap();
        let state = player.get_state();

        assert_eq!(state.playback, PlaybackState::Stopped);
        assert_eq!(state.current_track_id, None);
        assert_eq!(state.volume, 1.0);
    }
}
