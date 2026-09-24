/// This structure holds information about a music album.
pub struct Album {
    /// Named ID of the album, most often follows the convention `lowercase_artist-lowercase_title[-lowercase_song_version]`.
    pub album_id: String,

    /// Album title.
    pub title: String,

    /// Album artist.
    pub artist: String,

    /// Release year.
    pub year: Option<u32>,

    /// Song IDs.
    pub songs: Vec<String>,
}
