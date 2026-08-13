use crate::{error, hive::jobs::process_library_video::ProcessingType, info, log_fn_name, success};
use rust_ffmpeg::{Codec, Duration, FFmpegBuilder, Input, Output, Progress};
use std::path::Path;

pub async fn ffmpeg_cut_video_streamcopy(
    source_path: &Path,
    destination_path: &Path,
    start_time_ms: Option<u64>,
    end_time_ms: Option<u64>,
    on_progress: impl Fn(Progress) + Send + Sync + 'static,
) -> Result<(), rust_ffmpeg::Error> {
    log_fn_name!("ffmpeg_cut_video_streamcopy");

    let input = Input::new(source_path.to_string_lossy().to_string());
    let input = if let Some(start_time_ms) = start_time_ms {
        input.seek(Duration::from_millis(start_time_ms))
    } else {
        input
    };
    let input = if let Some(end_time_ms) = end_time_ms {
        input.duration(Duration::from_millis(end_time_ms - start_time_ms.unwrap_or(0)))
    } else {
        input
    };
    let ffmpeg = FFmpegBuilder::new()?
        .input(input)
        .output(
            Output::new(destination_path.to_string_lossy().to_string())
                .audio_codec(Codec::copy())
                .video_codec(Codec::copy()),
        )
        .no_overwrite()
        .on_progress(on_progress);

    let args = ffmpeg.build_args();
    info!("running ffmpeg with arguments: {args:?}");

    match ffmpeg.run().await.unwrap().into_result() {
        Ok(out) => {
            success!(
                "ffmpeg process finished successfully (exit code {}): stdout: {:?}, stderr: {:?}",
                out.status,
                out.stdout_str(),
                out.stderr_str()
            );
            Ok(())
        }
        Err(e) => {
            error!("ffmpeg process finished with an error (non-zero exit code): {e:?}");
            Err(e)?
        }
    }
}

pub async fn ffmpeg_process_video(
    source_path: &Path,
    destination_path: &Path,
    processing_type: ProcessingType,
    on_progress: impl Fn(Progress) + Send + Sync + 'static,
) -> Result<(), rust_ffmpeg::Error> {
    log_fn_name!("ffmpeg_cut_video_streamcopy");

    let vcodec = processing_type.vcodec();
    let acodec = processing_type.acodec();

    let input = Input::new(source_path.to_string_lossy().to_string());
    let output = Output::new(destination_path.to_string_lossy().to_string())
        .video_codec_opts(vcodec)
        .audio_codec_opts(acodec);
    let ffmpeg = FFmpegBuilder::new()?
        .input(input)
        .output(output)
        .no_overwrite()
        .on_progress(on_progress);

    let args = ffmpeg.build_args();
    info!("running ffmpeg with arguments: {args:?}");

    match ffmpeg.run().await.unwrap().into_result() {
        Ok(out) => {
            success!(
                "ffmpeg process finished successfully (exit code {}): stdout: {:?}, stderr: {:?}",
                out.status,
                out.stdout_str(),
                out.stderr_str()
            );
            Ok(())
        }
        Err(e) => {
            error!("ffmpeg process finished with an error (non-zero exit code): {e:?}");
            Err(e)?
        }
    }
}
