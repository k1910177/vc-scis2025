#!/bin/bash

if [ "$1" == "debug" ]; then
    BINARY_PATH=./target/debug/merkle_verify
else
    BINARY_PATH=./target/release/merkle_verify
fi

WIDTHS=2,4,8
SIZES_POWER_FROM=2
SIZES_POWER_TO=18
OUTPUT_PATH=./result

$BINARY_PATH --widths $WIDTHS --sizes-power-from $SIZES_POWER_FROM --sizes-power-to $SIZES_POWER_TO --output $OUTPUT_PATH
