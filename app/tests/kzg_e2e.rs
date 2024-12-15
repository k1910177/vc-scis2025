use alloy::providers::builder;
use app::utils::*;
use ark_bn254::{Bn254, Fr};
use ark_ec::pairing::Pairing;
use ark_ff::UniformRand;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
use ark_std::rand::thread_rng;
use eyre::Result;
use foundry_contracts::kzgverifier::{
    Curve::{G1Point, G2Point},
    KZGVerifier,
};
use kzg_commitment::kzg::KZG;

type UniPoly = DensePolynomial<<Bn254 as Pairing>::ScalarField>;
type PCS = KZG<Bn254, UniPoly>;

#[tokio::test]
async fn test_kzg_e2e() -> Result<()> {
    let provider = builder().with_recommended_fillers().on_anvil_with_wallet();
    let mut rng = thread_rng();

    // Define the degree of the polynomial
    let degree = 10;

    // Generate the setup parameters
    let pp = PCS::setup(degree, false, &mut rng).unwrap();

    // Generate setup parameter
    let (ck, vk) = PCS::trim(pp, degree).unwrap();
    let u256_vec = curve_to_u256_vec(vk.beta_h);
    let g2tau_point = G2Point {
        X: [u256_vec[1], u256_vec[0]],
        Y: [u256_vec[3], u256_vec[2]],
    };

    // Deploy the contract
    let contract = KZGVerifier::deploy(&provider, g2tau_point).await?;
    println!("Deployed contract at address: {}", contract.address());

    // Generate a random polynomial and commit to it
    let poly = UniPoly::rand(degree, &mut rng);
    let (com, rand) = PCS::commit(&ck, &poly, None, None).unwrap();
    let u256_vec = curve_to_u256_vec(com);
    let com = G1Point {
        X: u256_vec[0],
        Y: u256_vec[1],
    };
    let builder = contract.commit(com);
    let _ = builder.send().await?.watch().await?;
    println!(
        "Commited X: {}",
        contract.commitment().call().await?.X.to_string()
    );

    // Open the polynomial at a random point
    let point = Fr::rand(&mut rng);
    let value = poly.evaluate(&point);
    let proof = PCS::open(&ck, &poly, point, &rand).unwrap();

    // Verify the proof
    let z = scalar_to_u256(point);
    let y = scalar_to_u256(value);
    let u256_vec = curve_to_u256_vec(proof.w);
    let proof = G1Point {
        X: u256_vec[0],
        Y: u256_vec[1],
    };

    let result = contract.verify(z, y, proof).call().await?._0;
    assert!(result);

    Ok(())
}
