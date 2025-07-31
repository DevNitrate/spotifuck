use std::env::var;

use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use lazy_static::lazy_static;
use supabase_rs::SupabaseClient;
use tera::Tera;

use crate::{archive::upload, auth::{create_user, login, logout}, routes::{add_playlist, del_playlist, index, loginp, search, signin, uploadp}};

mod routes;
mod archive;
mod auth;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let src: &str = "templates/**/*";
        let tera: Tera = Tera::new(src).unwrap();
        tera
    };
}

fn render_page(name: &str, ctx: tera::Context) -> HttpResponse {
    match TEMPLATES.render(name, &ctx) {
        Ok(page) => {
            HttpResponse::Ok().body(page)
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Error loading page:\n\n{}", e))
        }
    }
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("Requested page not found")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = 8080;

    dotenv().ok();
    let supabase: SupabaseClient = SupabaseClient::new(var("SUPABASE_URL").unwrap(), var("SUPABASE_KEY").unwrap()).unwrap();
    let supabase_data = web::Data::new(supabase);

    HttpServer::new(move || {
        App::new()
            .default_service(web::to(not_found))
            .app_data(supabase_data.clone())
            .service(Files::new("/static", "./static"))
            .service(index).service(create_user).service(login).service(logout).service(signin).service(loginp).service(uploadp).service(upload).service(search).service(add_playlist).service(del_playlist)
    })
    .bind(("0.0.0.0", port)).unwrap()
    .run().await
}
