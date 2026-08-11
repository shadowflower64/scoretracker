pub mod api;

use actix_files::NamedFile;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post};
use actix_web::{Error, HttpRequest};
use relative_path::{Component, RelativePathBuf};
use std::path::PathBuf;

use crate::util::relative_path_from_segments;
use crate::{debug, info, log_fn_name, log_should_print_debug};

pub const STATIC_FILE_PATH_SEGMENTS: &[&str] = &["web-frontend", "static"];
pub fn static_files_path() -> PathBuf {
    relative_path_from_segments(STATIC_FILE_PATH_SEGMENTS).to_path(".")
}

#[get("/")]
async fn index() -> Result<NamedFile, Error> {
    Ok(NamedFile::open(static_files_path().join("app.html"))?)
}

#[get("/static/{filename:.*}")]
async fn static_handler(req: HttpRequest) -> Result<NamedFile, Error> {
    log_fn_name!("static_handler");
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
    let fullpath = relpath.to_path(static_files_path());
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

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(String::from("echoed: ") + &req_body)
}

#[get("/hey")]
async fn hey() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
pub async fn web_main() -> std::io::Result<()> {
    log_fn_name!("web_main");
    const HOST: &str = "127.0.0.1";
    const PORT: u16 = 8080;
    info!("starting server on: http://{HOST}:{PORT}");
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(static_handler)
            .service(echo)
            .service(hey)
            .service(api::put_match)
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
