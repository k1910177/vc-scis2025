use alloy::primitives::U256;
use alloy::providers::builder;
use app::utils::{curve_to_u256_vec, scalar_to_u256};
use ark_bn254::Fr;
use ark_ec::{AffineRepr, CurveGroup};
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
use ark_std::rand::thread_rng;
use eyre::Result;
use foundry_contracts::verkleverifier::{
    Curve::{G1Point, G2Point},
    VerkleVerifier::{self, Multiproof, VerkleProof},
};
use std::ops::Neg;

use verkle_tree::tree::VerkleTree;
type Domain = GeneralEvaluationDomain<Fr>;

#[tokio::test]
async fn test_verkle_e2e() -> Result<()> {
    let provider = builder().with_recommended_fillers().on_anvil_with_wallet();
    let mut rng = thread_rng();

    let n = 16;
    let width = 4;

    // Generate the setup parameters
    let mut tree = VerkleTree::setup(width, &mut rng);

    // Get setup parameter
    let tau_g2_neg_vec = curve_to_u256_vec(tree.vk.beta_h.into_group().neg().into_affine());
    let tau_g2_neg_point = G2Point {
        X: [tau_g2_neg_vec[1], tau_g2_neg_vec[0]],
        Y: [tau_g2_neg_vec[3], tau_g2_neg_vec[2]],
    };
    let domain = Domain::new(width).unwrap();
    let roots_of_unity = scalar_to_u256(domain.element(1));

    // Deploy the contract
    let contract = VerkleVerifier::deploy(
        &provider,
        tau_g2_neg_point,
        U256::from(width),
        roots_of_unity,
    )
    .await?;
    println!("Deployed contract at address: {}", contract.address());

    // Generate vector and commit
    let vec: Vec<Fr> = (1..=n).map(Fr::from).collect();
    tree.commit(vec.as_slice());

    // Post root hash to contract
    let root_hash = scalar_to_u256(tree.root_hash());
    let builder = contract.commit(root_hash);
    let _ = builder.send().await?.watch().await?;

    // Open the tree at a random index and create a proof
    let index = 2;
    let (value, multi_proof) = tree.open(index).unwrap();

    // Prepare parameters for verification
    let index = U256::from(index);
    let value = scalar_to_u256(value);
    let d_vec = curve_to_u256_vec(multi_proof.multi_proof.d);
    let pi_vec = curve_to_u256_vec(multi_proof.multi_proof.pi);
    let coms: Vec<G1Point> = multi_proof
        .coms
        .iter()
        .map(|com| {
            let com_vec = curve_to_u256_vec(com.0);
            G1Point {
                X: com_vec[0],
                Y: com_vec[1],
            }
        })
        .collect();
    let multi_proof = Multiproof {
        d: G1Point {
            X: d_vec[0],
            Y: d_vec[1],
        },
        pi: G1Point {
            X: pi_vec[0],
            Y: pi_vec[1],
        },
    };
    let verkle_proof = VerkleProof {
        commitments: coms,
        multiproof: multi_proof,
    };

    // Verify the proof on the contract
    let builder = contract.verify(index, value, verkle_proof);
    let result = builder.call().await?._0;
    let gas = builder.estimate_gas().await?;
    println!("Gas used: {}", gas);

    assert!(result);

    Ok(())
}
