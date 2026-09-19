use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::{Error, HttpRequest, get};
use function_name::named;
use relative_path::{Component, RelativePathBuf};
use scoretracker::{debug, log_fn_name, log_should_print_debug, util::relative_path_from_segments};

use crate::server::http::start::WEB_FRONTEND_DIR_PATH_SEGMENTS;

pub fn static_file_dir_path() -> PathBuf {
    relative_path_from_segments(WEB_FRONTEND_DIR_PATH_SEGMENTS).join("app").to_path(".")
}

#[get("/app/{filename:.*}")]
#[named]
async fn static_handler(req: HttpRequest) -> Result<NamedFile, Error> {
    log_fn_name!(auto);
    log_should_print_debug!(true);

    let path_str = req.match_info().query("filename");
    let relpath = RelativePathBuf::from(path_str);
    for component in relpath.components() {
        match component {
            Component::CurDir => panic!("Invalid path"),    // TODO: cleaner error?
            Component::ParentDir => panic!("Invalid path"), // TODO: cleaner error?
            Component::Normal(_) => {}
        }
    }
    let fullpath = relpath.to_path(static_file_dir_path());
    debug!("requested path: {relpath:?} -> {fullpath:?}");
    let file = NamedFile::open(fullpath)?;
    Ok(
        file.use_last_modified(true),
        /* .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }) */
    )
}
