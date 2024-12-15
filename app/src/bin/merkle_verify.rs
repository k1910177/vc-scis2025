use alloy::primitives::{FixedBytes, U256};
use alloy::providers::builder;
use app::utils::*;
use clap::Parser;
use eyre::Result;
use foundry_contracts::merkleverifier::MerkleVerifier;
use itertools::izip;
use merkle_tree::tree::ceil_log_base;
use rand::Rng;
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    widths_powers_from: usize,

    #[arg(long)]
    widths_powers_to: usize,

    #[arg(long, value_parser, value_delimiter = ',')]
    sizes_powers: Vec<usize>,

    #[arg(long)]
    output: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let provider = builder().with_recommended_fillers().on_anvil_with_wallet();

    let sizes: Vec<usize> = args
        .sizes_powers
        .iter()
        .map(|power| 10_usize.pow(*power as u32))
        .collect();
    let widths_powers: Vec<usize> = (args.widths_powers_from..=args.widths_powers_to).collect();
    let widths: Vec<usize> = widths_powers
        .iter()
        .map(|power| 2_usize.pow(*power as u32))
        .collect();

    if !std::path::Path::new(&args.output).exists() {
        fs::create_dir_all(&args.output)?;
    }

    let file_path = format!("{}/merkle_verify.csv", args.output);
    let mut file = if !fs::metadata(file_path.clone()).is_ok() {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)?;
        writeln!(file, "power,size,width,result,gas,proof_size")?;
        file
    } else {
        OpenOptions::new().append(true).open(file_path)?
    };

    for (size, power) in izip!(sizes, args.sizes_powers) {
        for width in widths.iter() {
            println!("################################");
            println!("Power: {:?}, Width: {:?}", power, width);

            let height = ceil_log_base(width.clone(), size.clone());

            // Deploy the contract
            let contract = MerkleVerifier::deploy(&provider, U256::from(width.clone())).await?;

            // Commit
            let com = random_bytes32();
            let builder = contract.commit(FixedBytes::from_slice(&com));
            let _ = builder.send().await?.watch().await?;

            // Open and generate proof
            let random_index = rand::thread_rng().gen_range(0..size.clone());
            let value = random_bytes32();
            let proof_size = (width - 1) * height;
            let proof: Vec<FixedBytes<32>> = (0..proof_size)
                .into_iter()
                .map(|_| {
                    let random = random_bytes32();
                    FixedBytes::from_slice(&random)
                })
                .collect();

            // Proof size in bytes
            let proof_size = 32 * proof.len();

            // Verify the proof on the contract
            let builder = contract.verify(
                U256::from(random_index),
                U256::from_be_bytes(to_bytes32_array(&value)),
                proof,
            );
            let gas_result = builder.estimate_gas().await;
            let gas = match gas_result {
                Ok(gas) => gas,
                Err(_) => 30000000,
            };
            println!("Result: {}, Gas: {}", false, gas);
            writeln!(
                file,
                "{},{},{},{},{},{}",
                power, size, width, false, gas, proof_size
            )?;
        }
    }

    Ok(())
}
