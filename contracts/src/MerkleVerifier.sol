// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

contract MerkleVerifier {
    bytes32 public rootHash;
    uint256 internal immutable treeWidth;

    constructor(uint256 treeWidth_) {
        treeWidth = treeWidth_;
    }

    function commit(bytes32 newRootHash) external {
        rootHash = newRootHash;
    }

    function verify(
        uint256 index,
        uint256 value,
        bytes32[] memory proof
    ) external view returns (bool) {
        bytes32 computedHash = keccak256(abi.encodePacked(value));

        for (uint256 proofIdx = 0; proofIdx < proof.length; ) {
            bytes memory siblings;
            for (
                uint256 siblingsIdx = 0;
                siblingsIdx < treeWidth;
                siblingsIdx++
            ) {
                siblings = abi.encodePacked(
                    siblings,
                    siblingsIdx == index % treeWidth
                        ? computedHash
                        : proof[proofIdx++]
                );
            }
            computedHash = keccak256(siblings);
            index /= treeWidth;
        }

        return computedHash == rootHash;
    }
}
