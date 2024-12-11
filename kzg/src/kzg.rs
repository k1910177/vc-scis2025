use std::{collections::BTreeMap, marker::PhantomData};
use ark_ec::{pairing::Pairing, scalar_mul::ScalarMul, CurveGroup, PrimeGroup};
use ark_ff::UniformRand;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::{
    kzg10::{self, Commitment, Powers, Proof, Randomness, UniversalParams, VerifierKey},
    Error,
};
use ark_std::{ops::Div, ops::Mul, rand::RngCore, One, Zero};

pub struct KZG<E: Pairing, P: DenseUVPolynomial<E::ScalarField>> {
    _engine: PhantomData<E>,
    _poly: PhantomData<P>,
}

impl<E, P> KZG<E, P>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    pub fn setup<R: RngCore>(
        max_degree: usize,
        produce_g2_powers: bool,
        rng: &mut R,
    ) -> Result<UniversalParams<E>, Error> {
        if max_degree < 1 {
            return Err(Error::DegreeIsZero);
        }
        let beta = E::ScalarField::rand(rng);
        let g = E::G1::generator();
        let gamma_g = E::G1::zero();
        let h = E::G2::generator();

        // powers_of_beta = [1, b, ..., b^(max_degree + 1)], len = max_degree + 2
        let mut powers_of_beta = vec![E::ScalarField::one()];
        let mut cur = beta;
        for _ in 0..=max_degree {
            powers_of_beta.push(cur);
            cur *= &beta;
        }

        let powers_of_g = g.batch_mul(&powers_of_beta[0..max_degree + 1]);

        // Use the entire `powers_of_beta`, since we want to be able to support
        // up to D queries.
        let powers_of_gamma_g =
            gamma_g.batch_mul(&powers_of_beta).into_iter().enumerate().collect();

        let neg_powers_of_h = if produce_g2_powers {
            let mut neg_powers_of_beta = vec![E::ScalarField::one()];
            let mut cur = E::ScalarField::one() / &beta;
            for _ in 0..max_degree {
                neg_powers_of_beta.push(cur);
                cur /= &beta;
            }

            h.batch_mul(&neg_powers_of_beta).into_iter().enumerate().collect()
        } else {
            BTreeMap::new()
        };

        let h = h.into_affine();
        let beta_h = h.mul(beta).into_affine();
        let prepared_h = h.into();
        let prepared_beta_h = beta_h.into();

        let pp = UniversalParams {
            powers_of_g,
            powers_of_gamma_g,
            h,
            beta_h,
            neg_powers_of_h,
            prepared_h,
            prepared_beta_h,
        };
        Ok(pp)
    }

    pub fn trim<'a>(
        pp: UniversalParams<E>,
        mut supported_degree: usize,
    ) -> Result<(Powers<'a, E>, VerifierKey<E>), Error> {
        if supported_degree == 1 {
            supported_degree += 1;
        }
        let powers_of_g = pp.powers_of_g[..=supported_degree].to_vec();
        let powers_of_gamma_g = (0..=supported_degree).map(|i| pp.powers_of_gamma_g[&i]).collect();

        let powers = Powers {
            powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
            powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
        };
        let vk = VerifierKey {
            g: pp.powers_of_g[0],
            gamma_g: pp.powers_of_gamma_g[&0],
            h: pp.h,
            beta_h: pp.beta_h,
            prepared_h: pp.prepared_h.clone(),
            prepared_beta_h: pp.prepared_beta_h.clone(),
        };
        Ok((powers, vk))
    }

    pub fn commit(
        powers: &Powers<E>,
        polynomial: &P,
        hiding_bound: Option<usize>,
        rng: Option<&mut dyn RngCore>,
    ) -> Result<(Commitment<E>, Randomness<E::ScalarField, P>), Error> {
        kzg10::KZG10::commit(powers, polynomial, hiding_bound, rng)
    }

    pub fn open<'a>(
        powers: &Powers<E>,
        p: &P,
        point: P::Point,
        rand: &Randomness<E::ScalarField, P>,
    ) -> Result<Proof<E>, Error> {
        kzg10::KZG10::open(powers, p, point, rand)
    }

    pub fn check(
        vk: &VerifierKey<E>,
        comm: &Commitment<E>,
        point: E::ScalarField,
        value: E::ScalarField,
        proof: &Proof<E>,
    ) -> Result<bool, Error> {
        kzg10::KZG10::check(vk, comm, point, value, proof)
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_camel_case_types)]
    use crate::kzg::*;
    use ark_bn254::Bn254;
    use ark_ff::ToConstraintField;
    use ark_poly::univariate::DensePolynomial;
    use ark_std::test_rng;

    type UniPoly = DensePolynomial<<Bn254 as Pairing>::ScalarField>;
    type PCS = KZG<Bn254, UniPoly>;

    #[test]
    fn test_setup_param() {
        let rng = &mut test_rng();
        let degree = 4;
        let pp = PCS::setup(degree, false, rng).unwrap();
        let (_, vk) = PCS::trim(pp, degree).unwrap();

        println!("vk.h: {:?}", vk.h.to_field_elements());
        println!("vk.g: {:?}", vk.g.to_field_elements());
    }
}
