use ark_bn254::{Bn254, Fr};
use ark_ff::UniformRand;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
use ark_poly_commit::kzg10::Commitment;
use ark_std::rand::thread_rng;
use keccak_asm::Keccak256;
use kzg_commitment::{kzg::KZG, multiproof::KZGMultiproof};

type UniPoly = DensePolynomial<Fr>;
type PCS = KZG<Bn254, UniPoly>;
type PCSMultiproof = KZGMultiproof<Bn254, UniPoly, Keccak256>;

#[test]
fn it_tests_two_polys() {
    // Config
    let mut rng = thread_rng();
    let degree = 10;

    // Setup
    let pp = PCS::setup(degree, false, &mut rng).unwrap();
    let (ck, vk) = PCS::trim(pp, degree).unwrap();

    // Generate two random polys and commmit
    let poly1 = UniPoly::rand(degree, &mut rng);
    let poly2 = UniPoly::rand(degree, &mut rng);
    let (com1, _) = PCS::commit(&ck, &poly1, None, None).unwrap();
    let (com2, _) = PCS::commit(&ck, &poly2, None, None).unwrap();

    // Evaluate points
    let point1 = Fr::rand(&mut rng);
    let value1 = poly1.evaluate(&point1);
    let point2 = Fr::rand(&mut rng);
    let value2 = poly2.evaluate(&point2);

    // Prove
    let proof = PCSMultiproof::prove(
        &ck,
        &[poly1, poly2],
        &[com1, com2],
        &[point1, point2],
        &[value1, value2],
    )
    .unwrap();

    // Verify
    let result = PCSMultiproof::verify(
        &vk,
        &[com1, com2],
        &[point1, point2],
        &[value1, value2],
        &proof,
    )
    .unwrap();

    assert!(result);
}

#[test]
fn it_tests_multiple_polys() {
    // Config
    let mut rng = thread_rng();
    let degree = 10;
    let num = 10;

    // Setup
    let pp = PCS::setup(degree, false, &mut rng).unwrap();
    let (ck, vk) = PCS::trim(pp, degree).unwrap();

    // Generate random polys and commmit
    let polys: Vec<UniPoly> = (0..num).map(|_| UniPoly::rand(degree, &mut rng)).collect();
    let coms: Vec<Commitment<Bn254>> = polys
        .iter()
        .map(|poly| PCS::commit(&ck, &poly, None, None).unwrap().0)
        .collect();

    // Evaluate points
    let points: Vec<Fr> = (0..num).map(|_| Fr::rand(&mut rng)).collect();
    let values: Vec<Fr> = polys
        .iter()
        .zip(points.iter())
        .map(|(poly, point)| poly.evaluate(point))
        .collect();

    // Prove
    let proof = PCSMultiproof::prove(
        &ck,
        &polys.as_slice(),
        &coms.as_slice(),
        &points.as_slice(),
        &values.as_ref(),
    )
    .unwrap();

    // Verify
    let result = PCSMultiproof::verify(
        &vk,
        &coms.as_slice(),
        &points.as_slice(),
        &values.as_slice(),
        &proof,
    )
    .unwrap();

    assert!(result);
}

#[test]
fn it_tests_high_degree_polys() {
    // Config
    let mut rng = thread_rng();
    let degree = 256;
    let num = 100;

    // Setup
    let pp = PCS::setup(degree, false, &mut rng).unwrap();
    let (ck, vk) = PCS::trim(pp, degree).unwrap();

    // Generate random polys and commmit
    let polys: Vec<UniPoly> = (0..num).map(|_| UniPoly::rand(degree, &mut rng)).collect();
    let coms: Vec<Commitment<Bn254>> = polys
        .iter()
        .map(|poly| PCS::commit(&ck, &poly, None, None).unwrap().0)
        .collect();

    // Evaluate points
    let points: Vec<Fr> = (0..num).map(|_| Fr::rand(&mut rng)).collect();
    let values: Vec<Fr> = polys
        .iter()
        .zip(points.iter())
        .map(|(poly, point)| poly.evaluate(point))
        .collect();

    // Prove
    let proof = PCSMultiproof::prove(
        &ck,
        &polys.as_slice(),
        &coms.as_slice(),
        &points.as_slice(),
        &values.as_ref(),
    )
    .unwrap();

    // Verify
    let result = PCSMultiproof::verify(
        &vk,
        &coms.as_slice(),
        &points.as_slice(),
        &values.as_slice(),
        &proof,
    )
    .unwrap();

    assert!(result);
}
