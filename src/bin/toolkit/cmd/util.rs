use std::path::Path;

use function_name::named;
use scoretracker::{info, library::cache::compute_sha256_hash_of_file, log_fn_name, success};

use crate::toolkit::error::CmdError;

#[named]
pub fn file_sha256(path: &Path) -> Result<(), CmdError> {
    log_fn_name!(auto);

    info!("computing...");
    let hash = compute_sha256_hash_of_file(&path);
    println!("{hash}");
    success!("done");

    Ok(())
}
