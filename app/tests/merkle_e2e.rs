use alloy::primitives::{FixedBytes, U256};
use alloy::providers::builder;
use eyre::Result;
use foundry_contracts::merkleverifier::MerkleVerifier;
use merkle_tree::tree::MerkleTree;
use rand::Rng;
use app::utils::*;

#[tokio::test]
async fn test_merkle_e2e() -> Result<()> {
    let provider = builder().with_recommended_fillers().on_anvil_with_wallet();

    // Setup merkle tree
    let branching_factor = 3;
    let mut tree = MerkleTree::setup(branching_factor);

    // Deploy the contract
    let contract = MerkleVerifier::deploy(&provider, U256::from(branching_factor)).await?;
    println!("Deployed contract at address: {}", contract.address());

    // Generate random vector
    let vec_length = 100;
    let vec = (0..vec_length)
        .map(|_| random_bytes32())
        .collect::<Vec<_>>();
    let vec = vec.iter().collect::<Vec<_>>();

    // Commit
    tree.commit(vec.as_slice());
    let com = tree.root_hash().unwrap();
    let builder = contract.commit(FixedBytes::from_slice(&com));
    let _ = builder.send().await?.watch().await?;
    println!(
        "Commited to: {}",
        contract.rootHash().call().await?._0.to_string()
    );

    // Open and generate proof
    let random_index = rand::thread_rng().gen_range(0..vec_length);
    let (value, proof) = tree.open(random_index);
    let proof = proof
        .hashes
        .iter()
        .map(|p| FixedBytes::from_slice(p))
        .collect::<Vec<_>>();

    // Verify the proof on the contract
    let builder = contract.verify(
        U256::from(random_index),
        U256::from_be_bytes(to_bytes32_array(&value)),
        proof,
    );
    let result = builder.call().await?._0;
    let gas = builder.estimate_gas().await?;
    println!("Gas used: {}", gas);

    assert!(result);

    Ok(())
}
