// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import {Field} from "./Field.sol";

library Curve {
    // The base field
    type Fq is uint256;

    // Field size of base field
    uint256 private constant FIELD_SIZE =
        21888242871839275222246405745257275088696311157297823662689037894645226208583;

    struct G1Point {
        Fq X;
        Fq Y;
    }

    // Encoding of field elements is: X[0] * z + X[1]
    struct G2Point {
        Fq[2] X;
        Fq[2] Y;
    }

    /// @return the generator of G1
    function P1() internal pure returns (G1Point memory) {
        return G1Point(Fq.wrap(1), Fq.wrap(2));
    }

    /// @return the generator of G2
    function P2() internal pure returns (G2Point memory) {
        return
            G2Point(
                [
                    Fq.wrap(
                        11559732032986387107991004021392285783925812861821192530917403151452391805634
                    ),
                    Fq.wrap(
                        10857046999023057135944570762232829481370756359578518086990519993285655852781
                    )
                ],
                [
                    Fq.wrap(
                        4082367875863433681332203403145435568316851327593401208105741076214120093531
                    ),
                    Fq.wrap(
                        8495653923123431417604973247489272438418190587263600148770280649306958101930
                    )
                ]
            );
    }

    function neg(G1Point memory p) internal pure returns (G1Point memory) {
        // The prime q in the base field F_q for G1
        if (Fq.unwrap(p.X) == 0 && Fq.unwrap(p.Y) == 0)
            return G1Point(Fq.wrap(0), Fq.wrap(0));
        return
            G1Point(p.X, Fq.wrap(FIELD_SIZE - (Fq.unwrap(p.Y) % FIELD_SIZE)));
    }

    function add(
        G1Point memory p1,
        G1Point memory p2
    ) internal view returns (G1Point memory r) {
        Fq[4] memory input;
        input[0] = p1.X;
        input[1] = p1.Y;
        input[2] = p2.X;
        input[3] = p2.Y;
        bool success;

        // solium-disable-next-line security/no-inline-assembly
        assembly {
            success := staticcall(sub(gas(), 2000), 0x6, input, 0xc0, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success
            case 0 {
                invalid()
            }
        }

        require(success, "pairing-add-failed");
    }

    function sub(
        G1Point memory p1,
        G1Point memory p2
    ) internal view returns (G1Point memory) {
        return add(p1, neg(p2));
    }

    function mulScalar(
        G1Point memory p,
        Field.Fr s
    ) internal view returns (G1Point memory r) {
        uint256[3] memory input;
        input[0] = Fq.unwrap(p.X);
        input[1] = Fq.unwrap(p.Y);
        input[2] = Field.Fr.unwrap(s);
        bool success;
        // solium-disable-next-line security/no-inline-assembly
        assembly {
            success := staticcall(sub(gas(), 2000), 0x7, input, 0x80, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success
            case 0 {
                invalid()
            }
        }
        require(success, "pairing-mul-failed");
    }

    function pairing(
        G1Point[2] memory p1,
        G2Point[2] memory p2
    ) internal view returns (bool) {
        uint256 inputSize = 12;
        Fq[] memory input = new Fq[](inputSize);

        for (uint256 i = 0; i < 2; i++) {
            uint256 j = i * 6;
            input[j + 0] = p1[i].X;
            input[j + 1] = p1[i].Y;
            input[j + 2] = p2[i].X[0];
            input[j + 3] = p2[i].X[1];
            input[j + 4] = p2[i].Y[0];
            input[j + 5] = p2[i].Y[1];
        }
        uint[1] memory out;
        bool success;
        // solium-disable-next-line security/no-inline-assembly
        assembly {
            success := staticcall(
                sub(gas(), 2000),
                0x8,
                add(input, 0x20),
                mul(inputSize, 0x20),
                out,
                0x20
            )
            // Use "invalid" to make gas estimation work
            switch success
            case 0 {
                invalid()
            }
        }
        require(success, "pairing-opcode-failed");
        return out[0] != 0;
    }

    function toBytes(G1Point calldata p) internal pure returns (bytes memory) {
        return abi.encodePacked(p.X, p.Y);
    }

    function hash(G1Point calldata p) internal pure returns (Field.Fr) {
        return Field.hashBytes(toBytes(p));
    }
}
