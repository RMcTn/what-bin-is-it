# What bin is it

Scrapes the North Lanarkshire site for the next bins collection, then sends an email with the bins to be put out using AWS SES.

Pulls emails, postcodes, and addresses to check from the sqlite database file given by the DB_FILENAME env var

```
brew install geckodriver
brew install zig
cargo install cargo-zigbuild
```

Run ```geckodriver``` before running the program

## Required ENV vars
See https://docs.aws.amazon.com/ses/latest/dg/setting-up.html for AWS related credentials

AWS_ACCESS_KEY_ID  
AWS_SECRET_ACCESS_KEY  
FROM_EMAIL_ADDRESS  
DATABASE_URL
ADMIN_PASSWORD

## Dependencies
```cargo install sqlx-cli```

## Cross compile
requires Zig
```
brew install zig
cargo install cargo-zigbuild
make build-for-server
```
