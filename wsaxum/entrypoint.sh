#!/bin/bash

set -x
set -e
# Add network latency
tc qdisc add dev eth0 root netem delay 20ms

ulimit -n 1000000
# Wait for server to start
sleep 3
RUST_LOG=info wsaxum     
