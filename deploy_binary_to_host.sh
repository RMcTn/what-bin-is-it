#!/bin/bash
# Is this a ridiculous deployment method? Yes. Does it work? Yes.
# Is it a workaround for slow cross compile times until I setup something better? Yes.
if [ -z $1 ]; then 
	echo "Need to provide a host";
else
	ssh root@$1 "mkdir -p /root/what-bin-is-it/archive"
	scp -r ./target/x86_64-unknown-linux-gnu/release/server root@$1:/root/what-bin-is-it/server-new
	ssh root@$1 "mv /root/what-bin-is-it/server /root/what-bin-is-it/archive/server_$(date +"%Y%m%d%H%M%S")"
	ssh root@$1 "mv /root/what-bin-is-it/server-new /root/what-bin-is-it/server"
	ssh root@$1 "systemctl restart whatbin"
fi
