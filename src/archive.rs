use std::{env::var, fs::{self, remove_file, File}, io::Write, time::SystemTime};

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, Responder};
use futures_util::{StreamExt, TryStreamExt};
use iars::{Credentials, Item};
use serde_json::json;
use supabase_rs::SupabaseClient;

use crate::render_page;

#[actix_web::post("/upload")]
pub async fn upload(supabase: web::Data<SupabaseClient>, mut payload: Multipart, req: HttpRequest) -> impl Responder {
    let mut filen = String::new();
    let mut filep = String::new();
    let mut track_title = String::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition().unwrap();
        let name = content_disposition.get_name().unwrap();

        if name == "title" {
            let mut text = Vec::new();
            while let Some(chunk) = field.next().await {
                text.extend_from_slice(&chunk.unwrap());
            }
            let title = String::from_utf8(text).unwrap();
            track_title = title;
        } else if name == "file" {
            let filename = content_disposition
                .get_filename()
                .map(|f| sanitize_filename::sanitize(f))
                .unwrap_or("upload.bin".into());
    
            let filepath = format!("/tmp/{}", filename);
            filen = filename.clone();
            filep = filepath.clone();
            let mut f = File::create(filepath).unwrap();
    
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                f.write_all(&data).unwrap();
            }
        }
    }

    let creds = Credentials::new(var("ACCESS_KEY").unwrap().as_str(), var("PRIVATE_KEY").unwrap().as_str());

    let track_query = supabase
        .select("Tracks")
        .eq("title", track_title.as_str())
        .execute().await;

    if let Ok(q) = track_query {
        if !q.is_empty() {
            remove_file(filep).unwrap();
            let mut ctx = tera::Context::new();
            ctx.insert("status", "échouée");
            ctx.insert("msg", "ce titre est déjà utilisé");
            ctx.insert("btn_msg", "réessayer");
            ctx.insert("url", "/upload");

            return render_page("upload_res.html", ctx);
        }
    }

    let user_query = supabase
        .select("Users")
        .eq("id", req.cookie("user_id").unwrap().value())
        .execute().await;

    if let Ok(q) = user_query {
        let username = q.first().unwrap()["username"].as_str().unwrap();
        let mut user_tracks = q.first().unwrap()["tracks"].clone();

        let file = fs::read(format!("/tmp/{}", filen)).unwrap();

        let archive_name = format!("{}-{}{}", &filen[..filen.len()-4], SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), &filen[filen.len()-4..]);


        let item = Item::new("spotifuck").with_credentials(Some(creds));

        item.upload_file(false, &[("collection", "opensource_audio"), ("mediatype", "audio")], archive_name.as_str(), &file).unwrap();

        remove_file(filep).unwrap();

        let url = format!("https://archive.org/download/spotifuck/{}", archive_name);
        let extension = &filen[filen.len().saturating_sub(4)..];
        let form = if extension == ".wav" { "wav" } else { "mpeg" };

        let _insert = supabase.insert_without_defined_key("Tracks", json!({
            "title": track_title,
            "artist": username,
            "url": url,
            "format": form
        })).await.unwrap();

        if let Some(tracks) = user_tracks["tracks"].as_array_mut() {
            tracks.push(json!({
                "title": track_title
            }));
        }

        let _update = supabase.update_with_column_name("Users", "username", username, json!({
            "tracks": user_tracks
        })).await.unwrap();
    }

    let mut ctx = tera::Context::new();
    ctx.insert("status", "réussie");
    ctx.insert("msg", "votre publication mettra un certain temps avant d'être disponible");
    ctx.insert("btn_msg", "page d'accueil");
    ctx.insert("url", "/");

    render_page("upload_res.html", ctx)
}