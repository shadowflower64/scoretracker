//! A collection of proof files.
//!
//! A "library" is a directory on a hard drive that contains videos and images
//! that are proof of a player's [performance](crate::scoreboard::performance) on a song.
//!
//! Apart from just video and image files, a library directory contains additional files:
//! * `library_info.json` ([`info`]) - contains basic information about the library, such as the domain name.
//! * `library_cache.json` ([`cache`]) - stores file hashes in relation to file names and file stat, so that the hash doesn't have to be recalculated all the time.
//! * `library_index.json` ([`index`]) - contains a mapping from file paths to proof UUIDs, for easily locating proof files.
//! * `library_aux.json` ([`aux_data`]) - contains additional auxiliary data about the library, such as local tags.
//!
//! There is also a file that is shared across all libraries:
//! * `library_database.json` ([`database`]) - permanently stores all of the data about all of the proofs globally.
//!   This includes: URLs to the proof, comments, manually assigned categories, related performance UUIDs, etc.
pub mod aux_data;
pub mod cache;
pub mod database;
pub mod index;
pub mod info;
