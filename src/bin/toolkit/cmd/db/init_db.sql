
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
COMMENT ON SCHEMA $SCHEMA_NAME IS 'scoretracker schema created by scoretracker-toolkit';
SET search_path TO $SCHEMA_NAME;



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



-- Table: charts
-- DROP TABLE IF EXISTS charts;
CREATE TABLE IF NOT EXISTS charts
(
    game text COLLATE pg_catalog."default" NOT NULL,
    song_id text COLLATE pg_catalog."default" NOT NULL,
    chart_type text COLLATE pg_catalog."default" NOT NULL,
    details jsonb NOT NULL,
    chart_group text COLLATE pg_catalog."default" NOT NULL,
    CONSTRAINT chart_key PRIMARY KEY (game, song_id, chart_type)
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS charts OWNER TO scoretracker_dev;



-- Table: library
-- DROP TABLE IF EXISTS library;
CREATE TABLE IF NOT EXISTS library
(
    uuid uuid NOT NULL,
    sha256 bytea,
    library_urls text[] COLLATE pg_catalog."default" NOT NULL,
    youtube_id character(11) COLLATE pg_catalog."default",
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
    tags character varying(256)[] COLLATE pg_catalog."default",
    timestamp_added timestamp with time zone NOT NULL DEFAULT now(),
    metadata jsonb,
    CONSTRAINT library_pkey PRIMARY KEY (uuid)
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS library OWNER TO scoretracker_dev;



-- Table: matches
-- DROP TABLE IF EXISTS matches;
CREATE TABLE IF NOT EXISTS matches
(
    uuid uuid NOT NULL,
    "timestamp" timestamp(6) with time zone NOT NULL,
    song_id text COLLATE pg_catalog."default" NOT NULL,
    proof uuid[] NOT NULL,
    game text COLLATE pg_catalog."default" NOT NULL,
    details jsonb NOT NULL,
    metadata jsonb,
    CONSTRAINT matches_pkey PRIMARY KEY (uuid)
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS matches OWNER TO scoretracker_dev;



-- Table: players
-- DROP TABLE IF EXISTS players;
CREATE TABLE IF NOT EXISTS players
(
    uuid uuid NOT NULL,
    name character varying(32) COLLATE pg_catalog."default",
    CONSTRAINT players_pkey PRIMARY KEY (uuid)
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS players OWNER TO scoretracker_dev;



-- Table: performances
-- DROP TABLE IF EXISTS performances;
CREATE TABLE IF NOT EXISTS performances
(
    uuid uuid NOT NULL,
    player_uuid uuid NOT NULL,
    match_uuid uuid NOT NULL,
    proof uuid[],
    details jsonb NOT NULL,
    metadata jsonb,
    legit_fc boolean,
    chart_type text COLLATE pg_catalog."default",
    CONSTRAINT performances_pkey PRIMARY KEY (uuid),
    CONSTRAINT performances_match_uuid_fkey FOREIGN KEY (match_uuid)
        REFERENCES matches (uuid) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
        NOT VALID,
    CONSTRAINT performances_player_uuid_fkey FOREIGN KEY (player_uuid)
        REFERENCES players (uuid) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
)
TABLESPACE pg_default;
ALTER TABLE IF EXISTS performances OWNER TO scoretracker_dev;
COMMENT ON COLUMN performances.chart_type IS 'instrument id + difficulty id';

