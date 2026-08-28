//! Process (compress) a video from the library and save result to library
use crate::data::library::database::QualityState;
use crate::data::library::{create_stpl_url_to_relfile, get_library_dir_of_path, path_within_library_dir, scan_register_added_file};
use crate::ffmpeg::audio_settings::{AudioEncoder, AudioSettings, Bitrate};
use crate::ffmpeg::video_settings::{CpuPreset, VideoEncoder, VideoSettings};
use crate::ffmpeg::{ffmpeg_process_video, get_version};
use crate::hive::job::{AnyJob, Fail, Job, Success};
use crate::hive::worker::Worker;
use crate::util::uuid::UuidString;
use crate::{info, log_fn_name};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{path::PathBuf, sync::Arc};

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessLibraryVideoJob {
    pub source_path: PathBuf,
    pub source_proof_uuid_precondition_check: Option<UuidString>,
    #[serde(alias = "processing_type")]
    pub operation: Operation,
    pub destination_path: PathBuf,
}

impl Job for ProcessLibraryVideoJob {
    async fn run(&self, worker: Arc<Worker>) -> Result<Success, Fail> {
        log_fn_name!("job:process_library_video");

        let ffmpeg_version = get_version().await;
        info!("ffmpeg version: {:?}", ffmpeg_version);

        let worker_info = Some(worker.info());
        let config = worker.config();

        // Find library directory of the source proof file
        let source_library_dir = get_library_dir_of_path(&self.source_path).ok_or_else(|| Fail::PathNotInLibraryRepo {
            path: self.source_path.clone(),
        })?;
        let source_relpath =
            path_within_library_dir(&source_library_dir, &self.source_path).ok_or_else(|| Fail::CannotFindPathWithinLibraryDir {
                library_dir: source_library_dir.clone(),
                target_file_path: self.source_path.clone(),
            })?;

        // Fetch the source proof UUID from the local index
        let source_library_index = worker.read_library_index(&source_library_dir)?;
        let source_proof_uuid = source_library_index
            .files
            .get(&source_relpath)
            .ok_or_else(|| Fail::FileNotInIndex {
                library_dir: source_library_dir.clone(),
                target_relpath: source_relpath.clone(),
            })?
            .0;

        // Check if it matches the precondition (if it doesn't, that means that the input request was incorrect!)
        if let Some(precondition_uuid) = self.source_proof_uuid_precondition_check
            && source_proof_uuid != precondition_uuid.0
        {
            Err(Fail::PreconditionUuidDoesNotMatch {
                file_path: self.source_path.clone(),
                read_proof_uuid: source_proof_uuid.into(),
                precondition_uuid,
            })?
        }

        // Get source proof entry from the database
        let proof_entry = worker
            .fetch_library_entry_by_uuid(source_proof_uuid)
            .await?
            .ok_or(Fail::EntryNotFound(source_proof_uuid.into()))?;

        // Sanity check - proof entry with the given UUID should contain the given URL.
        // If it doesn't, that means the library needs to be rescanned or the request was invalid.
        let source_library_info = worker.read_library_info(&source_library_dir)?;
        let file_url = create_stpl_url_to_relfile(source_library_info, source_relpath);
        if !proof_entry.library_urls.contains(&file_url) {
            Err(Fail::FileUrlNotFoundInLibraryEntry {
                expected_url: file_url,
                entry: Box::new(proof_entry.to_owned()),
            })?
        }

        // Launch ffmpeg to cut the video losslessly
        let output_path = worker.create_temp_path();
        let worker2 = Arc::clone(&worker);
        ffmpeg_process_video(&self.source_path, &output_path, self.operation, move |progress| {
            worker2.update_task_progress_very_simple(format!("{progress:?}"));
        })
        .await?;

        // Get library info of the destination file
        let wet = if let Some(destination_library_dir) = get_library_dir_of_path(&self.source_path) {
            // Register new file to index and to database
            worker.upload_file_to_library(self.destination_path);

            let (_rel_path, uuid) = scan_register_added_file(
                &destination_library_dir,
                &config.library_database_path(),
                &self.destination_path,
                |entry| {
                    entry.dry = Some(source_proof_uuid.into());
                    entry.quality = self.operation.resulting_quality_state()
                },
                worker_info,
            )
            .map_err(|e| Fail::CannotRegisterFileIntoLibrary {
                file_path: self.destination_path.to_owned(),
                reason: e.to_string(),
            })?;
            Some(uuid.into())
        } else {
            None
        };

        Ok(Success::ProcessedVideo {
            dry: source_proof_uuid.into(),
            wet,
        })
    }
    fn into_any(self) -> AnyJob {
        AnyJob::ProcessLibraryVideo(self)
    }
}
