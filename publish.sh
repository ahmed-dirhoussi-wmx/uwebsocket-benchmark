#!/bin/bash

# Set the message you want to publish
MESSAGE="Message :  "

# Set the number of times you want to publish the message
NUM_TIMES=10

# Set the channel name
CHANNEL="test"

# Loop to publish the message multiple times
for ((i=1; i<=$NUM_TIMES; i++)); do
    redis-cli publish $CHANNEL "$MESSAGE $i"
    sleep 0.01
done
