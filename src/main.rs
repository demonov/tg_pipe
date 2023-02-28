use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().expect("Failed to read .env file");
    let db = env::var("db").expect("db not set");

    println!("db: {}", db);

}
