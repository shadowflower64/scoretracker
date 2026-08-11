use crate::data::library::database::QualityState;
use crate::data::library::{create_stpl_url_to_relfile, get_library_dir_of_path, path_within_library_dir, scan_register_added_file};
use crate::ffmpeg::ffmpeg_process_video;
use crate::hive::job::{AnyJob, Fail, Job, Success};
use crate::hive::worker::Worker;
use crate::util::uuid::UuidString;
use rust_ffmpeg::{Codec, CodecOptions};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingType {
    CompressFoldVideo,
    CompressCrumpleVideo,
    CompressShredVideo,
}

impl ProcessingType {
    pub fn vcodec(&self) -> CodecOptions {
        CodecOptions::new(Codec::h265())
    }
    pub fn acodec(&self) -> CodecOptions {
        // TODO - which is better?? just copy and don't care about audio size, or actually compress audio?
        // CodecOptions::new(Codec::opus())
        // CodecOptions::new(Codec::copy())
        todo!()
    }
    pub fn resulting_quality_state(&self) -> QualityState {
        match self {
            Self::CompressFoldVideo => QualityState::Folded,
            Self::CompressCrumpleVideo => QualityState::Crumpled,
            Self::CompressShredVideo => QualityState::Shredded,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessLibraryVideoJob {
    pub source_path: PathBuf,
    pub source_proof_uuid_precondition_check: Option<UuidString>,
    pub processing_type: ProcessingType,
    pub destination_path: PathBuf,
}

impl Job for ProcessLibraryVideoJob {
    async fn run(&self, worker: Arc<Worker>) -> Result<Success, Fail> {
        let worker_info = Some(&worker.info_cloned());
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
            })?;

        // Check if it matches the precondition (if it doesn't, that means that the input request was incorrect!)
        if let Some(precondition_uuid) = self.source_proof_uuid_precondition_check
            && *source_proof_uuid != precondition_uuid
        {
            Err(Fail::PreconditionUuidDoesNotMatch {
                file_path: self.source_path.clone(),
                read_proof_uuid: *source_proof_uuid,
                precondition_uuid,
            })?
        }

        // Get source proof entry from the database
        let library_db = worker.read_library_db()?;
        let proof_entry = library_db
            .find_entry_by_uuid(*source_proof_uuid)
            .ok_or(Fail::EntryNotFound(*source_proof_uuid))?
            .to_owned();
        drop(library_db);

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
        let worker2 = Arc::clone(&worker);
        ffmpeg_process_video(&self.source_path, &self.destination_path, self.processing_type, move |progress| {
            worker2.update_task_progress_very_simple(format!("{progress:?}"));
        })
        .await?;

        // Get library info of the destination file
        let wet = if let Some(destination_library_dir) = get_library_dir_of_path(&self.source_path) {
            // Register new file to index and to database
            Some(
                scan_register_added_file(
                    &destination_library_dir,
                    &config.library_database_path(),
                    &self.destination_path,
                    |entry| {
                        entry.dry = Some(*source_proof_uuid);
                        entry.quality = self.processing_type.resulting_quality_state()
                    },
                    worker_info,
                )
                .map_err(|e| Fail::CannotRegisterFileIntoLibrary {
                    file_path: self.destination_path.to_owned(),
                    reason: e.to_string(),
                })?,
            )
        } else {
            None
        };

        Ok(Success::ProcessedVideo {
            dry: *source_proof_uuid,
            wet,
        })
    }
    fn into_any(self) -> AnyJob {
        AnyJob::ProcessLibraryVideo(self)
    }
}
