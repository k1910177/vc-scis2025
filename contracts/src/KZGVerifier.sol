// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import {Field} from "./Field.sol";
import {Curve} from "./Curve.sol";

contract KZGVerifier {
    using Field for Field.Fr;
    using Curve for Curve.G1Point;

    Curve.Fq private immutable g2tau_x0;
    Curve.Fq private immutable g2tau_x1;
    Curve.Fq private immutable g2tau_y0;
    Curve.Fq private immutable g2tau_y1;

    Curve.G1Point public commitment;

    constructor(Curve.G2Point memory g2tau_point) {
        g2tau_x0 = g2tau_point.X[0];
        g2tau_x1 = g2tau_point.X[1];
        g2tau_y0 = g2tau_point.Y[0];
        g2tau_y1 = g2tau_point.Y[1];
    }

    function commit(Curve.G1Point memory newCommitment) external {
        commitment = newCommitment;
    }

    function verify(
        Field.Fr z,
        Field.Fr y,
        Curve.G1Point calldata proof
    ) external view returns (bool) {
        Curve.G1Point[2] memory p1 = [
            commitment.add(Curve.P1().mulScalar(y).neg()).add(
                proof.mulScalar(z)
            ),
            proof.neg()
        ];
        Curve.G2Point[2] memory p2 = [Curve.P2(), g2Tau()];

        return Curve.pairing(p1, p2);
    }

    function g2Tau() private view returns (Curve.G2Point memory) {
        return Curve.G2Point([g2tau_x0, g2tau_x1], [g2tau_y0, g2tau_y1]);
    }
}
