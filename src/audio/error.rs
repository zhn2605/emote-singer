use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("No default output device found")]
    NoDevice,

    #[error("Device error: {0}")]
    Device(#[from] cpal::DevicesError),

    #[error("Device description error: {0}")]
    DeviceDescription(#[from] cpal::DeviceNameError),

    #[error("No next supported config")]
    NoNextConfig,
    
    #[error("supported stream configs error: {0}")]
    SupportedConfigs(#[from] cpal::SupportedStreamConfigsError),

    #[error("{0}")]
    BuildStream(#[from] cpal::BuildStreamError),

    #[error("{0}")]
    PlayStream(#[from] cpal::PlayStreamError),

    #[error("{0}")]
    PauseStream(#[from] cpal::PauseStreamError)
}