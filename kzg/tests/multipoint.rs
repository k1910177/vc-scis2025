use ark_bn254::{Bn254, Fr};
use ark_ff::UniformRand;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
use ark_poly_commit::kzg10::Commitment;
use ark_std::rand::{thread_rng, Rng};
use keccak_asm::Keccak256;
use kzg_commitment::{kzg::KZG, multipoint::KZGMultipoint};

type UniPoly = DensePolynomial<Fr>;
type PCS = KZG<Bn254, UniPoly>;
type PCSMultiproof = KZGMultipoint<Bn254, UniPoly, Keccak256>;

#[test]
fn it_tests_multiple_points() {
    // Config
    let mut rng = thread_rng();
    let degree: usize = 100;
    let num_poly: usize = 10;
    let max_points: usize = 10;

    // Setup
    let pp = PCS::setup(degree, false, &mut rng).unwrap();
    let (ck, vk) = PCS::trim(pp, degree).unwrap();

    // Generate random polys and commmit
    let polys: Vec<UniPoly> = (0..num_poly)
        .map(|_| UniPoly::rand(degree, &mut rng))
        .collect();
    let coms: Vec<Commitment<Bn254>> = polys
        .iter()
        .map(|poly| PCS::commit(&ck, &poly, None, None).unwrap().0)
        .collect();

    // Evaluate points
    let mut points_vec: Vec<Vec<Fr>> = Vec::new();
    let mut values_vec: Vec<Vec<Fr>> = Vec::new();
    for poly in polys.iter() {
        let num_points = rng.gen_range(1..=max_points);
        let points: Vec<Fr> = (0..num_points).map(|_| Fr::rand(&mut rng)).collect();
        let values: Vec<Fr> = points.iter().map(|point| poly.evaluate(point)).collect();
        points_vec.push(points);
        values_vec.push(values);
    }

    let points_vec: Vec<&[Fr]> = points_vec.iter().map(|points| points.as_slice()).collect();
    let values_vec: Vec<&[Fr]> = values_vec.iter().map(|values| values.as_slice()).collect();

    // Prove
    let proof = PCSMultiproof::prove(
        &ck,
        &polys.as_slice(),
        &coms.as_slice(),
        &points_vec.as_slice(),
        &values_vec.as_slice(),
    )
    .unwrap();

    // Verify
    let result = PCSMultiproof::verify(
        &vk,
        &coms.as_slice(),
        &points_vec.as_slice(),
        &values_vec.as_slice(),
        &proof,
    )
    .unwrap();

    assert!(result);
}
