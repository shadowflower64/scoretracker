//! A library for "scoretracker", a complex system for storing rhythm gaming scores.
//!
//! # Components
//! Here is a list of 12 main components that "scoretracker" can be divided into:
//! | #  | Component name                      | Description                                                                                      | Progress |
//! | -- | ----------------------------------- | ------------------------------------------------------------------------------------------------ | -------- |
//! | 1  | [**scoretracker-toolkit**](../scoretracker_toolkit/index.html) | Command-Line Interface for all scoretracker features.                 |      50% |
//! | 2  | **scoretracker-gui**                | Probably Qt-based GUI application with all scoretracker features.                                |       0% |
//! | 3  | [**Web API**](web)                  | HTTP server with a JSON API interface for most or all scoretracker features.                     |       1% |
//! | 4  | **Web Frontend**                    | Web frontend that connects to the scoretracker API mentioned above.                              |       2% |
//! | 5  | [**Library**](data::library)        | A database of rhythm game achievement proofs (mainly screenshots and videos).                    |      20% |
//! | 6  | [**SongDB**](data::game::song)      | A database of rhythm game songs, along with their chart info, like note counts and other stats.  |     0.5% |
//! | 7  | [**Scoreboard**](data::scoreboard)  | A database of rhythm game plays/scores/performances.                                             |       2% |
//! | 8  | [**Hive**](hive)                    | System that handles various computation tasks and spawns workers to execute those tasks.         |      60% |
//! | 9  | **YouTube Manager**                 | System that handles uploading proofs to YouTube and assigns the metadata using the YouTube API.  |     0.1% |
//! | 10 | **OBS Frontend**                    | A web-based frontend designed specifically for OBS, for displaying stats etc.                    |       0% |
//! | 11 | **OCR**                             | Automatic proof reading with Optical Character Recognition.                                      |     0.1% |
//! | 12 | **Replay Reader**                   | Game-specific file parsing, mainly for reading replays.                                          |       0% |
//!
//! * This lib crate contains components: 3, 5, 6, 7, 8, 9, 11, 12.
//! * Component 1 (`scoretracker-toolkit`) is also present in this repository in `src/cli/main.rs`.
//! * Component 4 (web frontend) is also present in this repository in `web-frontend`.
//! * Components 2 and 10 are not yet decided...
//!
//! # Glossary
//! Here is some of the terminology used in "scoretracker":
//! - **Chart** - A complete set of notes that the player must play to finish a song.
//! - **Hive** - A centralized system that manages multiple workers and gives out tasks requested by the user to them.
//! - **Library item** (name wip) - An entry in the [`data::library`].
//!   One *library item* represents a unique file in the library, with respect to ownership
//!   (so, if multiple users upload the same exact file, it will still be classified as a separate library item).
//!   Duplicate files (files with the same SHA256 hash owned by the same user) are considered the same library item.
//!   For that reason, a library item can have multiple locations/paths recorded in it.
//! - **Match** - A grouping of several performances that happened at the same exact time.
//!   Most useful for multiplayer matches (both versus and co-op).
//!   Every performance has a match record associated with it; multiple performances can reference one match.
//!   The match data contains information that is always associated with all of the performances at once, such as
//!   the timestamp, the song ID, the game version, and other such data.
//!   Even if a game is completely singleplayer, a match record is created for consistency across the entire scoretracker system.
//! - **Performance** - One play of one player on one chart of a song, with a given difficulty level, an instrument, and a score.
//!   Every performance stores a UUID to a *match* record, which contains some additional information about the play.
//! - **Proof** - A library item that proves a performance is real (and was achieved legitimately).
//! - **Score** - Numerical amount of points usually displayed at the end of a song.
//! - **Song** - A song in a rhythm game. One song can have multiple charts (for example, different difficulties, or different instruments).
//! - **Worker** - A process that contributes computational resources by taking on various tasks from a central system called the "*Hive*".

pub mod config;
pub mod data;
pub mod ffmpeg;
pub mod hive;
pub mod spreadsheet;
pub mod tests;
pub mod util;
pub mod web;

/// Current version of `scoretracker`, read from `CARGO_PKG_VERSION` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
