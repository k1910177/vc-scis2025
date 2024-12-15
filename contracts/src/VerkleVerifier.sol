// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Field} from "./Field.sol";
import {Curve} from "./Curve.sol";

// import "forge-std/console.sol";

contract VerkleVerifier {
    using Field for Field.Fr;
    using Curve for Curve.G1Point;

    // Verifying key
    Curve.Fq private immutable tauG2neg_x0;
    Curve.Fq private immutable tauG2neg_x1;
    Curve.Fq private immutable tauG2neg_y0;
    Curve.Fq private immutable tauG2neg_y1;

    // Tree config
    Field.Fr internal immutable rootsOfUnity;
    uint256 internal immutable treeWidth;

    // Hash of the commitment of the tree root
    Field.Fr public rootHash;

    // Proof of KZG Multiproof
    struct Multiproof {
        Curve.G1Point d;
        Curve.G1Point pi;
    }

    // Proof of verkle tree
    struct VerkleProof {
        Curve.G1Point[] commitments;
        Multiproof multiproof;
    }

    constructor(
        Curve.G2Point memory tauG2neg_point,
        uint256 treeWidth_,
        Field.Fr rootsOfUnity_
    ) {
        tauG2neg_x0 = tauG2neg_point.X[0];
        tauG2neg_x1 = tauG2neg_point.X[1];
        tauG2neg_y0 = tauG2neg_point.Y[0];
        tauG2neg_y1 = tauG2neg_point.Y[1];

        treeWidth = treeWidth_;
        rootsOfUnity = rootsOfUnity_;
    }

    function commit(Field.Fr rootHash_) external {
        rootHash = rootHash_;
    }

    function verify(
        uint256 index,
        Field.Fr value,
        VerkleProof calldata verkleProof
    ) external view returns (bool) {
        Field.Fr[] memory pathIndex = computePathIndex(
            index,
            verkleProof.commitments.length
        );
        (Field.Fr r, Field.Fr t, Field.Fr[] memory pathValues) = computeRTValues(
            verkleProof.commitments,
            pathIndex,
            value,
            verkleProof.multiproof.d
        );
        (Curve.G1Point memory e, Field.Fr y) = computeEY(
            r,
            t,
            pathValues,
            pathIndex,
            verkleProof.commitments
        );

        return
            checkPairing(t, y, e, verkleProof.multiproof) &&
            rootHash.eq(verkleProof.commitments[0].hash());
    }

    function computePathIndex(
        uint256 index,
        uint256 height
    ) internal view returns (Field.Fr[] memory path) {
        path = new Field.Fr[](height);
        for (uint256 i = height - 1; ; i--) {
            path[i] = index > 0
                ? rootsOfUnity.pow(index % treeWidth)
                : Field.get(1);
            index /= treeWidth;
            if (i == 0) break;
        }
    }

    function computeRTValues(
        Curve.G1Point[] calldata coms,
        Field.Fr[] memory pathIndex,
        Field.Fr value,
        Curve.G1Point calldata d
    ) internal pure returns (Field.Fr r, Field.Fr t, Field.Fr[] memory pathValues) {
        bytes memory input;
        pathValues = new Field.Fr[](coms.length);
        for (uint256 i = 0; i < coms.length; i++) {
            pathValues[i] = i < coms.length - 1 ? coms[i + 1].hash() : value.hash();
            input = abi.encodePacked(input, coms[i].toBytes());
            input = abi.encodePacked(input, pathIndex[i]);
            input = abi.encodePacked(input, pathValues[i]);
        }
        r = Field.hashBytes(input);
        t = Field.hashBytes(abi.encodePacked(d.toBytes(), r));
    }

    function computeEY(
        Field.Fr r,
        Field.Fr t,
        Field.Fr[] memory pathValues,
        Field.Fr[] memory pathIndex,
        Curve.G1Point[] calldata coms
    ) internal view returns (Curve.G1Point memory e, Field.Fr y) {
        Field.Fr rExp = Field.get(1);
        for (uint256 i = 0; i < coms.length; i++) {
            Field.Fr divisor = t.sub(pathIndex[i]);
            y = y.add(rExp.mul(pathValues[i].div(divisor)));
            e = e.add(coms[i].mulScalar(rExp.div(divisor)));
            rExp = rExp.mul(r);
        }
    }

    function tauG2neg() internal view returns (Curve.G2Point memory) {
        return Curve.G2Point([tauG2neg_x0, tauG2neg_x1], [tauG2neg_y0, tauG2neg_y1]);
    }

    function checkPairing(
        Field.Fr t,
        Field.Fr y,
        Curve.G1Point memory e,
        Multiproof calldata proof
    ) internal view returns (bool) {
        Curve.G1Point[2] memory p1 = [
            e.sub(proof.d).sub(Curve.P1().mulScalar(y)).add(
                proof.pi.mulScalar(t)
            ),
            proof.pi
        ];
        Curve.G2Point[2] memory p2 = [Curve.P2(), tauG2neg()];

        return Curve.pairing(p1, p2);
    }
}
