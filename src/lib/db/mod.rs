use std::str::FromStr;

use function_name::named;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    config::{toml::TomlConfigError, toolkit::ToolkitConfig},
    data::{
        library::{entry::LibraryEntry, stpl_url::LibraryDomain},
        scoreboard::{r#match::Match, performance::Performance, player::Player},
    },
    info, log_fn_name, success,
};

/// Asynchronous database connection.
pub struct Database {
    pub client: tokio_postgres::Client,
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("postgres error: {0:?}")]
    PostgresError(#[from] tokio_postgres::Error),
    #[error("json error: {0:?}")]
    JsonError(#[from] serde_json::Error),
    #[error("config error: {0:?}")]
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

/// Usually Vecs that contain database results use the pagination limit as their capacity,
/// but that's bad if someone puts a "no limit" value (like 9999) as the pagination limit.
/// We don't want to over-allocate an insane number of bytes if we know we don't have this many records in the database.
/// This value can be used as a limit to reserving vector capacity.
const PREALLOCATE_CAPACITY_LIMIT: u32 = 100;

impl Database {
    #[named]
    pub async fn connect_and_spawn_actix_web(connection_string: &str, schema_name: &str) -> DbResult<Self> {
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
        client.execute("SET search_path TO $1", &[&schema_name]).await?;
        Ok(Self { client: client })
    }

    #[named]
    pub async fn connect_and_spawn_smol(connection_string: &str, schema_name: &str) -> DbResult<Self> {
        log_fn_name!(auto);

        let config = tokio_postgres::Config::from_str(connection_string)?;
        // info!("config: {config:?}");

        let (client, connection) = config.connect(tokio_postgres::NoTls).await?;

        smol::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        })
        .detach();

        success!("connected to database");
        client.execute("SET search_path TO $1", &[&schema_name]).await?;

        success!("set database search path to '{schema_name}'");
        Ok(Self { client: client })
    }

    pub async fn connect_and_spawn_smol_with_toolkit() -> DbResult<Self> {
        let config = ToolkitConfig::global()?;
        Self::connect_and_spawn_smol(
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

    pub async fn find_proof_by_youtube_id(&mut self, _youtube_id: &str) -> DbResult<Option<LibraryEntry>> {
        todo!()
    }

    pub async fn find_proofs_by_youtube_id(&mut self, _youtube_ids: &[&str]) -> DbResult<Vec<LibraryEntry>> {
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
            .query("SELECT player_uuid FROM players WHERE player_name = $1", &[&player_name])
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
                    "INSERT INTO players (player_uuid, player_name) VALUES ($1, $2)",
                    &[&player_uuid, &player_name],
                )
                .await?;
            transaction.commit().await?;

            success!("added new player successfully ({count} rows affected)");
            Ok((player_uuid, true))
        }
    }
}
