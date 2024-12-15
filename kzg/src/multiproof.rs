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

pub struct KZGMultiproof<E: Pairing, P: DenseUVPolynomial<E::ScalarField>, D: Digest> {
    _engine: PhantomData<E>,
    _poly: PhantomData<P>,
    _hash: PhantomData<D>,
}

impl<E, P, D> KZGMultiproof<E, P, D>
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
        points: &[P::Point],
        values: &[E::ScalarField],
    ) -> Result<Proof<E>, Error> {
        let r = Self::compute_r(coms, points, values);
        let g_poly = Self::compute_witness_poly(polys, points, values, &r);
        let (d_com, _) = KZG::<E, P>::commit(powers, &g_poly, None, None)?;
        let t = Self::compute_t(&d_com, &r);
        let (h_poly, y) = Self::compute_h_y(polys, points, values, &r, &t);
        let pi_poly = Self::compute_pi_poly(&h_poly, &g_poly, &y, &t);
        let (pi, _) = KZG::<E, P>::commit(powers, &pi_poly, None, None)?;

        Ok(Proof { pi: pi.0, d: d_com })
    }

    pub fn verify(
        vk: &VerifierKey<E>,
        coms: &[Commitment<E>],
        points: &[P::Point],
        values: &[E::ScalarField],
        proof: &Proof<E>,
    ) -> Result<bool, Error> {
        let r = Self::compute_r(coms, points, values);
        let t = Self::compute_t(&proof.d, &r);
        let (e, y) = Self::compute_e_y(coms, points, values, &r, &t);
        let result = Self::check_pairing(vk, &e, &proof.d, &y, &proof.pi, &t);
        Ok(result)
    }

    fn compute_r(
        coms: &[Commitment<E>],
        points: &[P::Point],
        values: &[E::ScalarField],
    ) -> E::ScalarField {
        let mut hasher = D::new();
        for (com, point, value) in izip!(coms, points, values) {
            // Append com
            let field_elements = com.to_field_elements().unwrap();
            for element in field_elements {
                if element.is_zero() {
                    continue;
                }
                let bytes: Vec<u8> = element.into_bigint().to_bytes_be();
                hasher.update(&bytes.as_slice());
            }

            // Append point
            let bytes = point.into_bigint().to_bytes_be();
            hasher.update(&bytes.as_slice());

            // Append value
            let bytes = value.into_bigint().to_bytes_be();
            hasher.update(&bytes.as_slice());
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
        points: &[P::Point],
        values: &[E::ScalarField],
        r: &E::ScalarField,
    ) -> P {
        let mut g_poly = P::zero();
        for (j, (poly, point, value)) in izip!(polys, points, values).enumerate() {
            let r_j = P::from_coefficients_slice(&[r.pow(&[j as u64])]);
            let value = P::from_coefficients_slice(&[*value]);
            let divisor = P::from_coefficients_slice(&[-*point, E::ScalarField::one()]);
            let dividend = poly.sub(&value).mul(&r_j);
            let g_frac = dividend.div(&divisor);

            g_poly = g_poly.add(g_frac)
        }

        g_poly
    }

    fn compute_h_y(
        polys: &[P],
        points: &[P::Point],
        values: &[E::ScalarField],
        r: &E::ScalarField,
        t: &E::ScalarField,
    ) -> (P, E::ScalarField) {
        let mut y = E::ScalarField::zero();
        let mut h_poly = P::zero();

        for (j, (poly, point, value)) in izip!(polys, points, values).enumerate() {
            let r_j = r.pow(&[j as u64]);
            let divisor: E::ScalarField = t.sub(point);
            let y_frac = r_j.mul(value).div(&divisor);
            y = y.add(y_frac);

            let r_j = P::from_coefficients_slice(&[r_j]);
            let divisor = P::from_coefficients_slice(&[t.sub(point)]);
            let h_frac = r_j.mul(poly).div(&divisor);
            h_poly = h_poly.add(h_frac);
        }

        (h_poly, y)
    }

    fn compute_e_y(
        coms: &[Commitment<E>],
        points: &[P::Point],
        values: &[E::ScalarField],
        r: &E::ScalarField,
        t: &E::ScalarField,
    ) -> (E::G1Affine, E::ScalarField) {
        let mut e_scalars = Vec::new();
        let mut y = E::ScalarField::zero();
        for (j, (point, value)) in izip!(points, values).enumerate() {
            let r_j = r.pow(&[j as u64]);
            let divisor: E::ScalarField = t.sub(point);
            let y_frac = r_j.mul(value).div(divisor);
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

#[cfg(test)]
mod tests {
    use ark_bn254::Fr;
    use ark_ff::{BigInteger, PrimeField};
    use keccak_asm::{Digest, Keccak256};

    #[test]
    fn it_tests_hasher() {
        let a = Fr::from(1);
        let b = Fr::from(2);

        let mut hasher = Keccak256::new();
        let bytes = a.into_bigint().to_bytes_be();
        hasher.update(&bytes.as_slice());
        let bytes = b.into_bigint().to_bytes_be();
        hasher.update(&bytes.as_slice());

        let result = hasher.finalize();
        let output = Fr::from_be_bytes_mod_order(&result);
        println!("Output: {:?}", output);
    }

    #[test]
    fn it_hashes_bytes() {
        let mut hasher = Keccak256::new();

        let hex_str = "0x0bda44190efee105577ec4a7e5e8e94c875e4a97ce41e820d82e3424382824f625b89cab6578c387dc140660106c9b44af234b055e21164c854d157753cc187b000000000000000000000000000000000000000000000000000000000000000124a4d6b8fed93c9c959df98a34e30aeac6e2933609b0c555d97bb7367577696e104be95f3efe84619f2dff041454f9829d7fa4003493e4509376824730b65204029bb394cee62bb6484697199a6ca59dca331734436252891203ad02bc4c24e700000000000000000000000000000000000000000000000000000000000000012450f6b7c10686ddeaf46ad3463313973d6d969633692eec67d9b10ac34f682600ea524c58f85af52e63dcaf38890aeff2e01f59d28392cf2c9db9ba56515cfe143d526f841d7b4eb22692b400f14d07d3c2269684d4284f74fb2ed42d151af230644e72e131a029b85045b68181585d2833e84879b9709143e1f593f000000000c620431992bb5a1818e1ef290d79b3c8f39838541f408b4e9d3ff4af71f857";
        let hex_str = &hex_str[2..]; // Remove the "0x" prefix
        let bytes: Vec<u8> = (0..hex_str.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).unwrap())
            .collect();

        println!("Bytes: {:?}", bytes);

        hasher.update(&bytes.as_slice());
        let result = hasher.finalize();
        let output = Fr::from_be_bytes_mod_order(&result);
        println!("Output: {:?}", output);
    }
}
