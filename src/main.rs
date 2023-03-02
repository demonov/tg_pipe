use dotenv::dotenv;
use std::env;
use sqlx::{sqlite::SqlitePool, Pool, query};


#[derive(Debug, sqlx::FromRow)]
struct Person {
    id: i32,
    name: String,
    age: i32,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().expect("Failed to read .env file");
    let db = env::var("db").expect("db not set");
    let url = format!("sqlite:{}", db);

    env::set_var("DATABASE_URL", url);
    let pool = SqlitePool::connect(&url).await?;

    // create a table
    sqlx::query("CREATE TABLE IF NOT EXISTS people (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)").execute(&pool).await?;

    // insert some data
    sqlx::query("INSERT INTO people (name, age) VALUES (?, ?)")
        .bind("Alice")
        .bind(25)
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO people (name, age) VALUES (?, ?)")
        .bind("Bob")
        .bind(30)
        .execute(&pool)
        .await?;

    // query the data
    let people: Vec<Person> = query!("SELECT id, name, age FROM people")
        .fetch_all(&pool)
        .await?;

    println!("{:?}", people);

    Ok(())
}
