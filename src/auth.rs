use actix_web::{cookie::{time::Duration, Cookie}, web, HttpRequest, HttpResponse, Responder};
use serde_json::{json, Value};
use supabase_rs::SupabaseClient;
use bcrypt::{hash, verify, DEFAULT_COST};

pub struct User {
    username: String,
    pswd: String,
    playlist: Value,
    tracks: Value
}

#[actix_web::post("/auth/create/{username}/{pswd}")]
async fn create_user(supabase: web::Data<SupabaseClient>, path: web::Path<(String, String)>) -> impl Responder {
    let (username, pswd) = path.into_inner();

    let query = supabase.
        select("Users")
        .columns(vec!["username"])
        .eq("username", username.as_str())
        .execute().await;

    match query {
        Ok(q) => {
            if q.is_empty() {
                let user: User = User {
                    username,
                    pswd: hash(pswd, DEFAULT_COST).unwrap(),
                    playlist: json!({
                        "tracks": []
                    }),
                    tracks: json!({
                        "tracks": []
                    })
                };

                let insert = supabase.insert_without_defined_key("Users", json!({
                    "username": user.username,
                    "pswd": user.pswd,
                    "playlist": user.playlist,
                    "tracks": user.tracks
                })).await;

                match insert {
                    Ok(_) => HttpResponse::Ok().json(json!({
                        "exists": false
                    })),
                    Err(e) => HttpResponse::InternalServerError().body(e)
                }
            } else {
                HttpResponse::Ok().json(json!({
                    "exists": true
                }))
            }
        },
        Err(e) => HttpResponse::InternalServerError().body(e)
    }
}

#[actix_web::post("/auth/login/{username}/{pswd}")]
async fn login(supabase: web::Data<SupabaseClient>, path: web::Path<(String, String)>) -> impl Responder {
    let (username, pswd) = path.into_inner();

    let query = supabase
        .select("Users")
        .eq("username", username.as_str())
        .execute().await;

    match query {
        Ok(q) => {
            if q.is_empty() {
                return HttpResponse::Ok().json(json!({
                    "error": "invalid username"
                }));
            }

            let hashed = q.first().unwrap()["pswd"].as_str().unwrap();
            let uuid = q.first().unwrap()["id"].as_str().unwrap();

            if verify(pswd, hashed).unwrap() {
                let cookie = Cookie::build("user_id", uuid).path("/").finish();

                return HttpResponse::Ok().cookie(cookie).json(json!({
                    "error": "none"
                }));
            } else {
                return HttpResponse::Ok().json(json!({
                        "error": "invalid password"
                }));
            }
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": e
            }));
        }
    }
    
}

#[actix_web::post("/auth/logout")]
async fn logout() -> impl Responder {
    let cookie = Cookie::build("user_id", "").path("/").max_age(Duration::ZERO).finish();

    HttpResponse::Ok().cookie(cookie).body("logged out")
}

#[actix_web::post("/auth/delete")]
async fn delete(supabase: web::Data<SupabaseClient>, req: HttpRequest) -> impl Responder {
    let uuid = req.cookie("user_id").unwrap().value().to_string();

    let _delete = supabase.delete("Users", &uuid).await.unwrap();

    let cookie = Cookie::build("user_id", "").path("/").max_age(Duration::ZERO).finish();

    HttpResponse::Ok().cookie(cookie).body("account deleted")
}

pub async fn is_user_logged_in(supabase: &web::Data<SupabaseClient>, req: &HttpRequest) -> bool {
    if let Some(cookie) = req.cookie("user_id") {
        let query = supabase
            .select("Users")
            .eq("id", cookie.value())
            .execute().await;

        if let Ok(q) = query {
            if !q.is_empty() {
                true
            } else {
                false
            }
        } else {
            false
        }


    } else {
        false
    }
}

pub async fn get_playlist(supabase: &web::Data<SupabaseClient>, req: &HttpRequest) -> Vec<String> {
    let playlist_query = supabase
        .select("Users")
        .eq("id", req.cookie("user_id").unwrap().value())
        .execute().await.unwrap();

    let playlist = playlist_query.first().unwrap()["playlist"].clone();

    let titles_quote: Vec<String> = playlist["tracks"].as_array().unwrap()
        .iter()
        .filter_map(|track| Some(track.get("title").unwrap().to_string()))
        .collect();

    let titles = titles_quote.iter().map(|s| s[1..s.len()-1].to_string()).collect();

    titles
}