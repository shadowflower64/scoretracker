use core::fmt;

use crate::formats;

pub enum VideoEncoder {
    Copy, // -c:v copy
    H264, // -c:v libx264
    H265, // -c:v libx265
}

impl fmt::Display for VideoEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Copy => "copy",
            Self::H264 => "libx264",
            Self::H265 => "libx265",
        };
        write!(f, "{str}")
    }
}

pub enum CpuPreset {
    UltraFast,
    SuperFast,
    VeryFast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    VerySlow,
    Placebo,
}

impl fmt::Display for CpuPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::UltraFast => "ultrafast",
            Self::SuperFast => "superfast",
            Self::VeryFast => "veryfast",
            Self::Faster => "faster",
            Self::Fast => "fast",
            Self::Medium => "medium",
            Self::Slow => "slow",
            Self::Slower => "slower",
            Self::VerySlow => "veryslow",
            Self::Placebo => "placebo",
        };
        write!(f, "{str}")
    }
}

pub struct VideoSettings {
    pub encoder: VideoEncoder,

    /// Constant rate factor. Quality value from 0-51. 0 = lossless, 51 = worst quality possible.
    ///
    /// Default value is 23 (for H.264), or 28 (for H.265), which correspond to about the same visual quality.
    /// "Subjectively sane range" for H.264 is 17-28.
    ///
    /// https://trac.ffmpeg.org/wiki/Encode/H.264#crf
    /// https://trac.ffmpeg.org/wiki/Encode/H.265#ConstantRateFactorCRF
    pub crf: Option<u8>,

    /// How fast should the encoding work. Faster = lower quality.
    pub preset: Option<CpuPreset>,

    /// Output resolution. May contain negative values as "auto" settings.
    ///
    /// [`None`] if the resolution should be unchanged.
    pub output_resolution: Option<(i32, i32)>,
}

impl VideoSettings {
    pub fn copy() -> Self {
        Self {
            encoder: VideoEncoder::Copy,
            crf: None,
            preset: None,
            output_resolution: None,
        }
    }

    pub fn append_args(&self, args: &mut Vec<String>) {
        // -c:v libx265 -crf 38 -preset slow -filter:v scale=-1:360
        let encoder = &self.encoder;
        args.extend_from_slice(&formats!["-c:v", "{encoder}"]);

        if let Some(crf) = self.crf {
            args.extend_from_slice(&formats!["-crf", "{crf}"]);
        }

        if let Some(preset) = &self.preset {
            args.extend_from_slice(&formats!["-preset", "{preset}"]);
        }
        if let Some((w, h)) = self.output_resolution {
            args.extend_from_slice(&formats!["-filter:v", "scale={w}:{h}"]);
        }
    }
}

impl fmt::Display for VideoSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = Vec::new();
        self.append_args(&mut args);
        write!(f, "{}", args.join(" "))
    }
}
