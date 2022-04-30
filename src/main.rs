use std::env;
use std::time::Duration;

use actix_web::{get, web, App, HttpServer, Responder};

use entity::post;
use entity::post::Entity as Post;
use entity::sea_orm::{self, ConnectOptions, Database, DatabaseConnection};
use migration::{Migrator, MigratorTrait};

#[derive(Debug, Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    // get env vars
    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    // establish connection to database and apply migrations
    // -> create post table if not exists
    let mut opt = ConnectOptions::new(db_url.to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    let conn = Database::connect(opt).await.unwrap();

    Migrator::up(&conn, None).await.unwrap();

    let state = AppState { conn };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/hello", web::get().to(|| async { "Hello World!" }))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
