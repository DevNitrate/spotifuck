use actix_web::{web, HttpRequest, HttpResponse, Responder};
use supabase_rs::SupabaseClient;

use crate::{auth::is_user_logged_in, render_page};

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
async fn signin() -> impl Responder {
    let ctx = tera::Context::new();
    render_page("signin.html", ctx)
}

#[actix_web::get("/login")]
async fn loginp() -> impl Responder {
    let ctx = tera::Context::new();
    render_page("login.html", ctx)
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

#[actix_web::get("/test")]
async fn test() -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("status", "réussie");
    ctx.insert("msg", "votre publication a réussie");
    ctx.insert("btn_msg", "page d'accueil");
    ctx.insert("url", "/");

    render_page("upload_res.html", ctx)
}