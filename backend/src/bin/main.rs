use num_traits::Pow;
use thesislib::{
    handler, ieee754::{fheint32_to_ieee754, ieee754_extract_exponent, ieee754_extract_mantissa, ieee754_extract_sign, u32_to_ieee754_2nd}, sqrt::{find_m_recursive, fsqrt, isqrt, isqrt_homo}, structs::AppState
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

use tfhe::{prelude::*, FheBool};
use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheInt32, FheInt64};

#[actix_web::main] // TODO: uncomment
async fn main() -> std::io::Result<()> { // TODO: uncomment
// fn main() {
    // let config = ConfigBuilder::default().build();
    // let (client_key, server_key) = generate_keys(config);
    // set_server_key(server_key);

    // // CONSTANTS
    // const IEEE754_MANTISSA_SIZE: u64 = 23;

    // // Number
    // let radicand:i32 = 25;
    // let radicand_encrypted = FheInt32::encrypt(radicand, &client_key);

    // // IEEE 754
    // let ieee754: FheInt64 = fheint32_to_ieee754(&radicand_encrypted).cast_into();
    // let exponent: FheInt64 = (ieee754.clone() >> IEEE754_MANTISSA_SIZE) - 127;
    // let mantissa: FheInt64 = ieee754.clone() & 0x7FFFFF;

    // let i754: i64 = ieee754.decrypt(&client_key);
    // println!("i754: [i64] {: ^20} [x] {:#018x} [b] {:064b}", i754, i754, i754);

    // // Calculate stuff
    // let common_one: FheInt64 = (ieee754.clone() | 0x80000000) >> 31u64;
    // let hidden_bit: FheInt64 = common_one.clone() << IEEE754_MANTISSA_SIZE;
    // let normalized_mantissa: FheInt64 = mantissa.clone() | hidden_bit.clone();
    // let big_n: FheInt64 = (exponent.clone() % 2).eq(&common_one).if_then_else(
    //     &(normalized_mantissa.clone() << (IEEE754_MANTISSA_SIZE + 1)),
    //     &(normalized_mantissa.clone() << IEEE754_MANTISSA_SIZE)
    // );
    // let new_exponent: FheInt64 = (exponent.clone() % 2).eq(&common_one).if_then_else(
    //     &(exponent.clone() - 1),
    //     &exponent.clone()
    // );
    // // let first_root: FheInt64 = FheInt64::encrypt(10485760, &client_key);
    // let first_root: FheInt64 = isqrt_homo(&big_n.clone().cast_into());
    // let base_exponent: FheInt64 = (127 + (new_exponent.clone() >> 1u64)) << IEEE754_MANTISSA_SIZE;
    // let root: FheInt64 = base_exponent | first_root;

    // let nmts: i64 = normalized_mantissa.decrypt(&client_key);
    // let bign: i64 = big_n.decrypt(&client_key);
    // let newe: i64 = new_exponent.decrypt(&client_key);
    // let rotd: i64 = root.decrypt(&client_key);
    // println!("nmts: [i64] {: ^20} [x] {:#018x} [b] {:064b}", nmts, nmts, nmts);
    // println!("bign: [i64] {: ^20} [x] {:#018x} [b] {:064b}", bign, bign, bign);
    // println!("newe: [i64] {: ^20} [x] {:#018x} [b] {:064b}", newe, newe, newe);
    // println!("rotd: [i64] {: ^20} [x] {:#018x} [b] {:064b}", rotd, rotd, rotd);
    
    // println!("root: [u32] {: ^20} [x] {:#018x} [b] {:064b}", root_decrypted, root_decrypted, root_decrypted);


    // TODO: Uncomment
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

    // Fetch the port from the environment variable or use default
    let bind_port: u16 = std::env::var("CORE_PORT")
    .unwrap_or("12345".to_string())
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
