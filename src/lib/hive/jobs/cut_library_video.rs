//! Losslessly cut a video from the library and save result to library
use crate::data::library::database::ClothInfo;
use crate::data::library::{create_stpl_url_to_relfile, get_library_dir_of_path, path_within_library_dir, scan_register_added_file};
use crate::ffmpeg::ffmpeg_cut_video_streamcopy;
use crate::hive::job::{AnyJob, Fail, Job, Success};
use crate::hive::worker::Worker;
use crate::util::timestamp::NsLocalTimestamp;
use crate::util::uuid::UuidString;
use crate::{info, log_fn_name};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CutLibraryVideoJob {
    pub source_path: PathBuf,
    pub source_proof_uuid_precondition_check: Option<UuidString>,
    pub cut_start_point: Option<NsLocalTimestamp>,
    pub cut_end_point: Option<NsLocalTimestamp>,
    pub destination_path: PathBuf,
}

impl Job for CutLibraryVideoJob {
    async fn run(&self, worker: Arc<Worker>) -> Result<Success, Fail> {
        log_fn_name!("job:cut_library_video");

        let ffmpeg_version = rust_ffmpeg::version().await;
        info!("ffmpeg version: {:?}", ffmpeg_version);

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
            .find_entry_by_uuid(source_proof_uuid.0)
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
        ffmpeg_cut_video_streamcopy(
            &self.source_path,
            &self.destination_path,
            self.cut_start_point.map(|x| x.as_millis() as u64),
            self.cut_end_point.map(|x| x.as_millis() as u64),
            move |progress| {
                worker2.update_task_progress_very_simple(format!("{progress:?}"));
            },
        )
        .await?;

        // Get library info of the destination file
        let fragment = if let Some(destination_library_dir) = get_library_dir_of_path(&self.source_path) {
            // Register new file to index and to database
            let (_rel_path, uuid) = scan_register_added_file(
                &destination_library_dir,
                &config.library_database_path(),
                &self.destination_path,
                |entry| {
                    entry.cloth = Some(ClothInfo {
                        uuid: *source_proof_uuid,
                        start_point: self.cut_start_point,
                        end_point: self.cut_end_point,
                    })
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

        Ok(Success::CutVideo {
            cloth: ClothInfo {
                uuid: *source_proof_uuid,
                start_point: self.cut_start_point,
                end_point: self.cut_end_point,
            },
            fragment,
        })
    }
    fn into_any(self) -> AnyJob {
        AnyJob::CutLibraryVideo(self)
    }
}
