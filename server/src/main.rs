mod api;
mod entities;
mod routes;

use actix_embed::Embed;
use actix_web::{
  middleware::{Compress, Logger, NormalizePath},
  web::scope,
  web::Data,
  App, HttpResponse, HttpServer,
};
use api::PaypalClient;
use log::info;
use migration::{Migrator, MigratorTrait};
use routes::v1;
use rust_embed::RustEmbed;
use sea_orm::{Database, DatabaseConnection};
use std::error::Error;

#[derive(Debug, Clone)]
struct AppState {
  conn: DatabaseConnection,
  paypal_client: PaypalClient,
}

async fn connect_database() -> Result<DatabaseConnection, Box<dyn Error>> {
  let database_url = std::env::var("DATABASE_URL")?;
  let conn = Database::connect(&database_url)
    .await
    .expect("Failed to connect to database");
  info!("Database connected, starting migration...");

  Migrator::up(&conn, None)
    .await
    .expect("Failed to migrate database");

  Ok(conn)
}

#[derive(RustEmbed)]
#[folder = "../client/dist"]
struct Static;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
  dotenv::dotenv().ok();
  pretty_env_logger::init();
  let (paypal_client, conn) = (PaypalClient::new()?, connect_database().await?);
  let state = AppState {
    conn,
    paypal_client,
  };
  let server_pms = HttpServer::new(move || {
    App::new()
      .wrap(NormalizePath::trim())
      .wrap(Compress::default())
      .service(
        scope("/api")
          .app_data(Data::new(state.clone()))
          .wrap(Logger::default())
          .service(v1()),
      )
      .service(
        Embed::new("/", &Static)
          .index_file("index.html")
          .fallback_handler(|_: &_| {
            HttpResponse::Ok().body(Static::get("index.html").unwrap().data)
          }),
      )
  })
  .bind(("localhost", 8080))?
  .run();
  info!("Starting server at http://localhost:8080/");
  server_pms.await?;
  Ok(())
}
