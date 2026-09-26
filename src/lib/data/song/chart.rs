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
    pub fn downcast_details<T: ChartDetails + 'static>(&self) -> Option<&T> {
        self.details.downcast_ref()
    }
}

#[cfg(test)]
mod test {
    use crate::data::{
        games::placeholder::PlaceholderChartDetails,
        song::chart::{Chart, ChartDetails},
    };
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::any::Any;

    #[test]
    pub fn testing_downcast() {
        // Some struct for testing
        #[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
        pub struct PlaceholderChartDetails2 {
            pub note_count: Option<u32>,
        }

        #[typetag::serde(name = "placeholder2")]
        impl ChartDetails for PlaceholderChartDetails2 {
            fn any_ref(&self) -> &dyn Any {
                self
            }
        }

        // Identical structure, different type
        #[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
        pub struct PlaceholderChartDetails3 {
            pub note_count: Option<u32>,
        }

        #[typetag::serde(name = "placeholder3")]
        impl ChartDetails for PlaceholderChartDetails3 {
            fn any_ref(&self) -> &dyn Any {
                self
            }
        }

        let cases = vec![
            json!({"game": "placeholder", "chartset_id": "xi-freedom_dive", "instrument": "piano", "difficulty": "expert", "chart_group": null, "details": {"game": "placeholder"}}),
            json!({"game": "placeholder", "chartset_id": "xi-freedom_dive", "instrument": "piano", "difficulty": "expert", "chart_group": null, "details": {"game": "placeholder2", "note_count": 1337}}),
            json!({"game": "placeholder", "chartset_id": "xi-freedom_dive", "instrument": "piano", "difficulty": "expert", "chart_group": null, "details": {"game": "placeholder3", "note_count": 420}}),
        ];
        for json_data in cases {
            println!("json data: {json_data}");
            let chart: Chart = serde_json::from_value(json_data).unwrap();
            println!("chart: {chart:?}");
            let specific = chart.downcast_details::<PlaceholderChartDetails>();
            println!("specific: {specific:?}");
            let specific2 = chart.downcast_details::<PlaceholderChartDetails2>();
            println!("specific2: {specific2:?}");
            let specific3 = chart.downcast_details::<PlaceholderChartDetails3>();
            println!("specific3: {specific3:?}");
        }
    }
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
