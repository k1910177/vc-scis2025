use ark_ec::pairing::Pairing;
use ark_poly_commit::kzg10::Commitment;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use derivative::Derivative;

/// `Proof` is an evaluation proof that is output by `KZG10::open`.
#[derive(Derivative, CanonicalSerialize, CanonicalDeserialize)]
#[derivative(
    Default(bound = ""),
    Hash(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub struct Proof<E: Pairing> {
    pub d: Commitment<E>,
    pub pi: E::G1Affine,
}
