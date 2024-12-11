// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import {VerkleVerifier} from "src/VerkleVerifier.sol";
import {Field} from "src/Field.sol";
import {Curve} from "src/Curve.sol";

contract Verkle is VerkleVerifier {
    constructor(
        Curve.G2Point memory g2tau_point,
        uint256 treeWidth_,
        Field.Fr rootsOfUnity_
    ) VerkleVerifier(g2tau_point, treeWidth_, rootsOfUnity_) {}

    function exposed_computePathIndex(
        uint256 index,
        uint256 height
    ) external view returns (Field.Fr[] memory path) {
        return super.computePathIndex(index, height);
    }

    function exposed_computeRTValues(
        Curve.G1Point[] calldata coms,
        Field.Fr[] memory pathIndex,
        Field.Fr value,
        Curve.G1Point calldata d
    ) external pure returns (Field.Fr r, Field.Fr t, Field.Fr[] memory values) {
        return super.computeRTValues(coms, pathIndex, value, d);
    }

    function exposed_computeEY(
        Field.Fr r,
        Field.Fr t,
        Field.Fr[] memory values,
        Field.Fr[] memory pathIndex,
        Curve.G1Point[] calldata coms
    ) external view returns (Curve.G1Point memory e, Field.Fr y) {
        return super.computeEY(r, t, values, pathIndex, coms);
    }

    function exposed_checkPairing(
        Field.Fr t,
        Field.Fr y,
        Curve.G1Point memory e,
        Multiproof calldata proof
    ) external view returns (bool) {
        return super.checkPairing(t, y, e, proof);
    }
}

contract VerkleVerifierTest is Test {
    using Field for Field.Fr;

    function test_computePathIndex() public {
        uint256 width = 4;
        uint256 height = 3;
        uint256 index = 2;
        Curve.G2Point memory zero;
        Verkle verkle = new Verkle(
            zero,
            width,
            Field.get(
                21888242871839275217838484774961031246007050428528088939761107053157389710902
            )
        );

        Field.Fr[] memory path = verkle.exposed_computePathIndex(index, height);
        assertEq(Field.Fr.unwrap(path[0]), 1);
        assertEq(Field.Fr.unwrap(path[1]), 1);
        assertEq(
            Field.Fr.unwrap(path[2]),
            21888242871839275222246405745257275088548364400416034343698204186575808495616
        );
    }

    function test_hash() public pure {
        Field.Fr a = Field.get(1);
        Field.Fr b = Field.get(2);

        bytes memory input = abi.encodePacked(a, b);
        Field.Fr result = Field.hashBytes(input);
        console.log("Result: ", Field.Fr.unwrap(result));
    }

    function test_computeRTValues() public {
        Curve.G2Point memory zero;
        Verkle verkle = new Verkle(zero, 0, Field.get(0));

        // Coms
        Curve.G1Point[] memory coms = new Curve.G1Point[](3);
        coms[0] = Curve.G1Point(
            Curve.Fq.wrap(
                5361083988867089710399804967480237779404359229360034048409193195725681861878
            ),
            Curve.Fq.wrap(
                17061756550767279298134144503746169082243458154456812990763095040247808661627
            )
        );
        coms[1] = Curve.G1Point(
            Curve.Fq.wrap(
                7371129781918440970809413557713693726021781394900063170599372464588760437252
            ),
            Curve.Fq.wrap(
                1179726416664383435354064713461112181483567465773180560024548364940552709351
            )
        );
        coms[2] = Curve.G1Point(
            Curve.Fq.wrap(
                414010214684174575640022944833839238575651792281152505652280931618686852350
            ),
            Curve.Fq.wrap(
                9154603592284676647174399678720717371050754519446580810525660760148997905138
            )
        );

        // treePath
        Field.Fr[] memory treePath = new Field.Fr[](3);
        treePath[0] = Field.get(1);
        treePath[1] = Field.get(1);
        treePath[2] = Field.get(
            21888242871839275222246405745257275088548364400416034343698204186575808495616
        );

        // value
        Field.Fr value = Field.get(3);

        // d
        Curve.G1Point memory d = Curve.G1Point(
            Curve.Fq.wrap(
                6060724583951416920956867748784536245435835582245323619736950249548145813744
            ),
            Curve.Fq.wrap(
                13746347499950653034215973399582598601955127370997677082062522091953795166209
            )
        );

        (Field.Fr r, Field.Fr t, Field.Fr[] memory values) = verkle
            .exposed_computeRTValues(coms, treePath, value, d);

        assertEq(
            Field.Fr.unwrap(r),
            11780454615500157520102689548655556374613356041053947727847761544501755914189
        );
        assertEq(
            Field.Fr.unwrap(t),
            1579437046259835611844654357071388436755610796452204807854455358064060693308
        );
        assertEq(
            Field.Fr.unwrap(values[0]),
            16574507428808333053086891268119249045180748155885729818898527890623608351086
        );
        assertEq(
            Field.Fr.unwrap(values[1]),
            16426313097779368566631131837912972000884308299076469260904415224381387532326
        );
        assertEq(
            Field.Fr.unwrap(values[2]),
            350058383718813365392004927025474590697634285266444909592954063146796054615
        );
    }

    function test_computeEY() public {
        Curve.G2Point memory zero;
        Verkle verkle = new Verkle(zero, 0, Field.get(0));

        // r,t
        Field.Fr r = Field.get(
            11780454615500157520102689548655556374613356041053947727847761544501755914189
        );
        Field.Fr t = Field.get(
            1579437046259835611844654357071388436755610796452204807854455358064060693308
        );

        // Coms
        Curve.G1Point[] memory coms = new Curve.G1Point[](3);
        coms[0] = Curve.G1Point(
            Curve.Fq.wrap(
                5361083988867089710399804967480237779404359229360034048409193195725681861878
            ),
            Curve.Fq.wrap(
                17061756550767279298134144503746169082243458154456812990763095040247808661627
            )
        );
        coms[1] = Curve.G1Point(
            Curve.Fq.wrap(
                7371129781918440970809413557713693726021781394900063170599372464588760437252
            ),
            Curve.Fq.wrap(
                1179726416664383435354064713461112181483567465773180560024548364940552709351
            )
        );
        coms[2] = Curve.G1Point(
            Curve.Fq.wrap(
                414010214684174575640022944833839238575651792281152505652280931618686852350
            ),
            Curve.Fq.wrap(
                9154603592284676647174399678720717371050754519446580810525660760148997905138
            )
        );

        // pathIndex
        Field.Fr[] memory pathIndex = new Field.Fr[](3);
        pathIndex[0] = Field.get(1);
        pathIndex[1] = Field.get(1);
        pathIndex[2] = Field.get(
            21888242871839275222246405745257275088548364400416034343698204186575808495616
        );

        // Values
        Field.Fr[] memory values = new Field.Fr[](3);
        values[0] = Field.get(
            16574507428808333053086891268119249045180748155885729818898527890623608351086
        );
        values[1] = Field.get(
            16426313097779368566631131837912972000884308299076469260904415224381387532326
        );
        values[2] = Field.get(
            350058383718813365392004927025474590697634285266444909592954063146796054615
        );

        (Curve.G1Point memory e, Field.Fr y) = verkle.exposed_computeEY(
            r,
            t,
            values,
            pathIndex,
            coms
        );

        assertEq(
            Curve.Fq.unwrap(e.X),
            10052286209880993681738282449484771248156780523149358278833404613408747687635
        );
        assertEq(
            Curve.Fq.unwrap(e.Y),
            14609002888602930800301319480365945505855402806062214920563973524081040762143
        );
        assertEq(
            Field.Fr.unwrap(y),
            6271523264282089853788485622305132448856969987971293918265561179693349249035
        );
    }

    function test_checkPairing() public {
        Curve.G2Point memory tau = Curve.G2Point(
            [
                Curve.Fq.wrap(
                    21078157932976788369811386224298604876283678095953300695193627580536184663017
                ),
                Curve.Fq.wrap(
                    3260306681974474822604563648776815682816091416970286175188033719997424721292
                )
            ],
            [
                Curve.Fq.wrap(
                    21872932232854780648641376857253831029627783545362812619304934661720191529610
                ),
                Curve.Fq.wrap(
                    15974835493233460998260511752626128505010865937675816409903067489306439600529
                )
            ]
        );
        Verkle verkle = new Verkle(tau, 0, Field.get(0));

        Field.Fr t = Field.get(
            1579437046259835611844654357071388436755610796452204807854455358064060693308
        );
        Field.Fr y = Field.get(
            6271523264282089853788485622305132448856969987971293918265561179693349249035
        );
        Curve.G1Point memory e = Curve.G1Point(
            Curve.Fq.wrap(
                10052286209880993681738282449484771248156780523149358278833404613408747687635
            ),
            Curve.Fq.wrap(
                14609002888602930800301319480365945505855402806062214920563973524081040762143
            )
        );
        Curve.G1Point memory d = Curve.G1Point(
            Curve.Fq.wrap(
                6060724583951416920956867748784536245435835582245323619736950249548145813744
            ),
            Curve.Fq.wrap(
                13746347499950653034215973399582598601955127370997677082062522091953795166209
            )
        );
        Curve.G1Point memory pi = Curve.G1Point(
            Curve.Fq.wrap(
                2666775281582791871520907961165093291574662709309750242366658028611767515793
            ),
            Curve.Fq.wrap(
                4953442848828247690981109000163821029141680911963254362791262857614238332014
            )
        );
        VerkleVerifier.Multiproof memory multiProof = VerkleVerifier
            .Multiproof(d, pi);

        bool result = verkle.exposed_checkPairing(t, y, e, multiProof);
        assertEq(result, true);
    }
}
