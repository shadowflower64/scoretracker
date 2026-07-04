//! A library for "scoretracker", a complex system for storing rhythm gaming scores.
//!
//! # Components
//! Here is a list of 12 main components that "scoretracker" can be divided into:
//! | #  | Component name               | Description                                                                          | Progress |
//! | -- | ---------------------------- | ------------------------------------------------------------------------------------ | -------- |
//! | 1  | [**Hive**](hive)             | Handles various computation tasks and dispatches workers to execute those tasks.     |      20% |
//! | 2  | [**Library**](library)       | A database of rhythm game achievement proofs (mainly screenshots and videos).        |      10% |
//! | 3  | **YouTubeManager**           | Handles uploading proofs to YouTube and assigns the metadata using the YouTube API.  |     0.1% |
//! | 4  | [**Scoreboard**](scoreboard) | A database of rhythm game plays/scores/performances.                                 |       2% |
//! | 5  | [**SongDB**](songdb)         | A database of songs from rhythm games, along with their note counts and other stats. |     0.1% |
//! | 6  | [**scoretracker-toolkit**](../scoretracker_toolkit/index.html)     | Command-Line Interface for all scoretracker features.                                |      15% |
//! | 7  | **scoretracker-api**         | HTTP server with a JSON API interface for most or all scoretracker features.         |       0% |
//! | 8  | **scoretracker-web**         | Web frontend that connects to the scoretracker-api mentioned above.                  |     0.1% |
//! | 9  | **obs-frontend**             | A web-based frontend designed specifically for OBS, for displaying stats etc.        |       0% |
//! | 10 | **scoretracker-gui**         | Probably Qt-based GUI application with all scoretracker features. (JS is awful)      |       0% |
//! | 11 | **OCR**                      | Automatic proof reading.                                                             |     0.1% |
//! | 12 | **ReplayReader**             | Game-specific file parsing, mainly for reading replays.                              |       0% |
//!
//! This lib crate specifically will include components 1..=5 and 11..=12. Components 6..=10 will be separate, but built upon the `scoretracker` library.
//! `scoretracker-toolkit` is also present in this codebase in `src/cli/main.rs`.
//!
//! # Glossary
//! Here is some of the terminology used in "scoretracker":
//! - **Chart** - A set of notes that the player must play.
//! - **Library item** (name wip) - An entry in the [`library`].
//!   One *library item* represents a unique file in the library.
//!   Duplicate files (files with the same SHA256 hash) are considered the same library item.
//!   For that reason, a library item can have multiple locations/paths recorded in it.
//! - **Performance** - One play of one player on one chart of a song, with a given difficulty level, an instrument, and a score.
//! - **Proof** - A library item that proves a performance is real.
//! - **Score** - Numerical amount of points usually displayed at the end of a song.
//! - **Song** - A song in a rhythm game. One song can have multiple charts (for example, different difficulties, or different instruments).

pub mod config;
pub mod ffmpeg;
pub mod game;
pub mod games;
pub mod hive;
pub mod library;
pub mod scoreboard;
pub mod songdb;
pub mod tests;
pub mod util;

/// Current version of `scoretracker`, read from `CARGO_PKG_VERSION` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn example_add(left: u64, right: u64) -> u64 {
    left + right
}
