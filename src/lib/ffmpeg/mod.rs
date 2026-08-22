pub mod audio_settings;
pub mod mapping;
pub mod video_settings;

use crate::ffmpeg::{audio_settings::AudioSettings, mapping::Mapping, video_settings::VideoSettings};
use crate::hive::jobs::process_library_video::Operation;
use crate::{error, formats, info, log_fn_name, success};
use function_name::named;
use smol::process;
use std::{path::Path, process::ExitStatus};
use thiserror::Error;

#[derive(Debug)]
pub struct Progress {
    // TODO
}

#[derive(Debug, Error)]
pub enum FFmpegError {
    #[error("command error: {0}")]
    CommandError(ExitStatus),
}

#[named]
pub async fn spawn_ffmpeg(args: &[String], _on_progress: impl Fn(Progress) + Send + Sync + 'static) -> process::Output {
    log_fn_name!(auto);
    info!("running ffmpeg with arguments: {args:?}");
    let child = process::Command::new("ffmpeg").args(args).spawn().expect("todo"); //TODO: implement on_progress
    child.output().await.expect("todo")
}

pub async fn get_version() -> String {
    String::from("VERSION TODO")
}

#[named]
pub fn handle_ffmpeg_output(out: process::Output) -> Result<(), FFmpegError> {
    log_fn_name!(auto);
    if out.status.success() {
        success!(
            "ffmpeg process finished successfully (exit status: {}): stdout: {}, stderr: {}",
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        Ok(())
    } else {
        error!(
            "ffmpeg process finished with an error (exit status: {}): stdout: {}, stderr: {}",
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        Err(FFmpegError::CommandError(out.status))
    }
}

#[named]
pub async fn ffmpeg_cut_video_streamcopy(
    source_path: &Path,
    destination_path: &Path,
    start_time_sec: Option<f64>,
    end_time_sec: Option<f64>,
    on_progress: impl Fn(Progress) + Send + Sync + 'static,
) -> Result<(), FFmpegError> {
    log_fn_name!(auto);

    let input_filename = source_path.to_string_lossy().to_string();
    let output_filename = destination_path.to_string_lossy().to_string();

    let mut args = Vec::new();
    args.extend_from_slice(&formats!["-n", "-i", "{input_filename}"]);
    if let Some(start_time_sec) = start_time_sec {
        args.extend_from_slice(&formats!["-ss", "{start_time_sec}"]);
    }
    if let Some(end_time_sec) = end_time_sec {
        let duration = end_time_sec - start_time_sec.unwrap_or(0.0);
        args.extend_from_slice(&formats!["-t", "{duration}"]);
    }
    Mapping::AllFromSource.append_args(&mut args);
    VideoSettings::copy().append_args(&mut args);
    AudioSettings::copy().append_args(&mut args);
    args.push(output_filename);

    info!("determined ffmpeg args for cutting");
    let out = spawn_ffmpeg(&args, on_progress).await;
    handle_ffmpeg_output(out)
}

#[named]
pub async fn ffmpeg_process_video(
    source_path: &Path,
    destination_path: &Path,
    operation: Operation,
    on_progress: impl Fn(Progress) + Send + Sync + 'static,
) -> Result<(), FFmpegError> {
    log_fn_name!(auto);

    let input_filename = source_path.to_string_lossy().to_string();
    let output_filename = destination_path.to_string_lossy().to_string();
    let video_settings = operation.video_settings();
    let audio_settings = operation.audio_settings();
    let mapping = if operation.preserve_all_streams() {
        Mapping::AllFromSource
    } else {
        Mapping::MainVideoAudioOnly
    };

    let mut args = Vec::new();
    args.extend_from_slice(&formats!["-n", "-i", "{input_filename}"]);
    mapping.append_args(&mut args);
    video_settings.append_args(&mut args);
    audio_settings.append_args(&mut args);
    args.push(output_filename);

    info!("determined ffmpeg args for processing");
    let out = spawn_ffmpeg(&args, on_progress).await;
    handle_ffmpeg_output(out)
}
