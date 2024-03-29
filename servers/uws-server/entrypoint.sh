#! /bin/bash
set -x
set -e
# Add network latency
tc qdisc add dev eth0 root netem delay 20ms

# Start connection
ulimit -n 1000000
chmod +x build/uws-server
./build/uws-server