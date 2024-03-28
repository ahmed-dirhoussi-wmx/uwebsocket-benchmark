#!/bin/bash

docker exec client tc qdisc add dev eth0 root netem delay 20ms
docker exec server tc qdisc add dev eth0 root netem delay 20ms