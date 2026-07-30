//! Submodules of this module contain data structures for specific games.
//!
//! The following modules define data structures for the game, game songs, matches and performances.
//! These structures implement [`crate::data::game::Game`], [`crate::data::game::song::SongTrait`], [`crate::data::scoreboard::match::MatchTrait`], [`crate::data::scoreboard::performance::PerformanceTrait`].
//!
//! The structure for Game is always empty and contains some static functions can be defined per-game.
//! An instance of a Game structure can be created by using [`crate::data::game::game_instance_from_id`]
pub mod adofai;
pub mod ch;
pub mod djmax_respect_v;
pub mod gh3;
pub mod in_falsus;
pub mod osu;
pub mod rd;
pub mod unbeatable;
pub mod yarg;
