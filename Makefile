USER = root

build-for-server:
	cargo zigbuild --target x86_64-unknown-linux-gnu --release

deploy-to-prod:
	@if [ -z ${WHAT_BIN_HOST} ]; then\
		echo "WHAT_BIN_HOST must be set" && exit 1;\
	fi
	ssh ${USER}@${WHAT_BIN_HOST} "mkdir -p /root/what-bin-is-it/archive"
	scp -r ./target/x86_64-unknown-linux-gnu/release/server ${USER}@${WHAT_BIN_HOST}:/root/what-bin-is-it/server-new
	ssh ${USER}@${WHAT_BIN_HOST} "mv /root/what-bin-is-it/server /root/what-bin-is-it/archive/server_$(date +"%Y%m%d%H%M%S")"
	ssh ${USER}@${WHAT_BIN_HOST} "mv /root/what-bin-is-it/server-new /root/what-bin-is-it/server"
	ssh ${USER}@${WHAT_BIN_HOST} "systemctl restart whatbin"
