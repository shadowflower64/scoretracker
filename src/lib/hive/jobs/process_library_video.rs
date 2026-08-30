//! Process (compress) a video from the library and save result to library
use crate::data::library::database::QualityState;
use crate::data::library::stpl_url::StplUrl;
use crate::ffmpeg::audio_settings::{AudioEncoder, AudioSettings, Bitrate};
use crate::ffmpeg::video_settings::{CpuPreset, VideoEncoder, VideoSettings};
use crate::ffmpeg::{ffmpeg_process_video, get_version};
use crate::hive::job::{AnyJob, Fail, Job, Success};
use crate::hive::worker::Worker;
use crate::util::uuid::UuidString;
use crate::{info, log_fn_name};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessLibraryVideoJob {
    pub source: StplUrl,
    pub source_proof_uuid_precondition_check: Option<UuidString>,
    pub destination: StplUrl,

    #[serde(alias = "processing_type")]
    pub operation: Operation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Turn a video into the [`QualityState::Folded`] state.
    CompressFoldVideo,

    /// Turn a video into the [`QualityState::Messy`] state.
    CompressMessUpVideo,

    /// Turn a video into the [`QualityState::Crumpled`] state.
    CompressCrumpleVideo,

    /// Turn a video into the [`QualityState::Shredded`] state.
    CompressShredVideo,
}

impl Operation {
    /// Create audio codec options for ffmpeg.
    pub fn audio_settings(&self) -> AudioSettings {
        match self {
            Self::CompressFoldVideo | Self::CompressMessUpVideo => AudioSettings {
                encoder: AudioEncoder::Copy,
                bitrate: None,
            },
            Self::CompressCrumpleVideo => AudioSettings {
                encoder: AudioEncoder::Opus,
                bitrate: Some(Bitrate::kbps(32)),
            },
            Self::CompressShredVideo => AudioSettings {
                encoder: AudioEncoder::Opus,
                bitrate: Some(Bitrate::kbps(16)),
            },
        }
    }

    /// Returns [`true`] if this operation should preserve all video and audio streams
    /// Returns [`false`] if this operation should only include one video and one audio stream in the final output video.
    pub fn preserve_all_streams(&self) -> bool {
        match self {
            Self::CompressFoldVideo | Self::CompressMessUpVideo | Self::CompressCrumpleVideo => true,
            Self::CompressShredVideo => false,
        }
    }

    /// Create video codec options, as well as a list of video filters for ffmpeg.
    pub fn video_settings(&self) -> VideoSettings {
        match self {
            Self::CompressFoldVideo => VideoSettings {
                encoder: VideoEncoder::H265,
                crf: Some(26),
                preset: Some(CpuPreset::Slow),
                output_resolution: None,
            },
            Self::CompressMessUpVideo => VideoSettings {
                encoder: VideoEncoder::H265,
                crf: Some(29),
                preset: Some(CpuPreset::Slower),
                output_resolution: Some((-1, 720)),
            },
            Self::CompressCrumpleVideo => VideoSettings {
                encoder: VideoEncoder::H265,
                crf: Some(35),
                preset: Some(CpuPreset::Slow),
                output_resolution: Some((-2, 480)),
            },
            Self::CompressShredVideo => VideoSettings {
                encoder: VideoEncoder::H265,
                crf: Some(38),
                preset: Some(CpuPreset::Slow),
                output_resolution: Some((-1, 360)),
            },
        }
    }

    /// Fetch the [`QualityState`] of the resulting video after this operation is concluded.
    pub fn resulting_quality_state(&self) -> QualityState {
        match self {
            Self::CompressFoldVideo => QualityState::Folded,
            Self::CompressMessUpVideo => QualityState::Messy,
            Self::CompressCrumpleVideo => QualityState::Crumpled,
            Self::CompressShredVideo => QualityState::Shredded,
        }
    }
}

impl FromStr for Operation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fold" => Ok(Self::CompressFoldVideo),
            "mess_up" => Ok(Self::CompressMessUpVideo),
            "crumple" => Ok(Self::CompressCrumpleVideo),
            "shred" => Ok(Self::CompressShredVideo),
            a => Err(format!("invalid processing type: {a}")),
        }
    }
}

impl Job for ProcessLibraryVideoJob {
    async fn run(&self, worker: Arc<Worker>) -> Result<Success, Fail> {
        log_fn_name!("job:process_library_video");

        let ffmpeg_version = get_version().await;
        info!("ffmpeg version: {:?}", ffmpeg_version);

        // Get source proof file
        let (source_path, dry_entry) = worker.find_or_download_proof_file(&self.source).await?;

        // Check if it matches the precondition (if it doesn't, that means that the input request was incorrect or has become invalid)
        if let Some(precondition_uuid) = self.source_proof_uuid_precondition_check
            && dry_entry.uuid != precondition_uuid
        {
            return Err(Fail::PreconditionUuidDoesNotMatch {
                stpl_url: self.source.clone(),
                read_proof_uuid: dry_entry.uuid,
                precondition_uuid,
            });
        }

        // Launch ffmpeg to cut the video losslessly
        let destination_path = worker.create_temp_path_for(&self.destination);
        let worker2 = Arc::clone(&worker);
        ffmpeg_process_video(&source_path, &destination_path, self.operation, async move |progress| {
            worker2.update_task_progress_very_simple(format!("{progress:?}")).await;
        })
        .await?;

        // Register new file into library
        let wet_entry = worker
            .move_or_upload_proof_file(&destination_path, &self.destination, |entry| {
                entry.dry = Some(dry_entry.uuid);
                entry.quality = self.operation.resulting_quality_state()
            })
            .await?;

        Ok(Success::ProcessedVideo {
            dry: dry_entry.uuid,
            wet: wet_entry.uuid,
        })
    }

    fn into_any(self) -> AnyJob {
        AnyJob::ProcessLibraryVideo(self)
    }
}
