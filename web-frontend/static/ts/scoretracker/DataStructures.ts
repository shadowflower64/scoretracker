type u32 = number;
type i32 = number;
type f64 = number;
type Option<T> = T | null;
type UuidString = string;
type Vec<T> = Array<T>;

/// This type is a `bigint` 99% of the time - it is only a `number` if it is very close to 1970-01-01 (within a few hours)
export type NsTimestamp = bigint | number;
export type BigintNsTimestamp = bigint;
export function msToNs(ms: number): BigintNsTimestamp {
    return BigInt(ms) * 1_000_000n;
}
export function nsToMs(ns: BigintNsTimestamp): number {
    return Number(ns / 1_000_000n);
}
export function nsTimestampToBigint(nsTimestamp: NsTimestamp): BigintNsTimestamp {
    return BigInt(nsTimestamp);
}
export function nsTimestampComponents(nsTimestamp: BigintNsTimestamp): [Date, number] {
    const ms = nsToMs(nsTimestamp);
    // const date = new Date(Math.floor(ms / 1_000) * 1_000);
    const date = new Date(ms);
    return [
        date,
        Number(nsTimestamp % 1_000_000n)
        // Number(nsTimestamp % 1_000_000_000n)
    ];
}

export type MetadataValue = string | number | boolean;
export type GenericMetadata = { [key: string]: MetadataValue; };
export type MatchMetadata = GenericMetadata;
export type PerformanceMetadata = GenericMetadata;

export type CommonPerformanceInfo = {
    /// UUID of the performance.
    uuid: UuidString,

    /// Player UUID.
    player_uuid: UuidString,

    /// List of library entry UUIDs that are proof of this performance.
    proof: Vec<UuidString>,

    /// Optional user comment.
    comment: Option<string>,

    /// Any additional performance metadata.
    metadata: PerformanceMetadata,
};

export type CommonMatchInfo = {
    /// UUID of the match.
    uuid: UuidString,

    /// Timestamp of the match - specifically, the timestamp of the first frame of the end screen. Can be approximate.
    timestamp: NsTimestamp,

    /// Named ID of the song.
    song_id: string,

    /// Performances belonging to this match.
    performance_ids: Vec<UuidString>,

    /// List of library entry UUIDs that are proof of this match.
    proof: Vec<UuidString>,

    /// Optional user comment.
    comment: Option<string>,

    /// Any additional match metadata.
    metadata: MatchMetadata,
};

export namespace InFalsus {
    export type JudgementCountSplit = {
        early: u32,
        late: u32,
    };

    export type SimpleBreakdown = {
        critical_exact: u32,
        exact: u32,
        near: u32,
        break: u32,
    };

    export type FullJudgementCount = {
        tap: u32,
        hold: u32,
        field: u32,
        flick: u32,
    };

    export type FullBreakdown = {
        critical_exact: FullJudgementCount,
        exact: FullJudgementCount,
        near: FullJudgementCount,
        break: FullJudgementCount,
    };

    export type Loadout = {};

    export type PlayerInfo = {
        name: String,
        loadout: Loadout,
    };

    export type EncounterResultType = "connected";

    export type EncounterResultShort = {
        main_score: u32,
        status: EncounterResultType,
        stars: u32,
        own_info: PlayerInfo,
        opponent_info: PlayerInfo,
    };

    export type OffensiveStats = {
        offensive_rate: u32,
        damage_dealt: f64,
        traits: f64,
        performance: f64,
        performance_max: f64,
        resolve_recovered_opponent: f64,
    };


    export type DefensiveStats = {
        defensive_rate: u32,
        damage_received: f64,
        traits: f64,
        performance: f64,
        performance_max: f64,
        resolve_recovered_own: f64,
    };


    export type PhaseStartStats = {
        resolve: f64,
        power: u32,
        defense: u32,
    };

    export type DamageDealtStats = {
        traits: f64,
        performance: f64,
        performance_max: f64,
        critical_exact: f64,
        exact: f64,
        near: f64,
        break: f64,
    };

    export type ResolveRecoveredStats = {
        traits: f64,
    };

    export type PhaseStats = {
        phase_start_stats: PhaseStartStats,
        damage_dealt: DamageDealtStats,
        resolve_recovered: ResolveRecoveredStats,
    };

    export type Side = "own" | "opponent";

    export type Phase = {
        own: PhaseStats,
        opponent: PhaseStats,
        highlighted: Side,
    };

    export type EncounterResultFull = {
        offensive_stats: OffensiveStats,
        defensive_stats: DefensiveStats,
        phases: [Phase, Phase, Phase, Phase, Phase],
    };

    export type DiveStatus = "dive_failed" | "dive_cleared" | "full_link" | "perfect_dive";

    export type Results = {
        max_link: u32,
        score: u32,
        simple_breakdown: SimpleBreakdown,
        full_breakdown: Option<FullBreakdown>,
        exact_split: Option<JudgementCountSplit>,
        near_split: Option<JudgementCountSplit>,
        dive_status: DiveStatus,
        miss_hp: i32,
        encounter_result: EncounterResultShort,
        encounter_result_full: EncounterResultFull,
    };

    export type Performance = CommonPerformanceInfo & {
        results: Results;
    };

    export type Match = CommonMatchInfo;
}
