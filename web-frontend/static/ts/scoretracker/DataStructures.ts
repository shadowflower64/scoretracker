type UuidString = string;

export interface Nanoseconds {
    seconds: number;
    frac: number;
}
export type NsTimestamp = Nanoseconds;
export function nanos(nanoseconds: Nanoseconds): bigint {
    return (BigInt(nanoseconds.seconds) * 1_000_000_000n) + BigInt(nanoseconds.frac);
}
export function millisFloor(nanoseconds: Nanoseconds): number {
    return Number(nanos(nanoseconds) / 1_000_000n);
}
export function fromMillis(ms: number): Nanoseconds {
    return {
        seconds: Math.floor(ms / 1000),
        frac: (ms % 1000) * 1_000_000
    };
}

export type MetadataValue = string | number | boolean;
export type GenericMetadata = { [key: string]: MetadataValue; };
export type MatchMetadata = { [key: string]: MetadataValue; };
export type PerformanceMetadata = { [key: string]: MetadataValue; };

export type CommonPerformanceInfo = {
    /// UUID of the performance.
    uuid: UuidString,

    /// Player UUID.
    player_uuid: UuidString,

    /// List of library entry UUIDs that are proof of this performance.
    proof: UuidString[],

    /// Optional user comment.
    comment?: string | null,

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
    performance_ids: UuidString[],

    /// List of library entry UUIDs that are proof of this match.
    proof: UuidString[],

    /// Optional user comment.
    comment?: string | null,

    /// Any additional match metadata.
    metadata: MatchMetadata,
};
