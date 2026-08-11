/**
 * Common types used in the Rust codebase.
 */

export type UuidString = string;

export interface Nanoseconds {
    /**
     * Integer
     */
    seconds: number;

    /**
     * Integer from 0 to 1_000_000_000
     */
    frac: number;
}
export namespace Nanoseconds {
    export function normalize(nanoseconds: Nanoseconds): Nanoseconds {
        const secondsWhole = Math.floor(nanoseconds.seconds);
        const secondsFrac = nanoseconds.seconds - secondsWhole;
        const totalNs = (BigInt(secondsWhole) * 1_000_000_000n) + BigInt(Math.floor(secondsFrac * 1_000_000_000)) + BigInt(Math.floor(nanoseconds.frac));
        return fromNanos(totalNs);
    }

    /**
     * Get a bigint containing the total amount of nanoseconds.
     * This is not the default representation because it is hard to (de)serialize.
     */
    export function nanos(nanoseconds: Nanoseconds): bigint {
        return (BigInt(nanoseconds.seconds) * 1_000_000_000n) + BigInt(nanoseconds.frac);
    }

    /**
     * Get a whole number of milliseconds.
     */
    export function millisWhole(nanoseconds: Nanoseconds): number {
        return Number(nanos(nanoseconds) / 1_000_000n);
    }

    /**
     * Get a remainder number of nanoseconds when diving into whole milliseconds.
     */
    export function millisFrac(nanoseconds: Nanoseconds): number {
        return nanoseconds.frac % 1_000_000;
    }

    /** 
     * Split up a Nanoseconds object into a number of whole milliseconds and a remainder number of nanoseconds.
     */
    export function millisParts(nanoseconds: Nanoseconds): [number, number] {
        return [millisWhole(nanoseconds), millisFrac(nanoseconds)];
    }

    /**
     * Split up a Nanoseconds object into a Date (with millisecond precision) and a remainder number of nanoseconds.
     */
    export function dateParts(nanoseconds: Nanoseconds): [Date, number] {
        return [new Date(millisWhole(nanoseconds)), millisFrac(nanoseconds)];
    }

    /**
     * Create a Nanoseconds object from a bigint amount of nanoseconds.
     */
    export function fromNanos(ns: bigint): Nanoseconds {
        const whole = ns / 1_000_000_000n;
        const frac = ns % 1_000_000_000n;
        return {
            seconds: Number(whole),
            frac: Number(frac)
        };
    }

    /**
     * Create a Nanoseconds object from a float amount of milliseconds.
     */
    export function fromMillis(ms: number): Nanoseconds {
        return fromMillisParts(ms, 0);
    }
    export function fromMillisParts(ms: number, nanos_frac: number): Nanoseconds {
        return normalize({
            seconds: ms / 1000,
            frac: nanos_frac
        });
    }
    export function fromDateParts(date: Date, nanos_frac: number): Nanoseconds {
        return fromMillisParts(date.getTime(), nanos_frac);
    }
}
export type NsTimestamp = Nanoseconds;
export type NsDuration = Nanoseconds;

export type MetadataValue = string | number | boolean;
export type GenericMetadata = { [key: string]: MetadataValue; };
export type MatchMetadata = { [key: string]: MetadataValue; };
export type PerformanceMetadata = { [key: string]: MetadataValue; };

export type CommonPerformanceInfo = {
    /**
     * UUID of the performance.
     */
    uuid: UuidString,

    /**
     * Player UUID.
     */
    player_uuid: UuidString,

    /**
     * List of library entry UUIDs that are proof of this performance.
     */
    proof: UuidString[],

    /**
     * Optional user comment.
     */
    comment?: string | null,

    /** 
     * Any additional performance metadata.
     */
    metadata: PerformanceMetadata,
};

export type CommonMatchInfo = {
    /**
     * UUID of the match.
     */
    uuid: UuidString,

    /** 
     * Timestamp of the match - specifically, the timestamp of the first frame of the end screen. Can be approximate.
     */
    timestamp: NsTimestamp,

    /**
     * Named ID of the song.
     */
    song_id: string,

    /**
     * Performances belonging to this match.
     */
    performance_ids: UuidString[],

    /**
     * List of library entry UUIDs that are proof of this match.
     */
    proof: UuidString[],

    /**
     * Optional user comment.
     */
    comment?: string | null,

    /**
     * Any additional match metadata.
     */
    metadata: MatchMetadata,
};
