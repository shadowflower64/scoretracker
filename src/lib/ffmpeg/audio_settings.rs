use crate::formats;
use std::fmt;

pub enum AudioEncoder {
    Copy, // -c:a copy
    Opus, // -c:a libopus
}

impl fmt::Display for AudioEncoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Copy => "copy",
            Self::Opus => "libopus",
        };
        write!(f, "{str}")
    }
}

pub enum BitrateUnit {
    Kbps,
}

impl fmt::Display for BitrateUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kbps => write!(f, "k"),
        }
    }
}

pub struct Bitrate {
    value: u32,
    unit: BitrateUnit,
}

impl Bitrate {
    pub fn kbps(kbps: u32) -> Self {
        Self {
            value: kbps,
            unit: BitrateUnit::Kbps,
        }
    }
}

impl fmt::Display for Bitrate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.value, self.unit)
    }
}

/// Audio encoding settings for ffmpeg.
///
/// # Examples
/// ```
/// use scoretracker::ffmpeg::audio_settings::{AudioSettings, AudioEncoder, Bitrate};
///
/// let audio_settings = AudioSettings { encoder: AudioEncoder::Copy, bitrate: None };
/// assert_eq!(&audio_settings.to_string(), "-c:a copy");
///
/// let audio_settings = AudioSettings { encoder: AudioEncoder::Opus, bitrate: Some(Bitrate::kbps(32)) };
/// assert_eq!(&audio_settings.to_string(), "-c:a libopus -b:a 32k");
///
/// let mut args = Vec::new();
/// let audio_settings = AudioSettings { encoder: AudioEncoder::Opus, bitrate: Some(Bitrate::kbps(32)) };
/// audio_settings.append_args(&mut args);
/// assert_eq!(args.join(" "), "-c:a libopus -b:a 32k");
/// ```
pub struct AudioSettings {
    pub encoder: AudioEncoder,
    pub bitrate: Option<Bitrate>,
}

impl AudioSettings {
    pub fn copy() -> Self {
        Self {
            encoder: AudioEncoder::Copy,
            bitrate: None,
        }
    }

    pub fn append_args(&self, args: &mut Vec<String>) {
        let encoder = &self.encoder;
        args.extend_from_slice(&formats!("-c:a", "{encoder}"));

        if let Some(bitrate) = &self.bitrate {
            args.extend_from_slice(&formats!("-b:a", "{bitrate}"));
        }
    }
}

impl fmt::Display for AudioSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = Vec::new();
        self.append_args(&mut args);
        write!(f, "{}", args.join(" "))
    }
}
