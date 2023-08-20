FROM rust:latest

WORKDIR /usr/src/what-bin-is-it

ADD Cargo.toml .
ADD bin_stuff bin_stuff
ADD server server
ADD scraper scraper
ADD migrations migrations
ADD .sqlx .sqlx

RUN cargo build --release
CMD cargo run --release
