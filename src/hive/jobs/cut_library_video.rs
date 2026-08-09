use crate::data::library::database::ClothInfo;
use crate::data::library::index::LibraryIndex;
use crate::data::library::{create_stpl_url_to_file, get_library_dir_of_path, path_within_library_dir, scan_register_added_file};
use crate::ffmpeg::ffmpeg_cut_video_streamcopy;
use crate::hive::job::{AnyJob, Failure, Job, Success};
use crate::hive::worker::Worker;
use crate::util::filelocked::FileLockableData;
use crate::util::timestamp::NsLocalTimestamp;
use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CutLibraryVideoJob {
    pub source_path: PathBuf,
    pub source_proof_uuid_precondition_check: Option<UuidString>,
    pub cut_start_point: Option<NsLocalTimestamp>,
    pub cut_end_point: Option<NsLocalTimestamp>,
    pub destination_path: PathBuf,
}

impl Job for CutLibraryVideoJob {
    async fn run(&self, worker: Arc<Worker>) -> Result<Success, Failure> {
        let worker_info = Some(&worker.info_cloned());
        let config = worker.config();
        // Helper functions

        // Fetch the source proof UUID from the local index
        let source_library_dir = get_library_dir_of_path(&self.source_path).expect("todo");
        let source_library_index =
            LibraryIndex::read_without_locking(source_library_dir.join(LibraryIndex::STANDARD_FILENAME)).expect("todo");

        let internal_source_path = path_within_library_dir(&source_library_dir, &self.source_path).expect("todo");
        let source_proof_uuid = source_library_index.files.get(&internal_source_path).expect("todo");

        // Check if it matches the precondition (if it doesn't, that means that the input request was incorrect!)
        if let Some(precondition_uuid) = self.source_proof_uuid_precondition_check
            && *source_proof_uuid != precondition_uuid
        {
            panic!("todo: precondition check failed")
        }

        // Get source proof entry from the database
        let library_db = worker.read_library_db()?;
        let proof_entry = library_db
            .find_entry_by_uuid(*source_proof_uuid)
            .ok_or(Failure::EntryNotFound(*source_proof_uuid))?
            .to_owned();
        drop(library_db);

        // Sanity check - proof entry with the given UUID should contain the given URL.
        // If it doesn't, that means the library needs to be rescanned or the request was invalid.
        let source_library_info = worker.read_library_info(&source_library_dir)?;
        let file_url = create_stpl_url_to_file(source_library_info, &source_library_dir, &self.source_path).expect("todo");
        if !proof_entry.library_urls.contains(&file_url) {
            return Err(Failure::FileUrlNotFoundInEntry {
                expected_url: file_url,
                entry: Box::new(proof_entry.to_owned()),
            });
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
        .await;

        // Get library info of the destination file
        let destination_library_dir = get_library_dir_of_path(&self.source_path).unwrap(); // TODO: error handling

        // Register new file to index and to database
        scan_register_added_file(
            &destination_library_dir,
            &config.library_database_path(),
            &self.destination_path,
            worker_info,
        )
        .map_err(|e| Failure::CannotRegisterFileIntoLibrary {
            file_path: self.destination_path.to_owned(),
            reason: e.to_string(),
        })?;

        Ok(Success::CutVideo {
            cloth: ClothInfo {
                uuid: *source_proof_uuid,
                start_point: self.cut_start_point,
                end_point: self.cut_end_point,
            },
            fragment: Uuid::now_v7().into(),
        })
    }
    fn into_any(self) -> AnyJob {
        AnyJob::CutLibraryVideo(self)
    }
}
