#!/bin/bash
# Is this a ridiculous deployment method? Yes. Does it work? Yes.
# Is it a workaround for slow cross compile times until I setup something better? Yes.
if [ -z $1 ]; then 
	echo "Need to provide a host";
else
	scp -r ./target/x86_64-unknown-linux-gnu/release/server root@$1:/root/what-bin-is-it/
fi
