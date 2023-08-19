FROM rust:latest

WORKDIR /usr/src/what-bin-is-it

ADD Cargo.toml .
ADD Cargo.lock .
ADD bin_stuff bin_stuff
ADD email_sender email_sender
ADD server server
ADD scraper scraper
ADD .sqlx .sqlx

RUN cargo build --release
