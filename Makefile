USER = root
APP_FOLDER = ~/what-bin-is-it

# TODO: DB backup
# TODO: DB setup on fresh server

host-set-check:
	@if [ -z ${WHAT_BIN_HOST} ]; then\
		echo "WHAT_BIN_HOST must be set" && exit 1;\
	fi

build-for-server:
	cargo zigbuild --target x86_64-unknown-linux-gnu --release

deploy-to-prod:
	make upload-to-prod
	ssh ${USER}@${WHAT_BIN_HOST} "systemctl restart whatbin"

upload-to-prod:
	make host-set-check
	ssh ${USER}@${WHAT_BIN_HOST} "mkdir -p ${APP_FOLDER}/archive"
	scp -r ./target/x86_64-unknown-linux-gnu/release/server ${USER}@${WHAT_BIN_HOST}:${APP_FOLDER}/server-new
	ssh ${USER}@${WHAT_BIN_HOST} "if [ -f ${APP_FOLDER}/server ]; then mv ${APP_FOLDER}/server ${APP_FOLDER}/archive/server_$$(date +"%Y%m%d%H%M%S"); fi"
	ssh ${USER}@${WHAT_BIN_HOST} "mv ${APP_FOLDER}/server-new ${APP_FOLDER}/server"

setup-server:
	make host-set-check
	# APT STUFF
	ssh ${USER}@${WHAT_BIN_HOST} "apt update"
	ssh ${USER}@${WHAT_BIN_HOST} "apt -y install build-essential pkg-config libssl-dev"
	ssh ${USER}@${WHAT_BIN_HOST} "apt -y install lnav" # For journalctl logs
	# GECKODRIVER
	ssh ${USER}@${WHAT_BIN_HOST} "wget https://github.com/mozilla/geckodriver/releases/download/v0.33.0/geckodriver-v0.33.0-linux64.tar.gz "
	ssh ${USER}@${WHAT_BIN_HOST} "tar -xzvf geckodriver-v0.33.0-linux64.tar.gz"
	ssh ${USER}@${WHAT_BIN_HOST} "mv geckodriver /bin"
	# FIREFOX
	ssh ${USER}@${WHAT_BIN_HOST} "wget 'https://download-installer.cdn.mozilla.net/pub/firefox/releases/115.0.3/linux-x86_64/en-GB/firefox-115.0.3.tar.bz2'"
	ssh ${USER}@${WHAT_BIN_HOST} "tar -xjvf firefox-115.0.3.tar.bz2"
	ssh ${USER}@${WHAT_BIN_HOST} "mv firefox /bin"
	# CADDY
	make setup-caddy
	# UPLOAD
	make upload-to-prod
	# SYSTEMD
	make services-setup

setup-caddy:
	make host-set-check
	ssh ${USER}@${WHAT_BIN_HOST} "sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https"
	ssh ${USER}@${WHAT_BIN_HOST} "curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --batch --yes --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg"
	ssh ${USER}@${WHAT_BIN_HOST} "curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list"
	ssh ${USER}@${WHAT_BIN_HOST} "sudo apt update"
	ssh ${USER}@${WHAT_BIN_HOST} "sudo apt install -y caddy"


BIN_SERVICE=whatbin.service
GECKODRIVER_SERVICE=geckodriver.service
services-setup:
	scp -r ./services/${BIN_SERVICE} ${USER}@${WHAT_BIN_HOST}:/lib/systemd/system/${BIN_SERVICE}
	scp -r ./services/${GECKODRIVER_SERVICE} ${USER}@${WHAT_BIN_HOST}:/lib/systemd/system/${GECKODRIVER_SERVICE}
	ssh ${USER}@${WHAT_BIN_HOST} "systemctl daemon-reload"
	ssh ${USER}@${WHAT_BIN_HOST} "systemctl enable ${GECKODRIVER_SERVICE}"
	ssh ${USER}@${WHAT_BIN_HOST} "systemctl restart ${GECKODRIVER_SERVICE}"
	ssh ${USER}@${WHAT_BIN_HOST} "systemctl enable ${BIN_SERVICE}"
