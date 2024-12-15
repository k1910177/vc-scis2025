// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/console.sol";

library Field {
    // Scalar Field
    type Fr is uint256;

    // Field size of scalar field
    uint256 private constant FIELD_SIZE =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;

    function get(uint256 i) internal pure returns (Fr) {
        return Fr.wrap(i % FIELD_SIZE);
    }

    function add(Fr a, Fr b) internal pure returns (Fr) {
        return Fr.wrap(addmod(Fr.unwrap(a), Fr.unwrap(b), FIELD_SIZE));
    }

    function sub(Fr a, Fr b) internal pure returns (Fr) {
        return add(a, Fr.wrap(FIELD_SIZE - (Fr.unwrap(b) % FIELD_SIZE)));
    }

    function mul(Fr a, Fr b) internal pure returns (Fr) {
        return Fr.wrap(mulmod(Fr.unwrap(a), Fr.unwrap(b), FIELD_SIZE));
    }

    function pow(Fr base, uint256 exponent) internal pure returns (Fr) {
        uint256 result = 1;
        base = Fr.wrap(Fr.unwrap(base) % FIELD_SIZE);
        while (exponent != 0) {
            if (exponent & 1 != 0) {
                result = mulmod(result, Fr.unwrap(base), FIELD_SIZE);
            }
            base = Fr.wrap(
                mulmod(Fr.unwrap(base), Fr.unwrap(base), FIELD_SIZE)
            );
            exponent >>= 1;
        }
        return Fr.wrap(result);
    }

    function inv(Fr a) internal pure returns (Fr) {
        require(Fr.unwrap(a) % FIELD_SIZE != 0, "cannot divide by zero");
        return pow(a, FIELD_SIZE - 2);
    }

    function div(Fr dividend, Fr divisor) internal pure returns (Fr) {
        return mul(dividend, inv(divisor));
    }

    function eq(Fr a, Fr b) internal pure returns (bool) {
        return Fr.unwrap(a) == Fr.unwrap(b);
    }

    function toBytes(Fr a) internal pure returns (bytes memory) {
        return abi.encodePacked(a);
    }

    function hash(Fr a) internal pure returns (Fr) {
        return Fr.wrap(uint256(keccak256(toBytes(a))) % FIELD_SIZE);
    }

    function hashBytes(bytes memory a) internal pure returns (Fr) {
        return Fr.wrap(uint256(keccak256(a)) % FIELD_SIZE);
    }
}
