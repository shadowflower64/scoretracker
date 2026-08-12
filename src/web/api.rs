use crate::data::games::{gh3, in_falsus};
use crate::data::scoreboard::r#match::{CommonMatchInfo, MatchMetadata};
use crate::info;
use crate::log_fn_name;
use crate::util::timestamp::NsTimestamp;
use actix_web::{HttpRequest, HttpResponse, Responder, get, put, web};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ResponseGetList<T: Serialize> {
    Ok { items: Vec<T> },
    _Error,
}

fn make_get_list_response_ok<T: Serialize>(items: Vec<T>) -> impl Responder {
    let response = ResponseGetList::Ok { items };
    HttpResponse::Ok().body(serde_json::to_string(&response).expect("could not convert response to json"))
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ResponsePut<T: Serialize> {
    Ok { item: T },
    _Error,
}

fn make_put_response_ok<T: Serialize>(item: T) -> impl Responder {
    let response = ResponsePut::Ok { item };
    HttpResponse::Ok().body(serde_json::to_string(&response).expect("could not convert response to json"))
}

#[get("/api/match")]
pub async fn get_match_list(_req: HttpRequest) -> impl Responder {
    log_fn_name!("get_match_list");
    info!("received get request for match list");
    make_get_list_response_ok(vec![
        //
        Box::new(gh3::Match {
            common: CommonMatchInfo {
                uuid: Uuid::now_v7().into(),
                timestamp: NsTimestamp::now(),
                song_id: "example".to_string(),
                proof: Vec::new(),
                comment: None,
                metadata: MatchMetadata::new(),
            },
            mode: gh3::Mode::Quickplay,
            score: 123456,
            notes_hit: 2345,
            max_streak: 1234,
            game_version: Some("gh3 Nonexistent Build".to_string()),
        }),
        //
        Box::new(gh3::Match {
            common: CommonMatchInfo {
                uuid: Uuid::now_v7().into(),
                timestamp: NsTimestamp::now(),
                song_id: "example2".to_string(),
                proof: Vec::new(),
                comment: None,
                metadata: MatchMetadata::new(),
            },
            mode: gh3::Mode::Quickplay,
            score: 123456,
            notes_hit: 2345,
            max_streak: 1234,
            game_version: Some("gh3 Nonexistent Build".to_string()),
        }),
        //
    ])
}

#[put("/api/match/{uuid}")]
pub async fn put_match(req: HttpRequest, req_body: web::Json<in_falsus::Match>) -> impl Responder {
    log_fn_name!("put_match");
    let uuid: Uuid = req
        .match_info()
        .get("uuid")
        .expect("uuid not provided")
        .parse()
        .expect("invalid uuid");
    let data = req_body.0;
    info!("received put request for match with uuid {uuid} -- {data:?}");
    make_put_response_ok(data)
}
