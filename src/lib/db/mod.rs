pub mod schema_name;

use std::str::FromStr;

use function_name::named;
use thiserror::Error;
use tokio_postgres::Transaction;
use uuid::Uuid;

use crate::{
    config::{toml::TomlConfigError, toolkit::ToolkitConfig},
    data::{
        chart::{chart::Chart, chartset::Chartset, song::Song},
        library::{proof::Proof, stpl_url::LibraryDomain},
        scoreboard::{r#match::Match, performance::Performance, player::Player},
    },
    db::schema_name::SafeSchemaName,
    debug, info, log_fn_name, log_should_print_debug, success,
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
    pub limit: u32,
    pub offset: u32,
}

pub struct DbExport {
    pub players: Vec<Player>,
    pub songs: Vec<Song>,
    pub chartsets: Vec<Chartset>,
    pub charts: Vec<Chart>,
    pub proofs: Vec<Proof>,
    pub matches: Vec<Match>,
    pub performances: Vec<Performance>,
}

/// Usually Vecs that contain database results use the pagination limit as their capacity,
/// but that's bad if someone puts a "no limit" value (like 9999) as the pagination limit.
/// We don't want to over-allocate an insane number of bytes if we know we don't have this many records in the database.
/// This value can be used as a limit to reserving vector capacity.
const PREALLOCATE_CAPACITY_LIMIT: u32 = 100;

impl Database {
    #[named]
    pub async fn connect_with_actix_web(connection_string: &str, schema_name: &SafeSchemaName) -> DbResult<Self> {
        log_fn_name!(auto);
        log_should_print_debug!(false);

        let config = tokio_postgres::Config::from_str(connection_string)?;
        debug!("config: {config:?}");

        let (client, connection) = config.connect(tokio_postgres::NoTls).await?;
        debug!("client");

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
        log_should_print_debug!(false);

        let config = tokio_postgres::Config::from_str(connection_string)?;
        debug!("config: {config:?}");

        let (client, connection) = config.connect(tokio_postgres::NoTls).await?;
        debug!("client");

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
        info!("exporting players");
        let records = transaction.query("SELECT player_uuid, name FROM players", &[]).await?;
        let mut players = Vec::with_capacity(records.len());
        for record in records {
            let player = Player::from_postgres_row(&record)?;
            players.push(player);
        }
        Ok(players)
    }

    #[named]
    pub async fn export_songs<'a>(transaction: &Transaction<'a>) -> DbResult<Vec<Song>> {
        log_fn_name!(auto);
        info!("exporting songs");
        let records = transaction
            .query("SELECT song_id, title, artist, album, year FROM songs", &[])
            .await?;
        let mut songs = Vec::with_capacity(records.len());
        for record in records {
            let song = Song::from_postgres_row(&record)?;
            songs.push(song);
        }
        Ok(songs)
    }

    #[named]
    pub async fn export_chartsets<'a>(transaction: &Transaction<'a>) -> DbResult<Vec<Chartset>> {
        log_fn_name!(auto);
        info!("exporting chartsets");
        let records = transaction
            .query("SELECT game, chartset_id, song_id, title, artist, details FROM chartsets", &[])
            .await?;
        let mut chartsets = Vec::with_capacity(records.len());
        for record in records {
            let chartset = Chartset::from_postgres_row(&record)?;
            chartsets.push(chartset);
        }
        Ok(chartsets)
    }

    #[named]
    pub async fn export_charts<'a>(transaction: &Transaction<'a>) -> DbResult<Vec<Chart>> {
        log_fn_name!(auto);
        info!("exporting charts");
        let records = transaction
            .query(
                "SELECT chart_id, game, chartset_id, instrument, difficulty, chart_group, song_id_override, details FROM charts",
                &[],
            )
            .await?;
        let mut charts = Vec::with_capacity(records.len());
        for record in records {
            let chart = Chart::from_postgres_row(&record)?;
            charts.push(chart);
        }
        Ok(charts)
    }

    #[named]
    pub async fn export_proofs<'a>(transaction: &Transaction<'a>) -> DbResult<Vec<Proof>> {
        log_fn_name!(auto);
        info!("exporting proofs");
        let records = transaction.query("SELECT proof_uuid, sha256, library_urls, youtube_id, entry_kind, file_stat, media_metadata, media_category, content_description, cut, quality, cloth, dry, clips, timestamp_start, timestamp_end, duration, automatic_content_detection_information, tags, timestamp_added, metadata FROM proofs", &[]).await?;
        let mut proofs = Vec::with_capacity(records.len());
        for record in records {
            let proof = Proof::from_postgres_row(&record)?;
            proofs.push(proof);
        }
        Ok(proofs)
    }

    #[named]
    pub async fn export_matches<'a>(transaction: &Transaction<'a>) -> DbResult<Vec<Match>> {
        log_fn_name!(auto);
        info!("exporting matches");
        let records = transaction
            .query(
                "
                SELECT match_uuid, timestamp, game, chartset_id, details, metadata, timestamp_added, array_remove(array_agg(proof_uuid), NULL) AS proofs
                FROM matches
                LEFT JOIN match_proofs USING (match_uuid)
                GROUP BY match_uuid
            ",
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
    pub async fn export_performances<'a>(transaction: &Transaction<'a>) -> DbResult<Vec<Performance>> {
        log_fn_name!(auto);
        info!("exporting performances");
        let records = transaction
            .query(
                "
            SELECT performance_uuid, player_uuid, match_uuid, chart_id, details, metadata, timestamp_added, legit_fc, array_remove(array_agg(proof_uuid), NULL) AS proofs
            FROM performances
            LEFT JOIN performance_proofs USING (performance_uuid)
            GROUP BY performance_uuid
        ",
                &[],
            )
            .await?;
        let mut performances = Vec::with_capacity(records.len());
        for record in records {
            let performance = Performance::from_postgres_row(&record)?;
            performances.push(performance);
        }
        Ok(performances)
    }

    #[named]
    pub async fn export_all(&mut self) -> DbResult<DbExport> {
        log_fn_name!(auto);

        let transaction = self.client.transaction().await?;

        let players = Self::export_players(&transaction).await?;
        let songs = Self::export_songs(&transaction).await?;
        let chartsets = Self::export_chartsets(&transaction).await?;
        let charts = Self::export_charts(&transaction).await?;
        let proofs = Self::export_proofs(&transaction).await?;
        let performances = Self::export_performances(&transaction).await?;
        let matches = Self::export_matches(&transaction).await?;

        transaction.commit().await?;

        let export = DbExport {
            players,
            songs,
            chartsets,
            charts,
            proofs,
            matches,
            performances,
        };

        success!(
            "fetched {} players, {} songs, {} chartsets, {} charts, {} proofs, {} matches, {} performances from the database",
            export.players.len(),
            export.songs.len(),
            export.chartsets.len(),
            export.charts.len(),
            export.proofs.len(),
            export.matches.len(),
            export.performances.len(),
        );
        Ok(export)
    }

    #[named]
    pub async fn import_players<'a>(transaction: &Transaction<'a>, players: &[Player]) -> DbResult<()> {
        log_fn_name!(auto);
        info!("importing {} players", players.len());
        let insert_statement = transaction
            .prepare(
                "INSERT INTO players (player_uuid, name)
            VALUES ($1, $2)
            ON CONFLICT (player_uuid)
            DO UPDATE SET name = EXCLUDED.name",
            )
            .await?;
        for player in players {
            transaction.execute(&insert_statement, &[&player.player_uuid, &player.name]).await?;
        }
        Ok(())
    }

    #[named]
    pub async fn import_songs<'a>(transaction: &Transaction<'a>, songs: &[Song]) -> DbResult<()> {
        log_fn_name!(auto);
        info!("importing {} songs", songs.len());
        let insert_statement = transaction
            .prepare(
                "INSERT INTO songs (song_id, title, artist, album, year)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (song_id)
            DO UPDATE SET
                title = EXCLUDED.title,
                artist = EXCLUDED.artist,
                album = EXCLUDED.album,
                year = EXCLUDED.year",
            )
            .await?;
        for song in songs {
            transaction
                .execute(
                    &insert_statement,
                    &[&song.song_id, &song.title, &song.artist, &song.album, &song.year],
                )
                .await?;
        }
        Ok(())
    }

    #[named]
    pub async fn import_chartsets<'a>(transaction: &Transaction<'a>, chartsets: &[Chartset]) -> DbResult<()> {
        log_fn_name!(auto);
        info!("importing {} chartsets", chartsets.len());
        let insert_statement = transaction
            .prepare(
                "INSERT INTO chartsets (game, chartset_id, song_id, title, artist, details)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (game, chartset_id)
            DO UPDATE SET
                song_id = EXCLUDED.song_id,
                title = EXCLUDED.title,
                artist = EXCLUDED.artist,
                details = EXCLUDED.details",
            )
            .await?;
        for chartset in chartsets {
            transaction
                .execute(
                    &insert_statement,
                    &[
                        &chartset.game,
                        &chartset.chartset_id,
                        &chartset.song_id,
                        &chartset.title,
                        &chartset.artist,
                        &chartset.details,
                    ],
                )
                .await?;
        }
        Ok(())
    }

    #[named]
    pub async fn import_charts<'a>(transaction: &Transaction<'a>, charts: &[Chart]) -> DbResult<()> {
        log_fn_name!(auto);
        info!("importing {} charts", charts.len());
        let insert_statement = transaction
            .prepare(
                "INSERT INTO charts (chart_id, game, chartset_id, instrument, difficulty, chart_group, song_id_override, details)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (chart_id)
            DO UPDATE SET
                game = EXCLUDED.game,
                chartset_id = EXCLUDED.chartset_id,
                instrument = EXCLUDED.instrument,
                difficulty = EXCLUDED.difficulty,
                chart_group = EXCLUDED.chart_group,
                song_id_override = EXCLUDED.song_id_override,
                details = EXCLUDED.details",
            )
            .await?;
        for chart in charts {
            transaction
                .execute(
                    &insert_statement,
                    &[
                        &chart.chart_id,
                        &chart.game,
                        &chart.chartset_id,
                        &chart.instrument,
                        &chart.difficulty,
                        &chart.chart_group,
                        &chart.song_id_override,
                        &chart.details,
                    ],
                )
                .await?;
        }
        Ok(())
    }

    #[named]
    pub async fn import_proofs<'a>(transaction: &Transaction<'a>, proofs: &[Proof]) -> DbResult<()> {
        log_fn_name!(auto);
        info!("importing {} proofs", proofs.len());
        let insert_statement = transaction
            .prepare(
                "INSERT INTO proofs (proof_uuid, sha256, library_urls, youtube_id, entry_kind, file_stat, media_metadata, media_category, content_description, cut, quality, cloth, dry, clips, timestamp_start, timestamp_end, duration, automatic_content_detection_information, tags, timestamp_added, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            ON CONFLICT (proof_uuid)
            DO UPDATE SET
                sha256 = EXCLUDED.sha256, 
                library_urls = EXCLUDED.library_urls, 
                youtube_id = EXCLUDED.youtube_id, 
                entry_kind = EXCLUDED.entry_kind, 
                file_stat = EXCLUDED.file_stat, 
                media_metadata = EXCLUDED.media_metadata, 
                media_category = EXCLUDED.media_category, 
                content_description = EXCLUDED.content_description, 
                cut = EXCLUDED.cut, 
                quality = EXCLUDED.quality, 
                cloth = EXCLUDED.cloth, 
                dry = EXCLUDED.dry, 
                clips = EXCLUDED.clips, 
                timestamp_start = EXCLUDED.timestamp_start, 
                timestamp_end = EXCLUDED.timestamp_end, 
                duration = EXCLUDED.duration, 
                automatic_content_detection_information = EXCLUDED.automatic_content_detection_information, 
                tags = EXCLUDED.tags, 
                timestamp_added = EXCLUDED.timestamp_added, 
                metadata = EXCLUDED.metadata",
            )
            .await?;
        for proof in proofs {
            transaction
                .execute(
                    &insert_statement,
                    &[
                        &proof.proof_uuid,
                        &proof.sha256,
                        &proof.library_urls,
                        &proof.youtube_id,
                        &proof.entry_kind,
                        &proof.file_stat,
                        &proof.media_metadata,
                        &proof.media_category,
                        &proof.content_description,
                        &proof.cut,
                        &proof.quality,
                        &proof.cloth,
                        &proof.dry,
                        &proof.clips,
                        &proof.timestamp_start,
                        &proof.timestamp_end,
                        &proof.duration,
                        &proof.automatic_content_detection_information,
                        &proof.tags,
                        &proof.timestamp_added,
                        &proof.metadata,
                    ],
                )
                .await?;
        }
        Ok(())
    }

    #[named]
    pub async fn import_matches<'a>(transaction: &Transaction<'a>, matches: &[Match]) -> DbResult<()> {
        log_fn_name!(auto);
        info!("importing {} matches", matches.len());
        let insert_statement = transaction
            .prepare(
                "INSERT INTO matches (match_uuid, timestamp, game, chartset_id, details, metadata, timestamp_added)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (match_uuid)
            DO UPDATE SET
                timestamp = EXCLUDED.timestamp,
                game = EXCLUDED.game,
                chartset_id = EXCLUDED.chartset_id,
                details = EXCLUDED.details,
                metadata = EXCLUDED.metadata,
                timestamp_added = EXCLUDED.timestamp_added",
            )
            .await?;
        for match_info in matches {
            transaction
                .execute(
                    &insert_statement,
                    &[
                        &match_info.match_uuid,
                        &match_info.timestamp,
                        &match_info.game,
                        &match_info.chartset_id,
                        &match_info.details,
                        &match_info.metadata,
                        &match_info.timestamp_added,
                    ],
                )
                .await?;
        }
        Ok(())
    }

    #[named]
    pub async fn import_performances<'a>(transaction: &Transaction<'a>, performances: &[Performance]) -> DbResult<()> {
        log_fn_name!(auto);
        info!("importing {} performances", performances.len());
        let insert_statement = transaction
            .prepare(
                "INSERT INTO performances (performance_uuid, player_uuid, match_uuid, chart_id, details, metadata, timestamp_added)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (performance_uuid)
            DO UPDATE SET
                player_uuid = EXCLUDED.player_uuid,
                match_uuid = EXCLUDED.match_uuid,
                chart_id = EXCLUDED.chart_id,
                details = EXCLUDED.details,
                metadata = EXCLUDED.metadata,
                timestamp_added = EXCLUDED.timestamp_added",
            )
            .await?;
        for performance in performances {
            transaction
                .execute(
                    &insert_statement,
                    &[
                        &performance.performance_uuid,
                        &performance.player_uuid,
                        &performance.match_uuid,
                        &performance.chart_id,
                        &performance.details,
                        &performance.metadata,
                        &performance.timestamp_added,
                    ],
                )
                .await?;
        }
        Ok(())
    }

    #[named]
    pub async fn import_all(&mut self, import: DbExport) -> DbResult<()> {
        log_fn_name!(auto);

        let transaction = self.client.transaction().await?;

        Self::import_players(&transaction, &import.players).await?;
        Self::import_songs(&transaction, &import.songs).await?;
        Self::import_chartsets(&transaction, &import.chartsets).await?;
        Self::import_charts(&transaction, &import.charts).await?;
        Self::import_proofs(&transaction, &import.proofs).await?;
        Self::import_matches(&transaction, &import.matches).await?;
        Self::import_performances(&transaction, &import.performances).await?;

        transaction.commit().await?;

        success!(
            "imported {} players into the database",
            import.players.len(),
            // import.matches.len(),
            // import.performances.len(),
            // import.proofs.len(),
            // import.songs.len()
        );
        Ok(())
    }
}
