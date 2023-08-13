#![allow(clippy::needless_return)]

use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::Form;
use axum::Router;
use axum_macros::debug_handler;
use serde::Deserialize;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::error::Error;
use std::net::SocketAddr;
use std::{dbg, env};

use bin_stuff::User;

// TODO would be nice to have an admin page that for adding new users
// TODO:  Some gotchas that need solved:
//  TODO: Not all house addresses are the same as what the site provides.
//      I.e someone could be in a named house but that still comes up at 5 Madeup Lane.
//      Could ask user to input the address they would put in the site
//  TODO: Not all houses have all bin access. I.e, some houses only have the general waste bin collection
//  TODO: Not all bin collection dates will be the same day. I.e, not all bin collections are on a Monday. Need the user to specify their collection date (or scrape it from the site again) TODO: Are bin collections the same for an entire postcode? Could be an opportunity for caching per postcode, but need to verify that assumption

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be specified");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS emails (
            id          INTEGER PRIMARY KEY,
            email       TEXT NOT NULL,
            postcode    TEXT NOT NULL,
            address     TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS EmailsUniqueIndexOnEmails ON emails (email)")
        .execute(&pool)
        .await?;

    let records = sqlx::query!("SELECT id, email, postcode, address FROM emails")
        .fetch_all(&pool)
        .await?;
    let mut people_to_notify = vec![];
    for record in records {
        let user = User {
            _id: record.id,
            email: record.email,
            postcode: record.postcode,
            address: record.address,
        };
        people_to_notify.push(user);
    }
    for person in &people_to_notify {
        println!("Found {:?}", person);
    }

    let app = Router::new()
        .route("/", get(show_create_user_form).post(submit_user_form))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    return Ok(());
}

#[debug_handler]
async fn submit_user_form(State(pool): State<SqlitePool>, Form(input): Form<CreateUser>) -> String {
    dbg!(&input);
    let user = create_user(&pool, input).await.unwrap();
    dbg!(&user);
    return "Wow".to_string();
}

async fn create_user(pool: &SqlitePool, input: CreateUser) -> Result<User, Box<dyn Error>> {
    let id = sqlx::query!(
        "INSERT INTO emails (email, postcode, address) VALUES (?1, ?2, ?3)",
        input.email,
        input.postcode,
        input.address
    )
    .execute(pool)
    .await?
    .last_insert_rowid();

    return Ok(User {
        _id: id,
        email: input.email,
        postcode: input.postcode,
        address: input.address,
    });
}

async fn show_create_user_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                
                    <form action="/" method="post" style="display:flex; flex-direction:column; flex-wrap: wrap">
                        <label for="email">
                            Enter the email:
                            <input type="text" name="email">
                        </label>

                        <label for="postcode">
                            Enter the postcode:
                            <input type="text" name="postcode">
                        </label>

                        <label for="address">
                            Enter the address:
                            <input type="text" name="address">
                        </label>

                        <input type="submit" value="Create user">
                    </form>
                </div>
            </body>
        </html>
        "#,
    )
}

#[derive(Deserialize, Debug)]
struct CreateUser {
    email: String,
    postcode: String,
    address: String,
}
