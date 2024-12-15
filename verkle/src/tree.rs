use ark_bn254::{Bn254, Fr};
use ark_ec::pairing::Pairing;
use ark_ff::{BigInteger, PrimeField, ToConstraintField};
use ark_poly::{
    univariate::DensePolynomial, DenseUVPolynomial, EvaluationDomain, GeneralEvaluationDomain,
    Polynomial,
};
use ark_poly_commit::{
    kzg10::{Commitment as KZGCommitment, Powers, VerifierKey},
    Error,
};
use ark_std::{rand::RngCore, Zero};
use keccak_asm::{Digest, Keccak256};
use kzg_commitment::{kzg::KZG, multiproof::KZGMultiproof};

use crate::data_structures::Proof;

type Poly = DensePolynomial<Fr>;
type Commitment = KZGCommitment<Bn254>;
type PCS = KZG<Bn254, Poly>;
type PCSMultiproof = KZGMultiproof<Bn254, Poly, Keccak256>;
type Domain = GeneralEvaluationDomain<Fr>;

#[derive(Debug, Clone)]
pub enum Node {
    Internal {
        children: Vec<Node>,
        value: Commitment,
        poly: Poly,
    },
    Leaf {
        value: Fr,
    },
}

#[derive(Debug)]
pub struct VerkleTree<'a> {
    pub root: Option<Node>,
    pub height: Option<usize>,
    pub width: usize,
    pub ck: Powers<'a, Bn254>,
    pub vk: VerifierKey<Bn254>,
}

impl VerkleTree<'_> {
    pub fn setup<R: RngCore>(width: usize, rng: &mut R) -> Self {
        let pp = PCS::setup(width, false, rng).unwrap();
        let (ck, vk) = PCS::trim(pp, width).unwrap();

        VerkleTree {
            root: None,
            height: None,
            width,
            ck,
            vk,
        }
    }

    pub fn print_tree(&self) {
        self.print_node(self.root.as_ref().unwrap(), 0);
    }

    pub fn root_hash(&self) -> Fr {
        match self.root.as_ref().unwrap() {
            Node::Internal { value, .. } => Self::hash_g1(&value.0),
            _ => Self::hash_fr(&Fr::from(0)),
        }
    }

    pub fn commit(&mut self, values: &[Fr]) {
        let leaf_nodes: Vec<Node> = values
            .iter()
            .map(|value| Node::Leaf {
                value: value.clone(),
            })
            .collect();

        self.root = Some(self.build_recursive(leaf_nodes));
        self.height = Some(ceil_log_base(self.width, values.len()));
    }

    pub fn open(&self, index: usize) -> Result<(Fr, Proof<Bn254>), Error> {
        let tree_path = Self::compute_path(index, self.height.unwrap(), self.width);
        let domain = Domain::new(self.width).unwrap();

        let mut final_value = Fr::from(0);
        let mut polys = Vec::<Poly>::new();
        let mut coms = Vec::<Commitment>::new();
        let mut points = Vec::<Fr>::new();
        let mut values = Vec::<Fr>::new();
        let mut current_node = self.root.as_ref().unwrap();
        for (_, &path_index) in tree_path.iter().enumerate() {
            match current_node {
                Node::Internal {
                    children,
                    value,
                    poly,
                } => {
                    let next_node = children.get(path_index).unwrap();
                    match next_node {
                        Node::Internal { value, .. } => values.push(Self::hash_g1(&value.0)),
                        Node::Leaf { value } => {
                            final_value = value.clone();
                            values.push(Self::hash_fr(value))
                        }
                    }
                    polys.push(poly.clone());
                    coms.push(value.clone());
                    points.push(domain.element(path_index));
                    current_node = next_node;
                }
                _ => (),
            }
        }

        let multi_proof = PCSMultiproof::prove(&self.ck, &polys, &coms, &points, &values)?;
        Ok((final_value, Proof { coms, multi_proof }))
    }

    pub fn verify(
        &self,
        index: usize,
        value: Fr,
        multi_proof: Proof<Bn254>,
    ) -> Result<bool, Error> {
        let tree_path = Self::compute_path(index, self.height.unwrap(), self.width);
        let domain = Domain::new(self.width).unwrap();
        let points: Vec<Fr> = tree_path
            .iter()
            .map(|p| domain.element(p.clone()))
            .collect();
        let mut values = Vec::<Fr>::new();
        for i in 1..multi_proof.coms.len() {
            let inner_value = Self::hash_g1(&multi_proof.coms.get(i).unwrap().0);
            values.push(inner_value)
        }
        values.push(Self::hash_fr(&value));
        let result = PCSMultiproof::verify(
            &self.vk,
            &multi_proof.coms,
            &points,
            &values,
            &multi_proof.multi_proof,
        )?;
        Ok(result)
    }

    fn compute_path(index: usize, height: usize, width: usize) -> Vec<usize> {
        let mut n = index.clone();
        let mut path = vec![0; height];
        for i in (0..height).rev() {
            if n == 0 {
                break;
            }
            path[i] = n % width;
            n /= width;
        }
        path
    }

    fn build_recursive(&self, nodes: Vec<Node>) -> Node {
        if nodes.len() <= self.width {
            let poly = Self::gen_poly_from_nodes(&nodes, self.width);
            let (com, _) = PCS::commit(&self.ck, &poly, None, None).unwrap();
            return Node::Internal {
                children: nodes,
                value: com,
                poly,
            };
        }

        let mut parent_nodes = Vec::<Node>::new();
        for i in (0..nodes.len()).step_by(self.width) {
            let from_index = i;
            let to_index = if i + self.width > nodes.len() {
                nodes.len()
            } else {
                i + self.width
            };
            let child_nodes: Vec<Node> = nodes[from_index..to_index].to_vec();
            let poly = Self::gen_poly_from_nodes(&child_nodes, self.width);
            let (com, _) = PCS::commit(&self.ck, &poly, None, None).unwrap();
            let parent_node = Node::Internal {
                children: child_nodes,
                value: com,
                poly,
            };
            parent_nodes.push(parent_node);
        }

        self.build_recursive(parent_nodes)
    }

    fn hash_g1(g1: &<Bn254 as Pairing>::G1Affine) -> Fr {
        let field_elements = g1.to_field_elements().unwrap();
        let mut hasher = Keccak256::new();
        for element in field_elements {
            if element.is_zero() {
                continue;
            }
            let bytes: Vec<u8> = element.into_bigint().to_bytes_be();
            hasher.update(&bytes.as_slice());
        }

        let result = hasher.finalize();
        Fr::from_be_bytes_mod_order(&result.as_slice())
    }

    fn hash_fr(fr: &Fr) -> Fr {
        let bytes: Vec<u8> = fr.into_bigint().to_bytes_be();
        let result = Keccak256::digest(bytes.as_slice());
        Fr::from_be_bytes_mod_order(&result.as_slice())
    }

    fn gen_poly_from_nodes(nodes: &Vec<Node>, domain_size: usize) -> Poly {
        let evals: Vec<Fr> = nodes
            .iter()
            .map(|node| match node {
                Node::Internal { value, .. } => Self::hash_g1(&value.0),
                Node::Leaf { value } => Self::hash_fr(value),
            })
            .collect();
        let domain = Domain::new(domain_size).unwrap();
        let coeffs = domain.ifft(&evals);
        Poly::from_coefficients_vec(coeffs)
    }

    fn print_node(&self, node: &Node, level: usize) {
        match node {
            Node::Internal {
                children,
                value,
                poly,
            } => {
                println!(
                    "{}Internal Node - Value: {:?}, Polynomial deg: {:?}, Children size: {}",
                    " ".repeat(level * 2),
                    value,
                    poly.degree(),
                    children.len()
                );
                for child in children {
                    self.print_node(child, level + 1);
                }
            }
            Node::Leaf { value } => {
                println!("{}Leaf Node - Value: {:?}", " ".repeat(level * 2), value);
            }
        }
    }
}

pub fn ceil_log_base(base: usize, n: usize) -> usize {
    if base <= 1 || n == 0 {
        panic!("Invalid base or n")
    }

    let base_f64 = base as f64;
    let n_f64 = n as f64;
    n_f64.log(base_f64).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_ff::Field;
    use ark_std::test_rng;

    #[test]
    fn roots_of_unity() {
        let domain = Domain::new(3).unwrap();
        let element = domain.element(1);
        println!("Element: {:?}", element);

        let two = element.pow(&[4 as u64]);
        println!("Two: {:?}", two);

        let element = domain.element(4);
        println!("Element: {:?}", element);

        let domain = Domain::new(10).unwrap();
        let element = domain.element(1);
        println!("Element: {:?}", element);
    }

    #[test]
    fn it_instantiates_tree() {
        let mut rng = test_rng();
        let width = 8;
        let tree = VerkleTree::setup(width, &mut rng);
        assert!(tree.height.is_none());
        assert!(tree.root.is_none());
    }

    #[test]
    fn it_commits() {
        let mut rng = test_rng();
        let width = 8;
        let mut tree = VerkleTree::setup(width, &mut rng);

        let vec = vec![Fr::from(0), Fr::from(1), Fr::from(2)];
        tree.commit(vec.as_slice());
    }

    #[test]
    fn it_commits_bigger_vector() {
        let n = 17;
        let width = 4;
        let mut rng = test_rng();
        let mut tree = VerkleTree::setup(width, &mut rng);

        let vec: Vec<Fr> = (0..n).map(Fr::from).collect();
        tree.commit(vec.as_slice());
        tree.print_tree();
    }

    #[test]
    fn it_tests_compute_path() {
        let width = 4;
        let height = 3;
        let index = 2;
        let tree_path = VerkleTree::compute_path(index, height, width);
        println!("Tree path: {:?}", tree_path);
    }

    #[test]
    fn it_computes_path_index() {
        let width = 4;
        let height = 3;
        let index = 2;
        let tree_path = VerkleTree::compute_path(index, height, width);

        let domain = Domain::new(width).unwrap();
        let roots_of_unity = domain.element(1);
        let path_index: Vec<Fr> = tree_path
            .iter()
            .map(|p| domain.element(p.clone()))
            .collect();
        println!("Tree path: {:?}", path_index);
        println!("Roots of unity: {:?}", roots_of_unity);
    }

    #[test]
    fn it_opens() {
        let n = 17;
        let width = 4;
        let mut rng = test_rng();
        let mut tree = VerkleTree::setup(width, &mut rng);

        let vec: Vec<Fr> = (1..=n).map(Fr::from).collect();
        tree.commit(vec.as_slice());

        let index = 2;
        let (value, _) = tree.open(index).unwrap();
        unsafe {
            assert_eq!(value, vec.get_unchecked(index).clone());
        }
    }

    #[test]
    fn it_verifies() {
        let n = 17;
        let width = 4;
        let mut rng = test_rng();
        let mut tree = VerkleTree::setup(width, &mut rng);

        let vec: Vec<Fr> = (1..=n).map(Fr::from).collect();
        tree.commit(vec.as_slice());

        let index = 2;
        let (value, multi_proof) = tree.open(index).unwrap();
        let result = tree.verify(index, value, multi_proof);
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn it_rejects_false_statement() {
        let n = 17;
        let width = 4;
        let mut rng = test_rng();
        let mut tree = VerkleTree::setup(width, &mut rng);

        let vec: Vec<Fr> = (1..=n).map(Fr::from).collect();
        tree.commit(vec.as_slice());

        let index = 2;
        let (value, multi_proof) = tree.open(index).unwrap();
        let result = tree.verify(index, value + Fr::from(1), multi_proof);
        assert_eq!(result.unwrap(), false);
    }
}
