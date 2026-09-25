pub mod schema_name;

use std::str::FromStr;

use function_name::named;
use thiserror::Error;
use tokio_postgres::Transaction;
use uuid::Uuid;

use crate::{
    config::{toml::TomlConfigError, toolkit::ToolkitConfig},
    data::{
        library::{entry::Proof, stpl_url::LibraryDomain},
        scoreboard::{r#match::Match, performance::Performance, player::Player},
        song::song::Song,
    },
    db::schema_name::SafeSchemaName,
    info, log_fn_name, success,
};

/// Asynchronous database connection.
pub struct Database {
    pub client: tokio_postgres::Client,
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("postgres error: {0:?} {0}")]
    PostgresError(#[from] tokio_postgres::Error),
    #[error("json error: {0:?} {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("config error: {0}")]
    GlobalConfigError(#[from] &'static TomlConfigError),
    #[error("toolkit config is missing database connection string")]
    ToolkitConfigNoDbConnectionString,
    #[error("toolkit config is missing database schema name")]
    ToolkitConfigNoDbSchemaName,
}

pub type DbResult<T> = Result<T, DbError>;

pub struct Pagination {
    limit: u32,
    offset: u32,
}

pub struct DbExport {
    pub players: Vec<Player>,
    pub matches: Vec<Match>,
    pub performances: Vec<Performance>,
    pub proofs: Vec<Proof>,
    pub songs: Vec<Song>,
}

/// Usually Vecs that contain database results use the pagination limit as their capacity,
/// but that's bad if someone puts a "no limit" value (like 9999) as the pagination limit.
/// We don't want to over-allocate an insane number of bytes if we know we don't have this many records in the database.
/// This value can be used as a limit to reserving vector capacity.
const PREALLOCATE_CAPACITY_LIMIT: u32 = 100;

impl Database {
    #[named]
    pub async fn connect_with_actix_web(connection_string: &str, schema_name: &str) -> DbResult<Self> {
        log_fn_name!(auto);

        let config = tokio_postgres::Config::from_str(connection_string)?;
        // info!("config: {config:?}");

        let (client, connection) = config.connect(tokio_postgres::NoTls).await?;

        actix_web::rt::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        success!("connected to database");
        client.execute(&format!("SET search_path TO '{schema_name}'"), &[]).await?;

        success!("set database search path to '{schema_name}'");
        Ok(Self { client: client })
    }

    #[named]
    pub async fn connect_with_tokio(connection_string: &str, schema_name: &SafeSchemaName) -> DbResult<Self> {
        log_fn_name!(auto);

        let config = tokio_postgres::Config::from_str(connection_string)?;
        // info!("config: {config:?}");

        let (client, connection) = config.connect(tokio_postgres::NoTls).await?;

        tokio::task::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        success!("connected to database");
        client.execute(&format!("SET search_path TO '{schema_name}'"), &[]).await?;

        success!("set database search path to '{schema_name}'");
        Ok(Self { client: client })
    }

    pub async fn connect_with_tokio_for_toolkit() -> DbResult<Self> {
        let config = ToolkitConfig::global()?;
        Self::connect_with_tokio(
            config
                .database_connection
                .as_ref()
                .ok_or(DbError::ToolkitConfigNoDbConnectionString)?,
            config.database_schema.as_ref().ok_or(DbError::ToolkitConfigNoDbSchemaName)?,
        )
        .await
    }

    pub async fn list_matches(&mut self, paging: Pagination) -> DbResult<Vec<Match>> {
        let results = self
            .client
            .query(
                "SELECT match_uuid, timestamp, chartset_id, proof, details, metadata FROM matches LIMIT $1 OFFSET $2",
                &[&paging.limit, &paging.offset],
            )
            .await?;

        let mut matches = Vec::with_capacity(paging.limit.min(PREALLOCATE_CAPACITY_LIMIT) as usize);
        for row in results {
            let match_info = Match::from_postgres_row(&row)?;
            matches.push(match_info);
        }
        Ok(matches)
    }

    pub async fn find_player_by_name(&mut self, name: &str) -> DbResult<Option<Player>> {
        let results = self
            .client
            .query("SELECT player_uuid, name FROM players LIMIT 1 WHERE name = $?", &[&name])
            .await?;

        if let Some(row) = results.first() {
            Ok(Some(Player::from_postgres_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn find_players_by_name(&mut self, _names: &[&str]) -> DbResult<Vec<Player>> {
        todo!()
    }

    pub async fn find_proof_by_youtube_id(&mut self, _youtube_id: &str) -> DbResult<Option<Proof>> {
        todo!()
    }

    pub async fn find_proofs_by_youtube_id(&mut self, _youtube_ids: &[&str]) -> DbResult<Vec<Proof>> {
        todo!()
    }

    pub async fn find_match_by_uuid(&mut self, _uuid: Uuid) -> DbResult<Option<Match>> {
        todo!()
    }

    pub async fn find_matches_by_uuid(&mut self, _uuids: &[Uuid]) -> DbResult<Vec<Match>> {
        todo!()
    }

    pub async fn find_performance_by_uuid(&mut self, _uuid: Uuid) -> DbResult<Option<Performance>> {
        todo!()
    }

    pub async fn find_performances_by_uuid(&mut self, _uuids: &[Uuid]) -> DbResult<Vec<Performance>> {
        todo!()
    }

    pub async fn remove_library_domain_from_db(&mut self, library_domain: &LibraryDomain) -> DbResult<()> {
        /*
        for entry in library_db.entries.iter_mut() {
            // Remove all old URLs that reference this library, without touching all of the other ones.
            entry.library_urls.retain(|url| url.domain != library_domain);
        }
         */
        todo!()
    }

    #[named]
    pub async fn get_or_insert_player(&mut self, player_name: &str) -> DbResult<(Uuid, bool)> {
        log_fn_name!(auto);

        let transaction = self.client.transaction().await?;
        let query = transaction
            .query("SELECT player_uuid FROM players WHERE name = $1 LIMIT 1", &[&player_name])
            .await?;

        if let Some(row) = query.first() {
            transaction.commit().await?;
            let player_uuid = row.try_get("player_uuid")?;

            info!("fetched existing player successfully");
            Ok((player_uuid, false))
        } else {
            let player_uuid = Uuid::now_v7();
            let count = transaction
                .execute(
                    "INSERT INTO players (player_uuid, name) VALUES ($1, $2)",
                    &[&player_uuid, &player_name],
                )
                .await?;
            transaction.commit().await?;

            success!("added new player successfully ({count} rows affected)");
            Ok((player_uuid, true))
        }
    }

    #[named]
    pub async fn export_players<'a>(transaction: &Transaction<'a>) -> DbResult<Vec<Player>> {
        log_fn_name!(auto);

        let records = transaction.query("SELECT player_uuid, name FROM players", &[]).await?;
        let mut players = Vec::with_capacity(records.len());
        for record in records {
            let player = Player::from_postgres_row(&record)?;
            players.push(player);
        }
        Ok(players)
    }

    #[named]
    pub async fn export_proofs<'a>(transaction: &Transaction<'a>) -> DbResult<Vec<Proof>> {
        log_fn_name!(auto);

        let records = transaction.query("SELECT proof_uuid, sha256, library_urls, youtube_id, entry_kind, file_stat, media_metadata, media_category, content_description, cut, quality, cloth, dry, clips, timestamp_start, timestamp_end, duration, automatic_content_detection_information, tags, timestamp_added, metadata FROM proofs", &[]).await?;
        let mut proofs = Vec::with_capacity(records.len());
        for record in records {
            let proof = Proof::from_postgres_row(&record)?;
            proofs.push(proof);
        }
        Ok(proofs)
    }

    #[named]
    pub async fn export_performances<'a>(transaction: &Transaction<'a>) -> DbResult<Vec<Performance>> {
        log_fn_name!(auto);

        let records = transaction.query("SELECT performance_uuid, player_uuid, match_uuid, game, chartset_id, instrument, difficulty, details, metadata, legit_fc, array_agg(proof_uuid) AS proofs FROM performances LEFT JOIN performance_proofs USING (performance_uuid) GROUP BY performance_uuid", &[]).await?;
        let mut performances = Vec::with_capacity(records.len());
        for record in records {
            let performance = Performance::from_postgres_row(&record)?;
            performances.push(performance);
        }
        Ok(performances)
    }

    #[named]
    pub async fn export_matches<'a>(transaction: &Transaction<'a>) -> DbResult<Vec<Match>> {
        log_fn_name!(auto);

        let records = transaction
            .query(
                "SELECT match_uuid, timestamp, game, chartset_id, proof, details, metadata FROM matches",
                &[],
            )
            .await?;
        let mut matches = Vec::with_capacity(records.len());
        for record in records {
            let match_info = Match::from_postgres_row(&record)?;
            matches.push(match_info);
        }
        Ok(matches)
    }

    #[named]
    pub async fn export_songs<'a>(transaction: &Transaction<'a>) -> DbResult<Vec<Song>> {
        log_fn_name!(auto);

        let records = transaction.query("SELECT song_id, title, artist, year FROM songs", &[]).await?;
        let mut songs = Vec::with_capacity(records.len());
        for record in records {
            let song = Song::from_postgres_row(&record)?;
            songs.push(song);
        }
        Ok(songs)
    }

    #[named]
    pub async fn export_all(&mut self) -> DbResult<DbExport> {
        log_fn_name!(auto);

        let transaction = self.client.transaction().await?;

        let players = Self::export_players(&transaction).await?;
        let proofs = Self::export_proofs(&transaction).await?;
        let performances = Self::export_performances(&transaction).await?;
        let matches = Self::export_matches(&transaction).await?;
        let songs = Self::export_songs(&transaction).await?;

        let export = DbExport {
            players,
            matches,
            performances,
            proofs,
            songs,
        };
        success!(
            "fetched {} players, {} matches, {} performances, {} proofs, {} songs from the database",
            export.players.len(),
            export.matches.len(),
            export.performances.len(),
            export.proofs.len(),
            export.songs.len()
        );
        Ok(export)
    }
}
