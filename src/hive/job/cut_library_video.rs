use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data::library::database::{ClothInfo, LibraryDatabase};
use crate::data::library::index::LibraryIndex;
use crate::data::library::info::LibraryInfo;
use crate::data::library::{create_stpl_url_to_file, get_library_dir_of_path, path_within_library_dir, scan_register_added_file};
use crate::ffmpeg::ffmpeg_cut_video_streamcopy;
use crate::hive::job::{Failure, Job, Success};
use crate::hive::worker::Worker;
use crate::util::filelocked::{ClosedFileLocked, FileLockableData, FileLockableDataDefault};
use crate::util::timestamp::NsLocalTimestamp;
use std::path::Path;
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CutLibraryVideoJob {
    pub source_path: PathBuf,
    pub cut_start_point: Option<NsLocalTimestamp>,
    pub cut_end_point: Option<NsLocalTimestamp>,
    pub destination_path: PathBuf,
}

impl Job for CutLibraryVideoJob {
    async fn run(&self, worker: Arc<Worker>) -> Result<Success, Failure> {
        let worker_info = worker.data().lock().unwrap().info.clone();
        let config = worker.config();
        // Helper functions
        let _open_library_db_readwrite = || {
            LibraryDatabase::lock_and_read_or_default(config.library_database_path(), Some(&worker_info))
                .map_err(|e| Failure::LibraryError(e.to_string()))
        };
        let open_library_db_readonly = || {
            LibraryDatabase::read_without_locking_or_default(config.library_database_path())
                .map_err(|e| Failure::LibraryError(e.to_string()))
        };
        let open_library_info_readonly = |library_dir: &Path| {
            LibraryInfo::read_without_locking(library_dir.join(LibraryInfo::STANDARD_FILENAME))
                .map_err(|e| Failure::LibraryError(e.to_string()))
        };
        let _reopen_library_db =
            |library_db: ClosedFileLocked<LibraryDatabase>| library_db.reopen().map_err(|e| Failure::LibraryError(e.to_string()));

        // TODO: ^^These helper functions are copied from super::AnyJob.
        // They should be placed somewhere else, presumably in the worker itself. it would make a lot of sense that way, no?

        // Fetch the source proof UUID from the local index
        let source_library_dir = get_library_dir_of_path(&self.source_path).expect("todo");
        let source_library_index =
            LibraryIndex::read_without_locking(source_library_dir.join(LibraryIndex::STANDARD_FILENAME)).expect("todo");

        let internal_source_path = path_within_library_dir(&source_library_dir, &self.source_path).expect("todo");
        let source_proof_uuid = source_library_index.files.get(&internal_source_path).expect("todo");

        // Get source proof entry from the database
        let library_db = open_library_db_readonly()?;
        let proof_entry = library_db
            .find_entry_by_uuid(*source_proof_uuid)
            .ok_or(Failure::EntryNotFound(*source_proof_uuid))?
            .to_owned();
        drop(library_db);

        // Sanity check - proof entry with the given UUID should contain the given URL.
        // If it doesn't, that means the library needs to be rescanned or the request was invalid.
        let library_info = open_library_info_readonly(&source_library_dir)?;
        let file_url = create_stpl_url_to_file(library_info, &source_library_dir, &self.source_path).expect("todo");
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
            Some(&worker_info),
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
}
