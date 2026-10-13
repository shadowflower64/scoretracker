use std::{collections::HashMap, sync::LazyLock};

use actix_web::{HttpRequest, post, web};
use function_name::named;
use scoretracker::{
    api::library::{ScanFullRequest, ScanFullResponse},
    data::library::stpl_url::StplUrl,
    info, log_fn_name,
    util::uuid::UuidString,
};

use crate::server::{globals::ServerGlobals, http::api::ApiResult};

// TODO: This should be a modifiable file
pub static LIBRARY_AUTH: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    let mut x = HashMap::new();
    x.insert("test.library", "hunter2");
    x
});

#[utoipa::path(
    context_path = "/api",
    responses(
        (status = OK, description = "TODO")
    )
)]
#[post("/library/scan_full")]
#[named]
pub async fn post_scan_full(body: web::Json<ScanFullRequest>, globals: web::Data<ServerGlobals>) -> ApiResult<ScanFullResponse, ()> {
    log_fn_name!(auto);
    info!("received post request for full scan");

    let body = body.into_inner();

    // Check authenticity of the library
    let Some(stored_auth_string) = LIBRARY_AUTH.get(body.library_domain.to_string().as_str()) else {
        panic!("library not registered");
    };
    if body.authenticity_string != *stored_auth_string {
        panic!("invalid authenticity string");
    }

    let mut db = globals.db.lock().await;

    // Remove ALL previous references to this library domain in the library URLs from the database
    db.remove_all_references_to_library_domain(&body.library_domain)
        .await
        .expect("todo");

    let mut proof_uuids = HashMap::new();

    // Add URLs for the specified files (meaning, all old files that used be in this library and are not present anymore will have their URLs removed, and any files currently present will have their URLs updated)
    for (internal_path, sha256_hash) in body.sha256_hashes {
        let url = StplUrl::new(body.library_domain.clone(), Some(internal_path.to_string()));

        // Insert new proof records in the case that a SHA256 hash was not found in the database
        let proof_uuid = db.add_url_to_proof_by_sha256_or_insert_new(&sha256_hash, url).await.expect("todo");
        proof_uuids.insert(internal_path, UuidString(proof_uuid));
    }

    // Return proof UUIDs for every file/hash pair in the request
    ApiResult::Ok {
        result: ScanFullResponse { proof_uuids },
    }
}
