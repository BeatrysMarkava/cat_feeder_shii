use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

async fn health_handler() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
    })
}

async fn index() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Potugy 01 Backend API",
        "endpoints": vec![
            "/health",
            "/api/devices",
            "/api/users",
        ]
    }))
}

pub async fn start_backend_server() {
    let port = 8000;
    println!("Starting backend server on http://localhost:{}", port);

    let server = HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health_handler))
            .configure(configure_api_routes)
    })
    .bind(format!("127.0.0.1:{}", port))
    .expect("Failed to bind server")
    .run()
    .await;

    if let Err(e) = server {
        eprintln!("Server error: {}", e);
    }
}

fn configure_api_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/devices", web::get().to(get_devices))
            .route("/users", web::get().to(get_users)),
    );
}

async fn get_devices() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "devices": []
    }))
}

async fn get_users() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "users": []
    }))
}
