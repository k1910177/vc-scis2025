use alloy::primitives::{FixedBytes, U256};
use alloy::providers::builder;
use app::utils::*;
use clap::Parser;
use eyre::Result;
use foundry_contracts::merkleverifier::MerkleVerifier;
use merkle_tree::tree::MerkleTree;
use rand::Rng;
use std::time::Instant;
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, value_parser, value_delimiter = ',')]
    widths: Vec<usize>,

    #[arg(long, value_parser, value_delimiter = ',')]
    sizes: Vec<usize>,

    #[arg(long)]
    output: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let provider = builder().with_recommended_fillers().on_anvil_with_wallet();

    let dir = format!("{}/merkle_e2e", args.output);
    if !std::path::Path::new(&dir).exists() {
        fs::create_dir_all(&dir)?;
    }

    for size in args.sizes.iter() {
        let file_path = format!("{}/merkle_{}.csv", dir, size);
        let mut file = if !fs::metadata(file_path.clone()).is_ok() {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(file_path)?;
            writeln!(
                file,
                "size,width,result,gas,proof_size,commit_time,open_time"
            )?;
            file
        } else {
            OpenOptions::new().append(true).open(file_path)?
        };
        for width in args.widths.iter() {
            println!("################################");
            println!("Size: {:?}, Width: {:?}", size, width);

            let mut tree = MerkleTree::setup(width.clone());

            // Deploy the contract
            let contract = MerkleVerifier::deploy(&provider, U256::from(width.clone())).await?;

            // Generate random vector
            let vec = (0..size.clone())
                .map(|_| random_bytes32())
                .collect::<Vec<_>>();
            let vec = vec.iter().collect::<Vec<_>>();

            // Commit
            let commit_time_start = Instant::now();
            tree.commit(vec.as_slice());
            let commit_time = Instant::now() - commit_time_start;
            let com = tree.root_hash().unwrap();
            let builder = contract.commit(FixedBytes::from_slice(&com));
            let _ = builder.send().await?.watch().await?;

            // Open and generate proof
            let random_index = rand::thread_rng().gen_range(0..size.clone());
            let open_time_start = Instant::now();
            let (value, proof) = tree.open(random_index);
            let open_time = Instant::now() - open_time_start;
            let proof = proof
                .hashes
                .iter()
                .map(|p| FixedBytes::from_slice(p))
                .collect::<Vec<_>>();

            // Proof size in bytes
            let proof_size = 32 * proof.len();

            // Verify the proof on the contract
            let builder = contract.verify(
                U256::from(random_index),
                U256::from_be_bytes(to_bytes32_array(&value)),
                proof,
            );
            let result = builder.call().await?._0;
            let gas = builder.estimate_gas().await?;

            println!(
                "Result: {}, Gas: {}, Proof size: {}, Commit time: {}, Open time: {}",
                result,
                gas,
                proof_size,
                commit_time.as_millis(),
                open_time.as_millis()
            );
            writeln!(
                file,
                "{},{},{},{},{},{},{}",
                size,
                width,
                result,
                gas,
                proof_size,
                commit_time.as_millis(),
                open_time.as_millis()
            )?;
        }
    }

    Ok(())
}
