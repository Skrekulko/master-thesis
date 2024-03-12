use num_traits::Pow;
use thesislib::{
    handler, ieee754::{ieee754_extract_exponent, ieee754_extract_mantissa, ieee754_extract_sign, u32_to_ieee754_2nd}, sqrt::{find_m_recursive, fsqrt, isqrt}, structs::AppState
};

use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use actix_cors::Cors;
use actix_web::{
    middleware::Logger,
    http::header,
    web,
    App,
    HttpServer,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let numb: u32 = 36;
    // let m: u32 = find_m_recursive(numb, 0);
    // let s: u32 = isqrt(numb, m);
    // println!("x = {}, 2^(2 * m = {}) = {}, s = {}", numb, m, 2.pow(2 * m), s);

    // let ieee: u32 = u32_to_ieee754_2nd(numb);
    // let sign: u32 = ieee754_extract_sign(ieee);
    // let expo: u32 = ieee754_extract_exponent(ieee);
    // let mant: u32 = ieee754_extract_mantissa(ieee);
    

    // println!("numb: [u32] {: ^20} [x] {:#018x} [b] {:064b}", numb, numb, numb);
    // println!("ieee: [u32] {: ^20} [x] {:#018x} [b] {:064b}", ieee, ieee, ieee);
    // println!("sign: [u32] {: ^20} [x] {:#018x} [b] {:064b}", sign, sign, sign);
    // println!("expo: [u32] {: ^20} [x] {:#018x} [b] {:064b}", expo, expo, expo);
    // println!("mant: [u32] {: ^20} [x] {:#018x} [b] {:064b}", mant, mant, mant);

    // let sqrt: u64 = fsqrt(23 as u64, expo as u64, mant as u64);
    // println!("sqrt: [u64] {: ^20} [x] {:#018x} [b] {:064b}", sqrt, sqrt, sqrt);

    println!("Starting server");

    // Check if RUST_LOG environment variable is unset; set to "actix_web=info" if unset
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    // Configuration Setup: Load environment variables from a .env file, if present, and initialize the logger
    dotenv().ok();
    env_logger::init();

    // Attempt to establish a PostgreSQL database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    println!("Server started successfull");

    // Fetch the port from the environment variable
    let bind_port: u16 = std::env::var("CORE_PORT")
    .expect("CORE_PORT environment variable must be set")
    .parse()
    .expect("Invalid port number");

    // Configure and run Actix Web server
    HttpServer::new(move || {
        let cors = Cors::default()
            // .allowed_origin("http://localhost:3000") // TODO: Allow origin only from proxy?
            .allow_any_origin()
            .allowed_methods(vec![
                "GET",
                "POST",
                "DELETE"
            ])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials(); // TODO: what, how?
        App::new()
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .configure(handler::config)
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", bind_port))?
    .run()
    .await
}
