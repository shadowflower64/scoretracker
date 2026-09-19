use actix_files::NamedFile;
use actix_web::{Error, get};

use crate::server::http::start::web_frontend_dir_path;

#[get("/")]
async fn index_handler() -> Result<NamedFile, Error> {
    Ok(NamedFile::open(web_frontend_dir_path().join("app.html"))?)
}
