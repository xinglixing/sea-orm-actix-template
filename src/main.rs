use std::env;
use std::time::Duration;

use actix_web::{
    get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};

use entity::post;
use entity::post::Entity as Post;
use entity::sea_orm::{
    self, ActiveModelTrait, ConnectOptions, Database, DatabaseConnection, DeleteResult,
    EntityTrait, PaginatorTrait, QueryOrder, Set,
};
use migration::{Migrator, MigratorTrait};
use serde::{Deserialize, Serialize};

const DEFAULT_POSTS_PER_PAGE: usize = 25;

#[derive(Debug, Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<usize>,
    posts_per_page: Option<usize>,
}

#[get("/")]
async fn list(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let conn = &data.conn;

    // get params
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);
    let paginator = post::Entity::find()
        .order_by_asc(post::Column::Id)
        .paginate(conn, posts_per_page);

    // let num_pages = paginator.num_pages().await.ok().unwrap();

    let posts = paginator
        .fetch_page(page - 1)
        .await
        .expect("could not retrieve posts");
    Ok(HttpResponse::Ok().json(posts))
}

#[post("/")]
async fn create(
    data: web::Data<AppState>,
    post_data: web::Json<post::Model>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let post = post_data.into_inner();

    println!("{:?}", &post);

    let data = post::ActiveModel {
        title: Set(post.title.to_owned()),
        text: Set(post.text.to_owned()),
        ..Default::default()
    };

    let data = data.insert(conn).await.expect("could not insert post");

    Ok(HttpResponse::Created().json(data))
}

#[post("/{id}")]
async fn update(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    post_data: web::Json<post::Model>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let data = post_data.into_inner();

    let data = post::ActiveModel {
        id: Set(id.into_inner()),
        title: Set(data.title.to_owned()),
        text: Set(data.text.to_owned()),
    };

    let data = data.update(conn).await.expect("could not edit post");

    Ok(HttpResponse::Ok().json(data))
}

#[post("/delete/{id}")]
async fn delete(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let conn = &data.conn;

    Post::delete_by_id(id.into_inner())
        .exec(conn)
        .await
        .expect("could not delete post");

    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // get env vars
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");
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
            .service(create)
            .service(list)
            .service(update)
            .service(delete)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
