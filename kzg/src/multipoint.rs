use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup, VariableBaseMSM};
use ark_ff::{BigInteger, Field, One, PrimeField, ToConstraintField, Zero};
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::{
    kzg10::{Commitment, Powers, VerifierKey},
    Error,
};
use ark_std::{
    ops::Add as StdAdd, ops::Div as StdDiv, ops::Mul as StdMul, ops::Sub as StdSub, vec::Vec,
};
use itertools::izip;
use keccak_asm::Digest;
use std::{marker::PhantomData, ops::Neg};

use crate::{data_structures::Proof, kzg::KZG};

pub struct KZGMultipoint<E: Pairing, P: DenseUVPolynomial<E::ScalarField>, D: Digest> {
    _engine: PhantomData<E>,
    _poly: PhantomData<P>,
    _hash: PhantomData<D>,
}

impl<E, P, D> KZGMultipoint<E, P, D>
where
    E: Pairing,
    E::G1Affine: ToConstraintField<<E::TargetField as Field>::BasePrimeField>,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    D: Digest,
    for<'a, 'b> &'a P: StdAdd<&'b P, Output = P>,
    for<'a, 'b> &'a P: StdSub<&'b P, Output = P>,
    for<'a, 'b> &'a P: StdMul<&'b P, Output = P>,
    for<'a, 'b> &'a P: StdDiv<&'b P, Output = P>,
{
    pub fn prove(
        powers: &Powers<E>,
        polys: &[P],
        coms: &[Commitment<E>],
        points_slice: &[&[P::Point]],
        values_slice: &[&[E::ScalarField]],
    ) -> Result<Proof<E>, Error> {
        let eval_polys = Self::compute_eval_polys(points_slice, values_slice);
        let zero_polys = Self::compute_zero_polys(points_slice);

        let r = Self::compute_r(coms);
        let g_poly = Self::compute_witness_poly(polys, &eval_polys, &zero_polys, &r);
        let (d_com, _) = KZG::<E, P>::commit(powers, &g_poly, None, None)?;
        let t = Self::compute_t(&d_com, &r);
        let (h_poly, y) = Self::compute_h_y(polys, &eval_polys, &zero_polys, &r, &t);
        let pi_poly = Self::compute_pi_poly(&h_poly, &g_poly, &y, &t);
        let (pi, _) = KZG::<E, P>::commit(powers, &pi_poly, None, None)?;

        Ok(Proof { pi: pi.0, d: d_com })
    }

    pub fn verify(
        vk: &VerifierKey<E>,
        coms: &[Commitment<E>],
        points_slice: &[&[P::Point]],
        values_slice: &[&[E::ScalarField]],
        proof: &Proof<E>,
    ) -> Result<bool, Error> {
        let eval_polys = Self::compute_eval_polys(points_slice, values_slice);
        let zero_polys = Self::compute_zero_polys(points_slice);

        let r = Self::compute_r(coms);
        let t = Self::compute_t(&proof.d, &r);
        let (e, y) = Self::compute_e_y(coms, &eval_polys, &zero_polys, &r, &t);
        let result = Self::check_pairing(vk, &e, &proof.d, &y, &proof.pi, &t);

        Ok(result)
    }

    fn compute_eval_polys(
        points_slice: &[&[P::Point]],
        values_slice: &[&[E::ScalarField]],
    ) -> Vec<P> {
        let interpolate_poly = |points: &[P::Point], values: &[E::ScalarField]| {
            let mut result = P::zero();
            for (i, (x_i, y_i)) in izip!(points, values).enumerate() {
                let mut l_i = P::from_coefficients_slice(&[E::ScalarField::one()]);
                for (j, x_j) in points.iter().enumerate() {
                    if i != j {
                        let denom = P::from_coefficients_slice(&[*x_i - *x_j]);
                        let numer = P::from_coefficients_slice(&[-*x_j, E::ScalarField::one()]);
                        l_i = l_i.mul(&numer.div(&denom))
                    }
                }
                let scaled_l_i = l_i.mul(&P::from_coefficients_slice(&[*y_i]));
                result += &scaled_l_i;
            }
            result
        };

        points_slice
            .iter()
            .zip(values_slice.iter())
            .map(|(points, values)| interpolate_poly(points, values))
            .collect()
    }

    fn compute_zero_polys(points_slice: &[&[P::Point]]) -> Vec<P> {
        let compute_zero_poly = |roots: &[P::Point]| {
            roots
                .iter()
                .map(|root| P::from_coefficients_slice(&[-*root, E::ScalarField::one()]))
                .reduce(|acc, poly| acc.mul(&poly))
                .unwrap_or_else(|| P::from_coefficients_slice(&[E::ScalarField::one()]))
        };

        points_slice
            .iter()
            .map(|points| compute_zero_poly(&points))
            .collect()
    }

    fn compute_r(coms: &[Commitment<E>]) -> E::ScalarField {
        let mut hasher = D::new();
        for com in coms {
            // Append com
            let field_elements = com.to_field_elements().unwrap();
            for element in field_elements {
                if element.is_zero() {
                    continue;
                }
                let bytes: Vec<u8> = element.into_bigint().to_bytes_be();
                hasher.update(&bytes.as_slice());
            }
        }

        // Calculate hash
        let result = hasher.finalize();
        E::ScalarField::from_be_bytes_mod_order(&result)
    }

    fn compute_t(d_com: &Commitment<E>, r: &E::ScalarField) -> E::ScalarField {
        let mut hasher = D::new();
        let field_elements = d_com.to_field_elements().unwrap();
        for element in field_elements {
            if element.is_zero() {
                continue;
            }
            let bytes: Vec<u8> = element.into_bigint().to_bytes_be();
            hasher.update(&bytes.as_slice());
        }

        // Append point
        let bytes = r.into_bigint().to_bytes_be();
        hasher.update(&bytes.as_slice());

        // Calculate hash
        let result = hasher.finalize();
        E::ScalarField::from_be_bytes_mod_order(&result)
    }

    fn compute_witness_poly(
        polys: &[P],
        eval_polys: &[P],
        zero_polys: &[P],
        r: &E::ScalarField,
    ) -> P {
        let mut g_poly = P::zero();
        for (j, (poly, eval_poly, zero_poly)) in izip!(polys, eval_polys, zero_polys).enumerate() {
            let r_j = P::from_coefficients_slice(&[r.pow(&[j as u64])]);
            let dividend = poly.sub(&eval_poly).mul(&r_j);
            let g_frac = dividend.div(&zero_poly);
            g_poly = g_poly.add(g_frac)
        }

        g_poly
    }

    fn compute_h_y(
        polys: &[P],
        eval_polys: &[P],
        zero_polys: &[P],
        r: &E::ScalarField,
        t: &E::ScalarField,
    ) -> (P, E::ScalarField) {
        let mut y = E::ScalarField::zero();
        let mut h_poly = P::zero();

        for (j, (poly, eval_poly, zero_poly)) in izip!(polys, eval_polys, zero_polys).enumerate() {
            let r_j = r.pow(&[j as u64]);
            let divisor: E::ScalarField = zero_poly.evaluate(&t);
            let y_frac = r_j.mul(&eval_poly.evaluate(&t)).div(&divisor);
            y = y.add(y_frac);

            let r_j = P::from_coefficients_slice(&[r_j]);
            let divisor = P::from_coefficients_slice(&[divisor]);
            let h_frac = r_j.mul(poly).div(&divisor);
            h_poly = h_poly.add(h_frac);
        }

        (h_poly, y)
    }

    fn compute_e_y(
        coms: &[Commitment<E>],
        eval_polys: &[P],
        zero_polys: &[P],
        r: &E::ScalarField,
        t: &E::ScalarField,
    ) -> (E::G1Affine, E::ScalarField) {
        let mut e_scalars = Vec::new();
        let mut y = E::ScalarField::zero();
        for (j, (eval_poly, zero_poly)) in izip!(eval_polys, zero_polys).enumerate() {
            let r_j = r.pow(&[j as u64]);
            let divisor: E::ScalarField = zero_poly.evaluate(&t);
            let y_frac = r_j.mul(&eval_poly.evaluate(&t)).div(&divisor);
            y = y.add(y_frac);

            let e_scaler = r_j.div(divisor);
            e_scalars.push(e_scaler.into_bigint());
        }

        let coms_g1: Vec<E::G1Affine> = coms.iter().map(|com| com.0).collect();
        let e = <E::G1 as VariableBaseMSM>::msm_bigint(&coms_g1, e_scalars.as_slice());

        (e.into_affine(), y)
    }

    fn compute_pi_poly(h_poly: &P, g_poly: &P, y: &E::ScalarField, t: &E::ScalarField) -> P {
        let y_poly = P::from_coefficients_slice(&[*y]);
        let dividend = h_poly.sub(g_poly).sub(&y_poly);
        let divisor = P::from_coefficients_slice(&[-*t, E::ScalarField::one()]);
        dividend.div(&divisor)
    }

    fn check_pairing(
        vk: &VerifierKey<E>,
        e: &E::G1Affine,
        d_com: &Commitment<E>,
        y: &E::ScalarField,
        pi: &E::G1Affine,
        t: &E::ScalarField,
    ) -> bool {
        let lhs_g1: E::G1 = e.sub(d_com.0).sub(vk.g.mul(y)).add(pi.mul(t));
        let rhs_g1: E::G1 = pi.into_group().neg();
        E::multi_pairing([lhs_g1, rhs_g1], [vk.h, vk.beta_h])
            .0
            .is_one()
    }
}
