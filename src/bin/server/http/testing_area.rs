// #[get("/openapi.json")]
// #[named]
// pub async fn openapi_doc() -> impl Responder {
//     log_fn_name!(auto);
//     log_should_print_debug!(true);
//     debug!("requested openapi doc");
//     static API_DOC_JSON: LazyLock<String> = LazyLock::new(|| {
//         debug!("documenting the api...");
//         match ApiDoc::openapi().to_pretty_json() {
//             Ok(json) => json,
//             Err(e) => format!("error: {e}"),
//         }
//     });
//     HttpResponse::Ok().body(API_DOC_JSON.clone())
// }

use actix_web::{HttpResponse, Responder, get, post};

#[post("/test/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(String::from("Echoed: ") + &req_body)
}

#[get("/test/hey")]
pub async fn hey() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
