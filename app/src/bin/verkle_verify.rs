use alloy::primitives::U256;
use alloy::providers::builder;
use app::utils::*;
use ark_bn254::{Bn254, Fr};
use ark_ec::pairing::Pairing;
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
use ark_std::rand::thread_rng;
use ark_std::UniformRand;
use clap::Parser;
use eyre::Result;
use foundry_contracts::verkleverifier::{
    Curve::{G1Point, G2Point},
    VerkleVerifier::{self, Multiproof, VerkleProof},
};
use itertools::izip;
use rand::Rng;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    ops::Neg,
};
use verkle_tree::tree::ceil_log_base;

type Domain = GeneralEvaluationDomain<Fr>;

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
    let mut ark_rng = thread_rng();

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

    let file_path = format!("{}/verkle_verify.csv", args.output);
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

            // Generate the setup parameters
            let random_beta_h = <Bn254 as Pairing>::G2::rand(&mut ark_rng);

            // Get setup parameter
            let tau_g2_neg_vec = curve_to_u256_vec(random_beta_h.neg());
            let tau_g2_neg_point = G2Point {
                X: [tau_g2_neg_vec[1], tau_g2_neg_vec[0]],
                Y: [tau_g2_neg_vec[3], tau_g2_neg_vec[2]],
            };
            let domain = Domain::new(width.clone()).unwrap();
            let roots_of_unity = scalar_to_u256(domain.element(1));

            // Deploy the contract
            let contract = VerkleVerifier::deploy(
                &provider,
                tau_g2_neg_point,
                U256::from(width.clone()),
                roots_of_unity,
            )
            .await?;

            // Post root hash to contract
            let root_hash = random_bytes32();
            let builder = contract.commit(U256::from_be_slice(root_hash.as_slice()));
            let _ = builder.send().await?.watch().await?;

            // Open the tree at a random index and create a proof
            let index = rand::thread_rng().gen_range(0..size.clone());
            let value = Fr::rand(&mut ark_rng);

            // Prepare parameters for verification
            let index = U256::from(index);
            let value = scalar_to_u256(value);
            let random_d = <Bn254 as Pairing>::G1::rand(&mut ark_rng);
            let random_pi = <Bn254 as Pairing>::G1::rand(&mut ark_rng);
            let d_vec = curve_to_u256_vec(random_d);
            let pi_vec = curve_to_u256_vec(random_pi);
            let coms: Vec<G1Point> = (0..height)
                .into_iter()
                .map(|_| {
                    let random_com = <Bn254 as Pairing>::G1::rand(&mut ark_rng);
                    let com_vec = curve_to_u256_vec(random_com);
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

            // Proof size in bytes
            let proof_size = 32 * 4 + 32 * 2 * verkle_proof.commitments.len();

            // Verify the proof on the contract
            let builder = contract.verify(index, value, verkle_proof);
            let gas_result = builder.estimate_gas().await;
            let gas = match gas_result {
                Ok(gas) => gas,
                Err(_) => 30000000
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
