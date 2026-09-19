
-- Database: $1

-- DROP DATABASE IF EXISTS $1;

CREATE DATABASE $1
    WITH
    OWNER = postgres
    ENCODING = 'UTF8'
    LC_COLLATE = 'English_United States.1250'
    LC_CTYPE = 'English_United States.1250'
    LOCALE_PROVIDER = 'libc'
    TABLESPACE = pg_default
    CONNECTION LIMIT = -1
    IS_TEMPLATE = False;

COMMENT ON DATABASE $1
    IS 'Database for scoretracker created by scoretracker-toolkit .';


    
-- SCHEMA: public

-- DROP SCHEMA IF EXISTS public ;

CREATE SCHEMA IF NOT EXISTS public
    AUTHORIZATION pg_database_owner;

COMMENT ON SCHEMA public
    IS 'standard public schema';

GRANT USAGE ON SCHEMA public TO PUBLIC;

GRANT ALL ON SCHEMA public TO pg_database_owner;



-- Type: cloth_info

-- DROP TYPE IF EXISTS public.cloth_info;

CREATE TYPE public.cloth_info AS
(
	uuid uuid,
	start_point double precision,
	end_point double precision
);

ALTER TYPE public.cloth_info
    OWNER TO postgres;



-- Type: quality_state

-- DROP TYPE IF EXISTS public.quality_state;

CREATE TYPE public.quality_state AS ENUM
    ('raw', 'folded', 'messy', 'crumpled', 'shredded');

ALTER TYPE public.quality_state
    OWNER TO postgres;



-- Table: public.charts

-- DROP TABLE IF EXISTS public.charts;

CREATE TABLE IF NOT EXISTS public.charts
(
    game text COLLATE pg_catalog."default" NOT NULL,
    song_id text COLLATE pg_catalog."default" NOT NULL,
    chart_type text COLLATE pg_catalog."default" NOT NULL,
    details jsonb NOT NULL,
    chart_group text COLLATE pg_catalog."default" NOT NULL,
    CONSTRAINT chart_key PRIMARY KEY (game, song_id, chart_type)
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.charts
    OWNER to postgres;



-- Table: public.library

-- DROP TABLE IF EXISTS public.library;

CREATE TABLE IF NOT EXISTS public.library
(
    uuid uuid NOT NULL,
    sha256 bytea,
    library_urls text[] COLLATE pg_catalog."default" NOT NULL,
    youtube_id character(11) COLLATE pg_catalog."default",
    entry_kind library_entry_kind_old NOT NULL,
    file_stat jsonb,
    media_metadata jsonb,
    media_category media_category_old,
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

ALTER TABLE IF EXISTS public.library
    OWNER to postgres;



-- Table: public.matches

-- DROP TABLE IF EXISTS public.matches;

CREATE TABLE IF NOT EXISTS public.matches
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

ALTER TABLE IF EXISTS public.matches
    OWNER to postgres;



-- Table: public.performances

-- DROP TABLE IF EXISTS public.performances;

CREATE TABLE IF NOT EXISTS public.performances
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
        REFERENCES public.matches (uuid) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
        NOT VALID,
    CONSTRAINT performances_player_uuid_fkey FOREIGN KEY (player_uuid)
        REFERENCES public.players (uuid) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.performances
    OWNER to postgres;

COMMENT ON COLUMN public.performances.chart_type
    IS 'instrument id + difficulty id';



-- Table: public.players

-- DROP TABLE IF EXISTS public.players;

CREATE TABLE IF NOT EXISTS public.players
(
    uuid uuid NOT NULL,
    name character varying(32) COLLATE pg_catalog."default",
    CONSTRAINT players_pkey PRIMARY KEY (uuid)
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.players
    OWNER to postgres;