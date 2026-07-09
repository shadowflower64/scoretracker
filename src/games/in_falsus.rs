use crate::game::Game;
use crate::scoreboard::r#match::{CommonMatchInfo, MatchTrait};
use crate::scoreboard::performance::{CommonPerformanceInfo, PerformanceTrait};
use serde::{Deserialize, Serialize};

/// Judgement count split between early/late, shown on mouse hover
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JudgementCountSplit {
    pub early: u32,
    pub late: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimpleBreakdown {
    pub critical_exact: u32,
    pub exact: u32,
    pub near: u32,
    #[serde(rename = "break")]
    pub break_: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FullJudgementCount {
    pub tap: u32,
    pub hold: u32,
    pub field: u32,
    pub flick: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FullBreakdown {
    pub critical_exact: FullJudgementCount,
    pub exact: FullJudgementCount,
    pub near: FullJudgementCount,
    #[serde(rename = "break")]
    pub break_: FullJudgementCount,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Loadout; // TODO

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerInfo {
    pub name: String,
    pub loadout: Loadout,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterResultType {
    Connected,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EncounterResultShort {
    pub main_score: u32,
    pub status: EncounterResultType,
    pub stars: u32,
    pub own_info: PlayerInfo,
    pub opponent_info: PlayerInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OffensiveStats {
    pub offensive_rate: u32,
    pub damage_dealt: f64,
    pub traits: f64,
    pub performance: f64,
    pub performance_max: f64,
    pub resolve_recovered_opponent: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefensiveStats {
    pub defensive_rate: u32,
    pub damage_received: f64,
    pub traits: f64,
    pub performance: f64,
    pub performance_max: f64,
    pub resolve_recovered_own: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PhaseStartStats {
    pub resolve: f64,
    pub power: u32,
    pub defense: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DamageDealtStats {
    pub traits: f64,
    pub performance: f64,
    pub performance_max: f64,
    pub critical_exact: f64,
    pub exact: f64,
    pub near: f64,
    #[serde(rename = "break")]
    pub break_: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResolveRecoveredStats {
    pub traits: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PhaseStats {
    pub phase_start_stats: PhaseStartStats,
    pub damage_dealt: DamageDealtStats,
    pub resolve_recovered: ResolveRecoveredStats,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Own,
    Opponent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Phase {
    pub own: PhaseStats,
    pub opponent: PhaseStats,
    pub highlighted: Side,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EncounterResultFull {
    pub offensive_stats: OffensiveStats,
    pub defensive_stats: DefensiveStats,
    pub phases: [Phase; 5],
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiveStatus {
    DiveFailed,
    DiveCleared,
    FullLink,
    PerfectDive,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Results {
    pub max_link: u32,
    pub score: u32,
    pub simple_breakdown: SimpleBreakdown,
    pub full_breakdown: Option<FullBreakdown>,
    pub exact_split: Option<JudgementCountSplit>,
    pub near_split: Option<JudgementCountSplit>,
    pub dive_status: DiveStatus,
    pub miss_hp: i32,
    pub encounter_result: EncounterResultShort,
    pub encounter_result_full: EncounterResultFull,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Performance {
    #[serde(flatten)]
    pub common: CommonPerformanceInfo,
    pub results: Results,
}

#[typetag::serde(name = "in_falsus")]
impl PerformanceTrait for Performance {
    fn common(&self) -> &CommonPerformanceInfo {
        &self.common
    }
    fn score(&self) -> f64 {
        self.results.score as f64
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Match {
    #[serde(flatten)]
    pub common: CommonMatchInfo,
}

#[typetag::serde(name = "in_falsus")]
impl MatchTrait for Match {
    fn common(&self) -> &CommonMatchInfo {
        &self.common
    }
    fn score(&self) -> f64 {
        unimplemented!()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InFalsus;

#[typetag::serde(name = "in_falsus")]
impl Game for InFalsus {
    fn pretty_name(&self) -> &'static str {
        "In Falsus"
    }
    fn url_shortname(&self) -> &'static str {
        "in_falsus"
    }
}
