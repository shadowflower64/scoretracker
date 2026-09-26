use dyn_clone::{DynClone, clone_trait_object};
use serde::{Deserialize, Serialize};
use std::{any::Any, fmt::Debug};

/// This structure represents a single chart (one difficulty, one instrument).
///
/// The struct holds information about a note chart, a playable set of notes.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chart {
    /// Game ID.
    pub game: String,

    /// ID of the chartset that this chart belongs to.
    pub chartset_id: String,

    /// Snake-case name of the instrument/play mode of the chart.
    ///
    /// Examples:
    /// * for osu! this is either `standard`, `taiko`, `mania`, or `catch`.
    /// * for Guitar Hero this is either `guitar`, `bass`, `drums`, or `vocals`.
    /// * for DJMAX RESPECT V this is `4k`, `5k`, `6k`, `8k`, `4b`, `5b`, `6b`, or `8b`.
    /// * for Duolingo Music this is always `piano`.
    ///
    /// etc.
    pub instrument: String,

    /// Snake-case name of the difficulty of the played chart. (`easy`, `hard`, `insane`, etc...)
    pub difficulty: String,

    /// Name of the chart group.
    ///
    /// Identical charts that are in different chartsets or different games should count as the same FC.
    /// This field defines that this chart is already somewhere else.
    ///
    /// For example the song "Electric Rock" by "Sworn" appears in Guitar Hero: World Tour, Guitar Hero 5, Band Hero and Guitar Hero: Warriors of Rock,
    /// because it is a DLC song compatible with all of these games.
    /// The most "real" way to play this song is on GHWT - newer games have not yet been released at the time of this DLC release.
    /// Since there are hit engine differences between GHWT and GH5 these FCs are not directly comparable.
    /// However, the chart is still identical.
    /// If you FC it in GHWT it should definitely count.
    ///
    /// In this example, the guitar expert chart for GH5's "Electro Rock"
    /// should have the `chart_group` set to "ghwt/sworn-electro_rock/guitar/expert".
    ///
    /// If not present, the implicit value is: `game/chartset_id/instrument/difficulty`.
    pub chart_group: Option<String>,

    /// Game-specific details about the chart (total note count etc.)
    pub details: Box<AnyChartDetails>,
}

impl Chart {
    pub fn game_specific_details<T: ChartDetails + 'static>(&self) -> Option<&T> {
        self.details.downcast_ref()
    }

    pub fn as_game_chart<'a, T: ChartDetails + 'static>(&'a self) -> Option<GameChart<'a, T>> {
        let game_specific_details = self.game_specific_details();
        game_specific_details.map(|x| GameChart {
            game: &self.game,
            chartset_id: &self.chartset_id,
            instrument: &self.instrument,
            difficulty: &self.difficulty,
            chart_group: self.chart_group.as_ref().map(|x| x.as_str()),
            details: x,
        })
    }
}

// TODO: Testing
// if this is ever necessary, it is possible to do... but i don't know if that's the best idea??
// this is like "a view" into the chart that guarantees that the details field is of a given type
// this can also be made for performance/match/chartset details
//
// but also you can just use fn `game_specific_details` and deal with the typed details struct by itself,
// without carrying around references to every other field. and if you need other fields, you can just get a ref
// to the original chart struct, instead of getting this GameChart borrow thing. i guess that only works if you have
// shared refs, and not exclusive/mutable references but idk if having mutable references here would work anyway.
#[derive(Debug, Clone)]
pub struct GameChart<'a, Details: ChartDetails + 'static> {
    pub game: &'a str,
    pub chartset_id: &'a str,
    pub instrument: &'a str,
    pub difficulty: &'a str,
    pub chart_group: Option<&'a str>,
    pub details: &'a Details,
}

#[typetag::serde(tag = "game")]
pub trait ChartDetails: Debug + DynClone {
    fn game_id(&self) -> &'static str {
        self.typetag_name()
    }
    fn any_ref(&self) -> &dyn Any;
}

impl dyn ChartDetails {
    fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.any_ref().downcast_ref()
    }
}

clone_trait_object! {ChartDetails}
pub type AnyChartDetails = dyn ChartDetails + 'static;
