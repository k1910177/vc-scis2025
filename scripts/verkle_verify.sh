#!/bin/bash

if [ "$1" == "debug" ]; then
    BINARY_PATH=./target/debug/verkle_verify
else
    BINARY_PATH=./target/release/verkle_verify
fi

WIDTHS=256,1024,4096
SIZES_POWER_FROM=2
SIZES_POWER_TO=18
OUTPUT_PATH=./result

$BINARY_PATH --widths $WIDTHS --sizes-power-from $SIZES_POWER_FROM --sizes-power-to $SIZES_POWER_TO --output $OUTPUT_PATH
