/// This structure holds information about music.
///
/// This struct contains data such as the song title, song artist, etc.
/// Keep in mind that "song" means the music itself, and not a song within the context of a rhythm game (those are called [`Chartset`]s).
/// Therefore, songs do not have information about charts.
pub struct Song {
    /// Named ID of the song, most often follows the convention `lowercase_artist-lowercase_title[-lowercase_song_version]`.
    ///
    /// In case of ID collisions, a "version" can be added at the end.
    /// Keep in mind that this convention is not guaranteed and this can really be any arbitrary string.
    pub song_id: String,

    /// Song title.
    ///
    /// The title of the song. This may be different to how it appears in rhythm games.
    ///
    /// TODO: should this include information such as (blank Cover) or (blank Remix)? or should that be separate
    pub title: String,

    /// Song artist.
    ///
    /// Artist string - who made the song.
    ///
    /// TODO: maybe we should have this be an array with artist IDs?
    pub artist: String,

    /// Release year.
    pub year: Option<u32>,
}
