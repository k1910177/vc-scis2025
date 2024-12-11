use ark_bn254::{Bn254, Fr};
use ark_ec::pairing::Pairing;
use ark_ff::ToConstraintField;
use ark_ff::UniformRand;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
use ark_std::rand::thread_rng;
use kzg_commitment::kzg::KZG;

type UniPoly = DensePolynomial<<Bn254 as Pairing>::ScalarField>;
type PCS = KZG<Bn254, UniPoly>;

fn main() {
    // Initialize the RNG
    let mut rng = thread_rng();

    // Define the degree of the polynomial
    let degree = 10;

    // Generate the setup parameters
    let pp = PCS::setup(degree, false, &mut rng).unwrap();

    let (ck, vk) = PCS::trim(pp, degree).unwrap();

    // Generate a random polynomial
    let poly = UniPoly::rand(degree, &mut rng);

    // Commit to the polynomial
    let (comm, rand) = PCS::commit(&ck, &poly, None, None).unwrap();

    let point = Fr::rand(&mut rng);
    let value = poly.evaluate(&point);
    let proof = PCS::open(&ck, &poly, point, &rand).unwrap();
    let result = PCS::check(&vk, &comm, point, value, &proof).unwrap();

    // Print the commitment
    println!("vk.beta_h: {:?}", vk.beta_h.to_field_elements());
    println!("vk.h: {:?}", vk.h.to_field_elements());
    println!("vk.g: {:?}", vk.g.to_field_elements());
    println!("Commitment: {:?}", comm.to_field_elements());
    println!("Point: {:?}", point);
    println!("Value: {:?}", value);
    println!("Proof: {:?}", proof.w.to_field_elements());
    println!("Result: {:?}", result);
}
