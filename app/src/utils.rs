use alloy::primitives::U256;
use ark_bn254::{Fq, Fr};
use ark_ff::ToConstraintField;
use ark_ff::{BigInteger, PrimeField};
use rand::Rng;

pub fn curve_to_u256_vec<T: ToConstraintField<Fq>>(input: T) -> Vec<U256> {
    input
        .to_field_elements()
        .unwrap()
        .iter()
        .map(|fe| U256::from_be_slice(&fe.into_bigint().to_bytes_be()))
        .collect()
}

pub fn scalar_to_u256(input: Fr) -> U256 {
    U256::from_be_slice(&input.into_bigint().to_bytes_be())
}

pub fn random_bytes32() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..32).map(|_| rng.gen()).collect()
}

pub fn to_bytes32_array(value: &Vec<u8>) -> [u8; 32] {
    let mut array = [0u8; 32];
    let bytes = &value[..array.len()];
    array.copy_from_slice(bytes);
    array
}
