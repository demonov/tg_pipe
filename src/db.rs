use std::env;
use std::error::Error;
use sqlx::{Pool, sqlite::Sqlite, SqlitePool};

#[derive(Debug, sqlx::FromRow)]
struct Chat {
    id: i32,
    name: String,
}

#[derive(Debug, sqlx::FromRow)]
struct Message {
    id: i32,
    text: String,
    chat_id: i32,
    user_id: i32,
}

pub struct Db {
    pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(url: String) -> Result<Self, Box<dyn Error>> {
        let url = format!("sqlite://{}", url);
        env::set_var("DATABASE_URL", &url);
        let pool = SqlitePool::connect(&url).await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), Box<dyn Error>> {
        sqlx::query("CREATE TABLE IF NOT EXISTS chats (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages \
                (id INTEGER PRIMARY KEY, text TEXT, chat_id INTEGER, user_id INTEGER)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

/*


// create a table


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
let mut people = sqlx::query("SELECT id, name, age FROM people")
    .fetch(&pool);

// while let Some(row) = people.try_next().await? {
//     let person = Person {
//         id: row.get(0),
//         name: row.get(1),
//         age: row.get(2),
//     };
//
//     println!("Found person {:?}", person);
// }

let mut stream = sqlx::query_as::<_, Person>("SELECT id, name, age FROM people")
    .fetch(&pool);

use futures_util::TryStreamExt;
while let Some(person) = stream.try_next().await? {
    println!("Found person {:?}", person);
}
*/
