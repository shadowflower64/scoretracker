use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};

/// The quality state of the proof file.
///
/// Videos that are "raw" can be transcoded and lossily compressed to save space.
///
/// The enum is ordered from best quality (least destructive) to worst quality (most destructive).
///
/// Naming of quality states and actions that change the quality state is based on the analogy of storing physical paper documents (well... at least I tried):
/// * The first state of a video is [`QualityState::Raw`] - this is a video file that has been taken straight from the recording software, without any additional processing.
/// * You can "preserve" a video to keep it in its [`QualityState::Raw`] state.
/// * You can "fold" a video and it will become a [`QualityState::Folded`] video.
///   A folded video is pretty much visually lossless, and it takes up a lot less space, just like a folded sheet of paper.
/// * You can "mess up" the video and it will become a [`QualityState::Messy`] video.
///   A messy video is compressed, but it is still not a pain to watch. It may have a lower resolution and reduced bitrate so it might not be great for editing, or for pausing or zooming in on, but it should still be fine to watch on it's own.
/// * You can "crumple" the video and it will become a [`QualityState::Crumpled`] video.
///   A crumpled video is visibly lossily compressed, but takes up a whole lot less space
/// * You can "shred" the video and it will become a [`QualityState::Shredded`] video.
///   A shredded video is compressed to a terrible quality, but it will take up a very small amount of space, usually under 5 MiB.
///
/// Additionally, you can also:
/// * "trash" the video - which means it won't be processed, and will be moved straight to the system trash, and
/// * "delete" the video - which means it will be `rm`'d from the filesystem entirely, without even going to trash.
///
/// These actions are traditionally applied to the "raw" video only, but in practice more destructive actions can also be used on already messy, folded or crumpled videos.
///
/// Please note that videos originally recorded in low quality (like 360p) will still be classified as "raw",
/// so this value does not correspond directly to the "perceptual quality" of the proof file.
///
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, FromSql, ToSql)]
#[serde(rename_all = "snake_case")]
#[postgres(rename_all = "snake_case")]
pub enum QualityState {
    /// Default value - value not selected by user yet.
    #[default]
    Unspecified,

    /// Raw unprocessed recording or replay file or stream vod, which may or may not have been cut using `ffmpeg` with with `-c copy`, or LosslessCut. Largest file and best quality.
    /// Not recommended to store for a long time.
    Raw,

    /// Transcoded cut video, but visually lossless. Takes up a lot less space because it is transcoded after the initial recording on a slower encoding preset.
    /// Useful for PBs and first FCs.
    ///
    /// Example ffmpeg settings:
    /// - Codec: libx265
    /// - Resolution: 1080p
    /// - Framerate: 60 fps
    /// - Quality: crf=26
    /// - Preset: slow          // Note: `slower` is way too slow for commonly processing 1080p videos, taking about 1 hour per minute of video. it can however be useful at lower resolutions.
    ///
    /// Results: (input video: 1080p60, duration: 00:03:02.684)
    /// - Time: 00:33:48 (5.40 fps, 0,090x speed) -- 00:11:06 per 1 minute of video
    /// - File size: 116.03 MiB (5332.4kbits/s)   -- 38.11 MiB per 1 minute of video
    ///
    /// See also: [`Operation::CompressFoldVideo`](crate::hive::jobs::process_library_video::Operation::CompressFoldVideo)
    Folded,

    /// Transcoded cut video, lossy but still looking good and definitely watchable. Possibly in worse resolution.
    /// Should take up less than 20 MiB per minute of video.
    ///
    /// Example ffmpeg settings:
    /// - Codec: libx265
    /// - Resolution: 720p
    /// - Framerate: 60 fps
    /// - Quality: crf=29
    /// - Preset: slower
    ///
    /// Results: (input video: 1080p60, duration: 00:03:02.684)
    /// - Time: 01:08:37 (2.66 fps, 0,044x speed) -- 00:22:32 per 1 minute of video
    /// - File size: 44.53 MiB (2048.8kbits/s)    -- 14.63 MiB per 1 minute of video
    ///
    /// See also: [`Operation::CompressMessUpVideo`](crate::hive::jobs::process_library_video::Operation::CompressMessUpVideo)
    Messy,

    /// Transcoded cut video, in worse resolution but still readable quality.
    /// Has to take up less than 4 MiB per minute of video.
    /// Useful for non-PB performances that would've usually been thrown in the trash entirely.
    ///
    /// Example ffmpeg settings:
    /// - Codec: libx265
    /// - Resolution: 480p
    /// - Framerate: 60 fps
    /// - Quality: crf=35
    /// - Preset: slow          // Note: we don't really care about quality here anymore, and the filesize does not decrease much further if `slower` is used instead. this is good enough.
    ///
    /// Results: (input video: 1080p60, duration: 00:03:02.684)
    /// - Time: 00:06:21 (28.74 fps, 0.479x speed) -- 00:02:05 per 1 minute of video
    /// - File size: 12.35 MiB (3758.1kbits/s)     -- 4.06 MiB per 1 minute of video
    ///
    /// See also: [`Operation::CompressCrumpleVideo`](crate::hive::jobs::process_library_video::Operation::CompressCrumpleVideo)
    Crumpled,

    /// Transcoded cut video, with terrible bitrate and 360p.
    /// Takes up around 2 MiB per minute of video.
    /// Useful for unfinished performances or otherwise something that should be deleted usually, but may come in handy later (for example, for counting attempts).
    ///
    /// Example ffmpeg settings:
    /// - Codec: libx265
    /// - Resolution: 360p
    /// - Framerate: 60 fps
    /// - Quality: crf=38
    /// - Preset: slow          // Note: due to a large amount of files in this case, apart from the file size, we care more about transcoding speed than the actual quality of the video. this is good enough.
    ///
    /// Results: (input video: 1080p60, duration: 00:03:02.684)
    /// - Time: 00:03:30 (52.12 fps, 0.869x speed) -- 00:01:09 per 1 minute of video
    /// - File size: 5.17 MiB (240.6kbits/s)       -- 1.70 MiB per 1 minute of video
    ///
    /// See also: [`Operation::CompressShredVideo`](crate::hive::jobs::process_library_video::Operation::CompressShredVideo)
    Shredded,
}
