use crate::data::games::in_falsus;
use actix_web::{HttpRequest, HttpResponse, Responder, put, web};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ResponsePut<T: Serialize> {
    Ok { item: T },
    Error,
}

fn put_ok<T: Serialize>(item: T) -> impl Responder {
    let response = ResponsePut::Ok { item };
    HttpResponse::Ok().body(serde_json::to_string(&response).expect("could not convert response to json"))
}

#[put("/api/match/{uuid}")]
pub async fn put_match(req: HttpRequest, req_body: web::Json<in_falsus::Match>) -> impl Responder {
    let uuid: Uuid = req
        .match_info()
        .get("uuid")
        .expect("uuid not provided")
        .parse()
        .expect("invalid uuid");
    let data = req_body.0;
    println!("received put request for match with uuid {uuid} -- {data:?}");
    put_ok(data)
}
