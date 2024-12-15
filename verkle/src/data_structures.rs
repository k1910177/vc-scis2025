use ark_ec::pairing::Pairing;
use ark_poly_commit::kzg10::Commitment;
use kzg_commitment::data_structures::Proof as KZGCommitmentProof;

#[derive(Debug)]
pub struct Proof<E: Pairing> {
    pub coms: Vec<Commitment<E>>,
    pub multi_proof: KZGCommitmentProof<E>
}
