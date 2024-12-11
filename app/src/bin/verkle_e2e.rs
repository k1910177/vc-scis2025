use alloy::primitives::U256;
use alloy::providers::builder;
use app::utils::{curve_to_u256_vec, scalar_to_u256};
use ark_bn254::Fr;
use ark_ec::{AffineRepr, CurveGroup};
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
use ark_std::rand::thread_rng;
use clap::Parser;
use eyre::Result;
use foundry_contracts::verkleverifier::{
    Curve::{G1Point, G2Point},
    VerkleVerifier::{self, Multiproof, VerkleProof},
};
use rand::Rng;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    ops::Neg,
};

use std::time::Instant;
use verkle_tree::tree::VerkleTree;
type Domain = GeneralEvaluationDomain<Fr>;

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
    let mut ark_rng = thread_rng();

    let dir = format!("{}/verkle_e2e", args.output);
    if !std::path::Path::new(&dir).exists() {
        fs::create_dir_all(&dir)?;
    }

    for size in args.sizes.iter() {
        let file_path = format!("{}/verkle_{}.csv", dir, size);
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

            // Generate the setup parameters
            let mut tree = VerkleTree::setup(width.clone(), &mut ark_rng);

            // Get setup parameter
            let tau_g2_neg_vec = curve_to_u256_vec(tree.vk.beta_h.into_group().neg().into_affine());
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

            // Generate vector and commit

            let vec: Vec<Fr> = (1..=size.clone()).map(|x| Fr::from(x as u64)).collect();
            let commit_time_start = Instant::now();
            tree.commit(vec.as_slice());
            let commit_time = Instant::now() - commit_time_start;

            // Post root hash to contract
            let root_hash = scalar_to_u256(tree.root_hash());
            let builder = contract.commit(root_hash);
            let _ = builder.send().await?.watch().await?;

            // Open the tree at a random index and create a proof
            let index = rand::thread_rng().gen_range(0..size.clone());
            let open_time_start = Instant::now();
            let (value, multi_proof) = tree.open(index).unwrap();
            let open_time = Instant::now() - open_time_start;

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

            // Proof size in bytes
            let proof_size = 32 * 4 + 32 * 2 * verkle_proof.commitments.len();

            // Verify the proof on the contract
            let builder = contract.verify(index, value, verkle_proof);
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
