//! Submodules of this module contain data structures for specific games.
//!
//! The following modules define data structures for the game, game songs, matches and performances.
//! These structures implement [`crate::data::game::Game`], [`crate::data::game::song::SongTrait`], [`crate::data::scoreboard::match::MatchTrait`], [`crate::data::scoreboard::performance::PerformanceTrait`].
//!
//! The structure for Game is always empty and contains some static functions can be defined per-game.
//! An instance of a Game structure can be created by using [`crate::data::game::game_instance_from_id`]

use std::sync::LazyLock;

use crate::data::game::AnyGame;
use linkme::distributed_slice;

pub mod adofai;
pub mod arcaea;
pub mod beatstar;
pub mod bh;
pub mod ch;
pub mod cytus;
pub mod cytus2;
pub mod deemo2;
pub mod djmax_respect_v;
pub mod fnfest;
pub mod gh3;
pub mod gh5;
pub mod gharcade;
pub mod ghsh;
pub mod ghvh;
pub mod ghwor;
pub mod ghwt;
pub mod in_falsus;
pub mod osu;
pub mod phigros;
pub mod pjd_megamix_plus;
pub mod rb3;
pub mod rb4;
pub mod rd;
pub mod rizline;
pub mod unbeatable;
pub mod vs;
pub mod yarg;

#[distributed_slice]
pub static GAMES: [fn() -> AnyGame];

pub fn registered_games() -> &'static Vec<AnyGame> {
    static ALL_GAMES: LazyLock<Vec<AnyGame>> = LazyLock::new(|| GAMES.iter().map(|game_factory| game_factory()).collect());
    &ALL_GAMES
}
