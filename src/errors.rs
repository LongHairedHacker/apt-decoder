use hound;
use image;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecoderError {
    #[error("Unable to read input file: {0}")]
    InputFileError(#[from] hound::Error),

    #[error("Expected a sampling rate of 48000Hz not {0}Hz")]
    UnexpectedSamplingRate(u32),

    #[error("Unable to write output file: {0}")]
    OutputFileError(#[from] image::ImageError),

    #[error("FIXME: Unknown decoder error")]
    Unknown,
}
