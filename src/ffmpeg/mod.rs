use std::path::Path;

use rust_ffmpeg::{Codec, Duration, FFmpegBuilder, Input, Output};

pub async fn ffmpeg_cut_video_streamcopy(
    source_path: &Path,
    destination_path: &Path,
    start_time_ms: Option<u64>,
    end_time_ms: Option<u64>,
) {
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
    FFmpegBuilder::new()
        .expect("todo")
        .input(input)
        .output(
            Output::new(destination_path.to_string_lossy().to_string())
                .audio_codec(Codec::copy())
                .video_codec(Codec::copy()),
        )
        .on_progress(|p| {
            println!("Progress: {:?}", p);
        })
        .run()
        .await
        .expect("todo2");
    todo!()
}
