#!/usr/bin/env bash

dmntk srv -c never -D ./models 2>&1 &
_pid=$!
sleep 0.1

kill -s SIGINT "$_pid"
sleep 0.1