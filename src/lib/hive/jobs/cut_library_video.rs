//! Losslessly cut a video from the library and save result to library
use crate::data::library::database::ClothInfo;
use crate::data::library::stpl_url::StplUrl;
use crate::ffmpeg::{ffmpeg_cut_video_streamcopy, get_version};
use crate::hive::job::{AnyJob, Fail, Job, Success};
use crate::hive::worker::Worker;
use crate::util::timestamp::NsLocalTimestamp;
use crate::util::uuid::UuidString;
use crate::{info, log_fn_name};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CutLibraryVideoJob {
    pub source: StplUrl,
    pub source_proof_uuid_precondition_check: Option<UuidString>,
    pub destination: StplUrl,

    pub cut_start_point: Option<NsLocalTimestamp>,
    pub cut_end_point: Option<NsLocalTimestamp>,
}

impl Job for CutLibraryVideoJob {
    async fn run(&self, worker: Arc<Worker>) -> Result<Success, Fail> {
        log_fn_name!("job:cut_library_video");

        let ffmpeg_version = get_version().await;
        info!("ffmpeg version: {:?}", ffmpeg_version);

        // Get source proof file
        let cloth = worker.find_or_download_proof_file(&self.source).await?;
        let cloth_info = ClothInfo {
            uuid: cloth.uuid(),
            start_point: self.cut_start_point,
            end_point: self.cut_end_point,
        };

        // Check if it matches the precondition (if it doesn't, that means that the input request was incorrect or has become invalid)
        if let Some(precondition_uuid) = self.source_proof_uuid_precondition_check
            && cloth.uuid() != precondition_uuid
        {
            return Err(Fail::PreconditionUuidDoesNotMatch {
                stpl_url: self.source.clone(),
                read_proof_uuid: cloth.uuid(),
                precondition_uuid,
            });
        }

        // Launch ffmpeg to cut the video losslessly
        let source_path = cloth.read_only_path();
        let destination_path = worker.create_temp_path_for(&self.destination).await;
        let start = self.cut_start_point.map(|x| x.as_secs_f64());
        let end = self.cut_end_point.map(|x| x.as_secs_f64());
        let worker2 = Arc::clone(&worker);
        ffmpeg_cut_video_streamcopy(&source_path, &destination_path, start, end, async move |progress| {
            worker2.update_task_progress_very_simple(format!("{progress:?}")).await;
        })
        .await?;

        // Register new file into library
        let fragment = worker
            .move_or_upload_proof_file(&destination_path, &self.destination, |entry| {
                entry.cloth = Some(cloth_info.clone());
                entry.cut = Some(true);
            })
            .await?;

        Ok(Success::CutVideo {
            cloth: cloth_info,
            fragment: fragment.uuid(),
        })
    }

    fn into_any(self) -> AnyJob {
        AnyJob::CutLibraryVideo(self)
    }
}
