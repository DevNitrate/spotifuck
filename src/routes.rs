use std::env::var;

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use supabase_rs::SupabaseClient;

use crate::{auth::{get_playlist, is_user_logged_in}, render_page};

pub fn redirect(path: &str) -> HttpResponse {
    HttpResponse::Found().append_header(("location", path)).finish()
}

#[actix_web::get("/")]
async fn index(supabase: web::Data<SupabaseClient>, req: HttpRequest) -> impl Responder {
    if is_user_logged_in(&supabase, &req).await {
        let ctx: tera::Context = tera::Context::new();
        render_page("index.html", ctx)
    } else {
        redirect("/login")
    }
}

#[actix_web::get("/signin")]
async fn signin(supabase: web::Data<SupabaseClient>, req: HttpRequest) -> impl Responder {
    if is_user_logged_in(&supabase, &req).await {
        redirect("/")
    } else {
        let ctx = tera::Context::new();
        render_page("signin.html", ctx)
    }
}

#[actix_web::get("/login")]
async fn loginp(supabase: web::Data<SupabaseClient>, req: HttpRequest) -> impl Responder {
    if is_user_logged_in(&supabase, &req).await {
        redirect("/")
    } else {
        let ctx = tera::Context::new();
        render_page("login.html", ctx)
    }
}

#[actix_web::get("/upload")]
async fn uploadp(supabase: web::Data<SupabaseClient>, req: HttpRequest) -> impl Responder {
    if is_user_logged_in(&supabase, &req).await {
        let ctx = tera::Context::new();
        render_page("upload.html", ctx)
    } else {
        redirect("/login")
    }
}

#[derive(Deserialize)]
struct TrackQuery {
    track: String
}

#[derive(Debug, Deserialize)]
struct Track {
    title: String,
    artist: String,
    url: String,
    format: String
}

#[derive(Deserialize, Serialize)]
struct TrackCtx {
    title: String,
    artist: String,
    url: String,
    format: String,
    playlist: String
}

#[actix_web::get("/search")]
async fn search(supabase: web::Data<SupabaseClient>, req: HttpRequest, track: web::Query<TrackQuery>) -> impl Responder {
    if is_user_logged_in(&supabase, &req).await {
        let track_name = track.into_inner().track;

        if !track_name.is_empty() {
            let client = reqwest::Client::new();
            let supabase_url = var("SUPABASE_URL").unwrap();
            let supabase_key = var("SUPABASE_KEY").unwrap();
    
            let track_query = client.get(format!("{}/rest/v1/{}", supabase_url, "Tracks"))
                .query(&[("title", format!("ilike.%{}%", track_name))])
                .header("apikey", &supabase_key)
                .header("Authorization", format!("Bearer {}", &supabase_key))
                .header("Accept", "application/json")
                .send().await.unwrap();
    
            let tracks: Vec<Track> = track_query.json().await.unwrap();
            let mut tracks_ctx: Vec<TrackCtx> = Vec::new();

            let playlist = get_playlist(&supabase, &req).await;
            
            for track in tracks {
                let title = track.title.clone();
                let in_playlist: &str;

                if playlist.contains(&title) {
                    in_playlist = "-";
                } else {
                    in_playlist = "+"
                }

                let track_ctx = TrackCtx {
                    title: track.title,
                    artist: track.artist,
                    url: track.url,
                    format: track.format,
                    playlist: in_playlist.to_string()
                };

                tracks_ctx.push(track_ctx);
            }
    
            let mut ctx = tera::Context::new();
            ctx.insert("tracks", &tracks_ctx);
            render_page("search.html", ctx)
        } else {
            let tracks: Vec<TrackCtx> = Vec::new();

            let mut ctx = tera::Context::new();
            ctx.insert("tracks", &tracks);
            render_page("search.html", ctx)
        }

    } else {
        redirect("/login")
    }
}

#[actix_web::get("/playlist")]
async fn playlistp(supabase: web::Data<SupabaseClient>, req: HttpRequest) -> impl Responder {
    if is_user_logged_in(&supabase, &req).await {
        let client = reqwest::Client::new();
        let supabase_url = var("SUPABASE_URL").unwrap();
        let supabase_key = var("SUPABASE_KEY").unwrap();

        let track_query = client.post(format!("{}/rest/v1/rpc/get_user_tracks", supabase_url))
            .header("apikey", &supabase_key)
            .header("Authorization", format!("Bearer {}", &supabase_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "user_id": req.cookie("user_id").unwrap().value()
            }))
            .send().await.unwrap();

        let tracks_json: serde_json::Value = track_query.json().await.unwrap();
        let mut tracks: Vec<TrackCtx> = Vec::new();

        for track_json in tracks_json.as_array().unwrap() {
            let title = track_json["title"].to_string();
            let artist = track_json["artist"].to_string();
            let url = track_json["url"].to_string();
            let format = track_json["format"].to_string();

            let track = TrackCtx {
                title: (&title[1..title.len()-1]).to_string(),
                artist: (&artist[1..artist.len()-1]).to_string(),
                url: (&url[1..url.len()-1]).to_string(),
                format: (&format[1..format.len()-1]).to_string(),
                playlist: "-".to_string()
            };

            tracks.push(track);
        }

        let mut ctx = tera::Context::new();
        ctx.insert("tracks", &tracks);
        render_page("playlist.html", ctx)
    } else {
        redirect("/login")
    }
}

#[actix_web::get("/settings")]
async fn settings(supabase: web::Data<SupabaseClient>, req: HttpRequest) -> impl Responder {
    if is_user_logged_in(&supabase, &req).await {
        let uuid = req.cookie("user_id").unwrap().value().to_string();

        let user_query = supabase
            .select("Users")
            .eq("id", &uuid)
            .execute().await.unwrap();

        let username = user_query.first().unwrap()["username"].as_str().unwrap();
        
        let mut ctx = tera::Context::new();
        ctx.insert("username", username);
        render_page("settings.html", ctx)
    } else {
        redirect("/login")
    }
}

#[actix_web::post("playlist/add/{track}")]
async fn add_playlist(supabase: web::Data<SupabaseClient>, path: web::Path<String>, req: HttpRequest) -> impl Responder {
    let track_title = path.into_inner();
    let uuid = req.cookie("user_id").unwrap().value().to_string();

    let playlist_query = supabase
        .select("Users")
        .eq("id", &uuid)
        .execute().await.unwrap();

    let mut playlist = playlist_query.first().unwrap()["playlist"].clone();

    if let Some(track) = playlist["tracks"].as_array_mut() {
        track.push(json!({
            "title": track_title
        }));
    }

    let _update = supabase.update_with_column_name("Users", "id", &uuid, json!({
        "playlist": playlist
    })).await.unwrap();

    HttpResponse::Ok()
}

#[actix_web::post("playlist/delete/{track}")]
async fn del_playlist(supabase: web::Data<SupabaseClient>, path: web::Path<String>, req: HttpRequest) -> impl Responder {
    let track_title = path.into_inner();
    let uuid = req.cookie("user_id").unwrap().value().to_string();

    let playlist_query = supabase
        .select("Users")
        .eq("id", &uuid)
        .execute().await.unwrap();

    let mut playlist = playlist_query.first().unwrap()["playlist"].clone();

    if let Some(track) = playlist["tracks"].as_array_mut() {
        track.retain(|item| item["title"] != track_title);
    }

    let _update = supabase.update_with_column_name("Users", "id", &uuid, json!({
        "playlist": playlist
    })).await.unwrap();

    HttpResponse::Ok()
}