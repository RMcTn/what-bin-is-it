#![allow(clippy::needless_return)]

use std::{dbg, env};
use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sesv2::Client;
use axum::extract::State;
use axum::Form;
use axum::response::Html;
use axum::Router;
use axum::routing::get;
use axum_macros::debug_handler;
use clokwerk::AsyncScheduler;
use clokwerk::Job;
use log::error;
use log::info;
use serde::Deserialize;
use sqlx::Row;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::sqlite::SqliteRow;
use sqlx::SqlitePool;

use bin_stuff::next_bin_collection_date;
use bin_stuff::User;

use crate::email_sender::email_user;

pub mod email_sender;

// TODO: Auth for pages (Or just remove it for now?)

// TODO:  Some gotchas that need solved:
//  TODO: Not all house addresses are the same as what the site provides.
//      I.e someone could be in a named house but that still comes up at 5 Madeup Lane.
//      Could ask user to input the address they would put in the site
//  TODO: Not all houses have all bin access. I.e, some houses only have the general waste bin collection
//  TODO: Not all bin collection dates will be the same day. I.e, not all bin collections are on a Monday. Need the user to specify their collection date (or scrape it from the site again) TODO: Are bin collections the same for an entire postcode? Could be an opportunity for caching per postcode, but need to verify that assumption
//

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
    aws_client: Client,
    from_email_address: String,
    geckodriver_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::init_from_env(env);

    let from_email_address =
        env::var("FROM_EMAIL_ADDRESS").expect("FROM_EMAIL_ADDRESS must be specified");
    let _aws_access_key_id =
        env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID must be specified");
    let _aws_secret_access_key =
        env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY must be specified");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be specified");
    let geckodriver_url_default = "http://127.0.0.1:4444".to_string();
    let geckodriver_url = match env::var("GECKODRIVER_URL") {
        Ok(url) => url,
        Err(_) => {
            info!(
                "GECKODRIVER_URL was not specified. Defaulting to {}",
                geckodriver_url_default
            );
            geckodriver_url_default
        }
    };
    let region_provider = RegionProviderChain::default_provider().or_else("eu-west-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let aws_client = Client::new(&config);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    sqlx::migrate!("../migrations").run(&pool).await?;

    let people_to_notify = get_all_users(&pool).await?;
    for person in &people_to_notify {
        println!("Found {:?}", person);
    }

    let app_state = AppState {
        pool,
        aws_client,
        from_email_address,
        geckodriver_url,
    };
    let scheduler_app_state = app_state.clone();

    let app = Router::new()
        .route("/", get(root_page))
        // .route("/create_user", get(show_create_user_form).post(submit_user_form))
        // .route("/users", get(show_all_users_page))
        .with_state(app_state);

    let mut scheduler = AsyncScheduler::new();

    scheduler
        .every(clokwerk::Interval::Sunday)
        .at("7:00 pm")
        .run(move || scrape_and_email_stuff(scheduler_app_state.clone()));

    let mut scheduler_poll_interval = tokio::time::interval(Duration::from_secs(60));
    tokio::spawn(async move {
        loop {
            scheduler_poll_interval.tick().await;
            info!("Running any pending scheduled jobs");
            scheduler.run_pending().await;
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    info!("Listening on {}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    return Ok(());
}

async fn scrape_and_email_stuff(app_state: AppState) {
    info!("Running email stuff now");
    let people_to_notify = get_all_users(&app_state.pool).await.unwrap();
    for user in &people_to_notify {
        info!("Beginning scraping for {}", user.email);
        // TODO: Store the scraped date somewhere?
        //  What's the use? lets us separate emails i guess
        let bins = scraper::get_stuff(
            &user.postcode,
            &user.address,
            Some(app_state.geckodriver_url.clone()),
        )
            .await
            .unwrap();
        let today = chrono::Utc::now().date_naive();
        let next_bin_collection = next_bin_collection_date(&bins, today);
        info!("Beginning emailing for {}", user.email);
        // TODO: Keep track of users that have successfully been sent an email so a retry doesn't
        // happen unexpectedly
        if let Err(e) = email_user(
            user,
            &next_bin_collection,
            &app_state.aws_client,
            &app_state.from_email_address,
        )
            .await
        {
            error!("{}", e);
        }
    }
}

#[debug_handler]
async fn submit_user_form(
    State(app_state): State<AppState>,
    Form(input): Form<CreateUser>,
) -> String {
    let pool = app_state.pool;
    dbg!(&input);
    let user = create_user(&pool, input).await.unwrap();
    dbg!(&user);
    // TODO: Redirect to users page
    return "Wow".to_string();
}

async fn create_user(pool: &SqlitePool, input: CreateUser) -> Result<User, Box<dyn Error>> {
    let id = sqlx::query("INSERT INTO emails (email, postcode, address) VALUES (?1, ?2, ?3)")
        .bind(&input.email)
        .bind(&input.postcode)
        .bind(&input.address)
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

async fn get_all_users(pool: &SqlitePool) -> Result<Vec<User>, Box<dyn Error>> {
    // TODO: Paging at some point
    let users = sqlx::query("SELECT id, email, postcode, address FROM emails")
        .map(|row: SqliteRow| User {
            _id: row.get("id"),
            email: row.get("email"),
            postcode: row.get("postcode"),
            address: row.get("address"),
        })
        .fetch_all(pool)
        .await?;

    return Ok(users);
}

async fn show_all_users_page() -> Html<&'static str> {
    Html("Nothing yet")
}

async fn root_page() -> Html<&'static str> {
    Html("Nothing yet")
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
