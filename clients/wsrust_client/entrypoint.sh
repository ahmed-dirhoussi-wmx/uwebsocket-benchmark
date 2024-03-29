#!/bin/bash

set -x
set -e
# Reduce network
tc qdisc add dev eth0 root netem delay 20ms
ulimit -n 1000000

nclients=(1000 3000 5000 10000)
batch_sizes=(1)
waits=(1000)

# Iterate over each combination of values
for client in "${nclients[@]}"; do
    for batch_size in "${batch_sizes[@]}"; do
        for wait in "${waits[@]}"; do
            # Wait for server to start
            sleep 3
            # Execute the command with the current values
            RUST_LOG=info ws_client \
                -s ws://wsserver:3000/ws \
                -c "$client" \
                -b "$batch_size" \
                -n 100 \
                -w "$wait" \
                -r 2 \
                --result-dir /app/results
        done
    done
done