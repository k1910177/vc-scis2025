#!/bin/bash

if [ "$1" == "debug" ]; then
    BINARY_PATH=./target/debug/merkle_e2e
else
    BINARY_PATH=./target/release/merkle_e2e
fi

WIDTHS=2,4,8,32,64,256
SIZES=10,100,1000,5000,10000,50000,100000,500000,1000000,5000000
OUTPUT_PATH=./result

$BINARY_PATH --widths $WIDTHS --sizes $SIZES --output $OUTPUT_PATH
