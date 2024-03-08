# What bin is it

Scrapes the North Lanarkshire site for the next bins collection, then sends an email with the bins to be put out using AWS SES.

Pulls emails, postcodes, and addresses to check from the sqlite database file given by the DATABASE_URL env var

Run `geckodriver` before running the program

## TODO
- Caddyfile is hardcoded to use a specific domain. Need to make dynamic

## ENV vars
### Required ENV vars
See https://docs.aws.amazon.com/ses/latest/dg/setting-up.html for AWS related credentials

AWS_ACCESS_KEY_ID  
AWS_SECRET_ACCESS_KEY  
FROM_EMAIL_ADDRESS  
ERROR_EMAIL_ADDRESS
DATABASE_URL
ADMIN_PASSWORD

### Optional ENV vars
GECKODRIVER_URL

## Dependencies
For server dependencies, see [Server setup](#server-setup)
```
cargo install sqlx-cli
cargo install cargo-zigbuild
brew install geckodriver
brew install zig
```

## Cross compile
requires [Ziglang](https://ziglang.org/)
```
brew install zig
cargo install cargo-zigbuild
make build-for-server
```

## Deployment
It is assumed that the app will have a working directory at `/root/what-bin-is-it/`

Ensure that the the host server has an .env file in the working directory of the app with the required ENV vars as listed in the [required ENV vars](#required-env-vars) section
### Server setup
The `setup-server` make target will:
- Install required packages on the server, such as `geckodriver` and `Firefox`.
- Install [Caddy](https://caddyserver.com/), the reverse proxy of choice.
- Upload the `What Bin` server binary to the server (So make sure to build it first).
- Setup [Services](https://wiki.debian.org/systemd/Services) for [systemd](https://systemd.io/) (Except Caddy).
- * Geckodriver will automatically be started as a Service (assuming it is installed). The profile for the driver will be at `/root/geckodriver-profiles`.

The `deploy-caddy` make target will:
- Copy our Caddyfile to the server.

NOTE: The Caddyfile should be updated to point to the domain you want to use
```
make setup-server WHAT_BIN_HOST=<ip for host>
make deploy-caddy WHAT_BIN_HOST=<ip for host>
```

### Deploying a binary
The `deploy-to-prod` make target will:
- Copy the RELEASE binary to the server.
- Restart the `whatbin` service

```
make build-for-server
make deploy-to-prod WHAT_BIN_HOST=<ip for host>
```

## Run now
If a file named `run-now` is found in the working directory of the program at startup, then the file is deleted, and scraping and sending of emails will begin immediately.  
