use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};

/// Category of the media that this entry describes - is it a screenshot, a video from a camera, a mobile screen recording, something else?
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, FromSql, ToSql)]
#[serde(rename_all = "snake_case")]
#[postgres(rename_all = "snake_case")]
pub enum MediaCategory {
    /// Default value - value not selected by user yet.
    #[default]
    #[serde(alias = "unset")] // TODO: temp alias for migration while testing, can be removed later
    Unspecified,

    /// An image of the screen captured from a PC.
    PcScreenshot,

    /// An image of the screen captured from a phone.
    MobileScreenshot,

    /// An image captured by a photo camera, a phone camera, or a webcam.
    CameraPhoto,

    /// A video of the screen captured by OBS Studio.
    ObsRecording,

    /// A video of the screen captured by OBS Studio, and then cut using the `autocut` script.
    ObsRecordingAutocut,

    /// A video of the screen captured by OBS Studio, and then cut using LosslessCut.
    ObsRecordingLosslessCut,

    /// A video of the screen captured by OBS Studio using the "Replay Buffer" feature.
    ObsReplay,

    /// A video of the screen captured by OBS Studio using the "Replay Buffer" feature, and then cut using the `autocut` script.
    ObsReplayAutocut,

    /// A video of the screen captured by OBS Studio using the "Replay Buffer" feature, and then cut using LosslessCut.
    ObsReplayLosslessCut,

    /// A video of the screen captured by a phone's screen recording software.
    MobileScreenRecording,

    /// A video of the screen captured by a phone's screen recording software, and then cut using LosslessCut.
    MobileScreenRecordingLosslessCut,

    /// A video captured by a photo camera, a phone camera, or a webcam.
    CameraVideo,

    /// Other media, that doesn't belong to any other category.
    Other,
}
