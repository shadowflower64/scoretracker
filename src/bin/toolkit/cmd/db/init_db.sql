
-- -- Database: postgres
-- -- DROP DATABASE IF EXISTS postgres;
-- CREATE DATABASE postgres
--     WITH
--     OWNER = postgres
--     ENCODING = 'UTF8'
--     LC_COLLATE = 'English_United States.1250'
--     LC_CTYPE = 'English_United States.1250'
--     LOCALE_PROVIDER = 'libc'
--     TABLESPACE = pg_default
--     CONNECTION LIMIT = -1
--     IS_TEMPLATE = False;
-- COMMENT ON DATABASE postgres
--     IS 'Database for scoretracker created by scoretracker-toolkit .';


    
-- SCHEMA: $SCHEMA_NAME
-- DROP SCHEMA IF EXISTS $SCHEMA_NAME;
-- CREATE SCHEMA IF NOT EXISTS $SCHEMA_NAME AUTHORIZATION scoretracker_dev;
-- COMMENT ON SCHEMA $SCHEMA_NAME IS 'scoretracker schema created by scoretracker-toolkit';
-- SET search_path TO $SCHEMA_NAME;



-- Type: cloth_info
-- DROP TYPE IF EXISTS cloth_info;
CREATE TYPE cloth_info AS
(
	uuid uuid,
	start_point double precision,
	end_point double precision
);
ALTER TYPE cloth_info OWNER TO scoretracker_dev;



-- Type: quality_state
-- DROP TYPE IF EXISTS quality_state;
CREATE TYPE quality_state AS ENUM ('raw', 'folded', 'messy', 'crumpled', 'shredded');
ALTER TYPE quality_state OWNER TO scoretracker_dev;



-- Type: library_entry_kind
-- DROP TYPE IF EXISTS library_entry_kind;
CREATE TYPE library_entry_kind AS ENUM ('unspecified', 'not_proof', 'unsupported', 'not_linked_yet', 'linked');
ALTER TYPE library_entry_kind OWNER TO scoretracker_dev;



-- Type: media_category
-- DROP TYPE IF EXISTS media_category;
CREATE TYPE media_category AS ENUM ('unspecified', 'other', 'pc_screenshot', 'mobile_screenshot', 'camera_photo', 'obs_recording', 'obs_recording_autocut', 'obs_recording_lossless_cut', 'obs_replay', 'obs_replay_autocut', 'obs_replay_lossless_cut', 'mobile_screen_recording', 'mobile_screen_recording_lossless_cut', 'camera_video');
ALTER TYPE media_category OWNER TO scoretracker_dev;



-- Table: songs
-- DROP TABLE IF EXISTS songs;
CREATE TABLE IF NOT EXISTS songs
(
    song_id text NOT NULL PRIMARY KEY,
    title text,
    artist text,
    album text,
    year integer
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS songs OWNER TO scoretracker_dev;



-- Table: chartsets
-- DROP TABLE IF EXISTS chartsets;
CREATE TABLE IF NOT EXISTS chartsets
(
    game text NOT NULL,
    chartset_id text NOT NULL,
    song_id text NOT NULL REFERENCES songs (song_id),
    details jsonb,
    PRIMARY KEY (game, chartset_id)
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS chartsets OWNER TO scoretracker_dev;



-- Table: charts
-- DROP TABLE IF EXISTS charts;
CREATE TABLE IF NOT EXISTS charts
(
    chart_id text NOT NULL,
    game text NOT NULL,
    chartset_id text NOT NULL,
    instrument text NOT NULL,
    difficulty text NOT NULL,
    details jsonb NOT NULL,
    chart_group text NOT NULL,
    song_id_override text REFERENCES songs (song_id),
    PRIMARY KEY (chart_id),
    FOREIGN KEY (game, chartset_id) REFERENCES chartsets
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS charts OWNER TO scoretracker_dev;



-- Table: library
-- DROP TABLE IF EXISTS library;
CREATE TABLE IF NOT EXISTS library
(
    proof_uuid uuid NOT NULL PRIMARY KEY,
    sha256 bytea,
    library_urls text[] NOT NULL,
    youtube_id character(11),
    entry_kind library_entry_kind NOT NULL,
    file_stat jsonb,
    media_metadata jsonb,
    media_category media_category,
    content_description jsonb,
    cut boolean,
    quality quality_state,
    cloth cloth_info,
    dry uuid,
    clips uuid[],
    timestamp_start timestamp with time zone,
    timestamp_end timestamp with time zone,
    duration double precision,
    automatic_content_detection_information jsonb,
    tags character varying(256)[],
    timestamp_added timestamp with time zone NOT NULL DEFAULT now(),
    metadata jsonb
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS library OWNER TO scoretracker_dev;



-- Table: players
-- DROP TABLE IF EXISTS players;
CREATE TABLE IF NOT EXISTS players
(
    player_uuid uuid NOT NULL PRIMARY KEY,
    name character varying(32)
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS players OWNER TO scoretracker_dev;



-- Table: matches
-- DROP TABLE IF EXISTS matches;
CREATE TABLE IF NOT EXISTS matches
(
    match_uuid uuid NOT NULL PRIMARY KEY,
    "timestamp" timestamp(6) with time zone NOT NULL,
    game text NOT NULL,
    chartset_id text NOT NULL,
    details jsonb NOT NULL,
    metadata jsonb,
    FOREIGN KEY (game, chartset_id) REFERENCES chartsets
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS matches OWNER TO scoretracker_dev;



-- Table: match_proofs
-- DROP TABLE IF EXISTS match_proofs;
CREATE TABLE IF NOT EXISTS match_proofs
(
    match_uuid uuid NOT NULL REFERENCES matches,
    proof_uuid uuid NOT NULL REFERENCES library,
    UNIQUE (match_uuid, proof_uuid)
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS match_proofs OWNER TO scoretracker_dev;



-- Table: performances
-- DROP TABLE IF EXISTS performances;
CREATE TABLE IF NOT EXISTS performances
(
    performance_uuid uuid NOT NULL PRIMARY KEY,
    player_uuid uuid NOT NULL REFERENCES players,
    match_uuid uuid NOT NULL REFERENCES matches,
    chart_id text NOT NULL REFERENCES charts,
    details jsonb NOT NULL,
    metadata jsonb,
    legit_fc boolean,
    FOREIGN KEY (chart_id) REFERENCES charts
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS performances OWNER TO scoretracker_dev;



-- Table: performance_proofs
-- DROP TABLE IF EXISTS performance_proofs;
CREATE TABLE IF NOT EXISTS performance_proofs
(
    performance_uuid uuid NOT NULL REFERENCES performances,
    proof_uuid uuid NOT NULL REFERENCES library,
    UNIQUE (performance_uuid, proof_uuid)
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS performance_proofs OWNER TO scoretracker_dev;
