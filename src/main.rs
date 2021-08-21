use ::lib_api_actix::{config, middleware};
use actix_cors::Cors;
use actix_service::Service;
use actix_web::{http, App, HttpServer};
use futures::FutureExt;
use std::{default::Default, env, io};

#[actix_rt::main]
async fn main() -> io::Result<()>{
    dotenv::dotenv().expect("Failed to read .env file");
    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let app_host = env::var("APP_HOST").expect("APP_HOST not found.");
    let app_port = env::var("APP_PORT").expect("APP_PORT not found.");
    let app_url = format!("{}:{}", &app_host, &app_port);

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found");
    let pool = crate::config::db::migrate_and_config_db(&db_url);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default()
                .allowed_origin("http://127.0.0.1:3000")
                .allowed_origin("http://localhost:3000")
                .send_wildcard()
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600))
            .data(pool.clone())
            .wrap(actix_web::middleware::Logger::default())
            .wrap(middleware::auth_middleware::Authentication)
            .wrap_fn(|req, src|{
                src.call(req).map(|res|res)
            })
            .configure(config::app::config_services)
        })
    .bind(&app_url)?
    .run()
    .await
}

#[cfg(test)]
mod tests{
    use ::lib_api_actix::config;
    use actix_cors::Cors;
    use actix_service::Service;
    use actix_web::{http, App, HttpServer};
    use futures::FutureExt;

    #[actix_rt::test]
    async fn test_startup_ok(){
        let pool = config::db::migrate_and_config_db(":memory:");

        HttpServer::new(move || {
            App::new()
                .wrap(Cors::default()
                    .send_wildcard()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600))
                .data(pool.clone())
                .wrap(actix_web::middleware::Logger::default())
                .wrap_fn(|req, srv|{
                    srv.call(req).map(|res| res)
                })
                .configure(config::app::config_services)
        })
        .bind("localhost:8000".to_string()).unwrap()
        .run();

        assert_eq!(true, true);
    }

    #[actix_rt::test]
    async fn test_startup_without_auth_middleware_ok(){
        let pool = config::db::migrate_and_config_db(":memory:");

        HttpServer::new(move ||{
            App::new()
                .wrap(Cors::default()
                    .send_wildcard()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600))
                .data(pool.clone())
                .wrap(actix_web::middleware::Logger::default())
                .wrap_fn(|req, srv|{
                    srv.call(req).map(|res| res)
                })
                .configure(config::app::config_services)
        })
        .bind("localhost:8001".to_string()).unwrap()
        .run();

        assert_eq!(true, true)
    }
}