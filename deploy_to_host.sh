#!/bin/bash
# Is this a ridiculous deployment method? Yes. Does it work? Yes.
# Is it a workaround for slow cross compile times until I setup something better? Yes.
if [ -z $1 ]; then 
	echo "Need to provide a host";
else
	scp -r Cargo.toml Cargo.lock build.rs docker-compose.yml Dockerfile root@$1:/root/what-bin-is-it
	scp -r scraper bin_stuff server migrations .sqlx root@$1:/root/what-bin-is-it
fi
