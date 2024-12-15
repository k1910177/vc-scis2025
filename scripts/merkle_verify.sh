#!/bin/bash

if [ "$1" == "debug" ]; then
    BINARY_PATH=./target/debug/merkle_verify
else
    BINARY_PATH=./target/release/merkle_verify
fi

WIDTHS_POWERS_FROM=1
WIDTHS_POWERS_TO=10

SIZES_POWERS=6,9,12
OUTPUT_PATH=./result

$BINARY_PATH --widths-powers-from $WIDTHS_POWERS_FROM --widths-powers-to $WIDTHS_POWERS_TO --sizes-powers $SIZES_POWERS --output $OUTPUT_PATH
