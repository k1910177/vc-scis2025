use keccak_asm::{Digest, Keccak256};

type Hash = [u8; 32];
type Value = Vec<u8>;

#[derive(Debug, Clone)]
pub enum Node {
    Internal { children: Vec<Node>, hash: Hash },
    Leaf { value: Option<Value>, hash: Hash },
}

#[derive(Debug)]
pub struct MerkleTree {
    pub root: Option<Node>,
    pub height: Option<usize>,
    pub k: usize,
}

#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub hashes: Vec<Hash>,
}

impl MerkleTree {
    pub fn setup(k: usize) -> Self {
        Self {
            root: None,
            height: None,
            k,
        }
    }

    pub fn commit(&mut self, values: &[&Value]) {
        let leaf_nodes: Vec<Node> = values
            .into_iter()
            .map(|value| {
                let hash = Keccak256::digest(value).into();
                Node::Leaf {
                    value: Some(value.to_vec()),
                    hash,
                }
            })
            .collect();

        self.root = Some(self.build_recursive(leaf_nodes));
        self.height = Some(ceil_log_base(self.k, values.len()));
    }

    pub fn root_hash(&self) -> Option<[u8; 32]> {
        match &self.root {
            Some(Node::Internal { hash, .. }) | Some(Node::Leaf { hash, .. }) => Some(*hash),
            None => None,
        }
    }

    fn build_recursive(&self, mut nodes: Vec<Node>) -> Node {
        if nodes.len() <= self.k {
            let children = Self::extend_nodes(&nodes, self.k);
            let hash = Self::hash_nodes(&children);
            return Node::Internal { children, hash };
        }

        let mut parent_nodes = Vec::new();
        while !nodes.is_empty() {
            let chunk: Vec<Node> = nodes.drain(0..nodes.len().min(self.k)).collect();
            let children = Self::extend_nodes(&chunk, self.k);
            let hash = Self::hash_nodes(&children);
            parent_nodes.push(Node::Internal { children, hash });
        }

        self.build_recursive(parent_nodes)
    }

    fn extend_nodes(nodes: &Vec<Node>, k: usize) -> Vec<Node> {
        let mut extended_nodes: Vec<Node> = Vec::new();
        let mut latest_node_hash: &Hash = &[0; 32];
        let mut is_leaf: Option<bool> = None;
        for i in 0..k {
            if i < nodes.len() {
                let node = nodes.get(i).unwrap();
                match node {
                    Node::Internal { hash, .. } => {
                        is_leaf = Some(false);
                        latest_node_hash = hash;
                    }
                    Node::Leaf { hash, .. } => {
                        is_leaf = Some(true);
                        latest_node_hash = hash;
                    }
                }
                extended_nodes.push(node.clone());
            } else {
                if is_leaf.unwrap() == true {
                    extended_nodes.push(Node::Leaf {
                        value: None,
                        hash: latest_node_hash.clone(),
                    });
                } else {
                    extended_nodes.push(Node::Internal {
                        children: vec![],
                        hash: latest_node_hash.clone(),
                    });
                }
            }
        }
        extended_nodes
    }

    fn hash_nodes(nodes: &Vec<Node>) -> Hash {
        let mut hasher = Keccak256::new();
        for node in nodes {
            match node {
                Node::Internal { hash, .. } | Node::Leaf { hash, .. } => {
                    hasher.update(hash);
                }
            }
        }
        hasher.finalize().into()
    }

    pub fn open(&self, index: usize) -> (Value, MerkleProof) {
        let tree_path = Self::compute_path(index, self.height.unwrap(), self.k);
        let mut merkle_hashes: Vec<Hash> = Vec::new();
        let mut current_node = self.root.as_ref().unwrap();
        for path_index in tree_path.iter() {
            match current_node {
                Node::Internal { children, .. } => {
                    let mut siblings_hash: Vec<Hash> = Vec::new();
                    for (i, child_node) in children.iter().enumerate().rev() {
                        if i == *path_index {
                            continue;
                        }
                        match child_node {
                            Node::Internal { hash, .. } | Node::Leaf { hash, .. } => {
                                siblings_hash.push(hash.clone())
                            }
                        }
                    }
                    merkle_hashes.extend(siblings_hash);
                    current_node = children.get(*path_index).unwrap();
                }
                _ => panic!("Invalid node type"),
            }
        }
        let tree_value = match current_node {
            Node::Leaf { value, .. } => value.as_ref().unwrap().clone(),
            _ => panic!("Invalid node type"),
        };

        merkle_hashes.reverse();

        (
            tree_value,
            MerkleProof {
                hashes: merkle_hashes,
            },
        )
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

    pub fn verify(&self, index: usize, value: &Value, proof: &MerkleProof) -> bool {
        let mut calculated_hash: Hash = Keccak256::digest(value).into();
        let mut path_index = Self::compute_path(index, self.height.unwrap(), self.k);
        let mut merkle_hash = proof.hashes.iter();
        path_index.reverse();

        for merkle_hash_index in path_index {
            let mut hasher = Keccak256::new();
            for branch_index in 0..self.k {
                if branch_index == merkle_hash_index {
                    hasher.update(calculated_hash);
                } else {
                    hasher.update(merkle_hash.next().unwrap());
                }
            }
            calculated_hash = hasher.finalize().into();
        }

        calculated_hash == self.root_hash().unwrap()
    }

    pub fn print_tree(&self) {
        self.print_node(self.root.as_ref().unwrap(), 0);
    }

    fn print_node(&self, node: &Node, level: usize) {
        match node {
            Node::Internal { children, hash } => {
                println!(
                    "{}Internal Node - Hash: {:?}, Children size: {}",
                    " ".repeat(level * 2),
                    hash,
                    children.len()
                );
                for child in children {
                    self.print_node(child, level + 1);
                }
            }
            Node::Leaf { value, hash } => {
                println!(
                    "{}Leaf Node - Hash: {:?}, Value: {:?}",
                    " ".repeat(level * 2),
                    hash,
                    value
                );
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
    use std::vec;

    use super::*;
    use rand::Rng;

    #[test]
    fn it_instantiates_tree() {
        let width = 8;
        let tree = MerkleTree::setup(width);
        assert!(tree.height.is_none());
        assert!(tree.root.is_none());
    }

    #[test]
    fn it_commits() {
        let width = 2;
        let mut tree = MerkleTree::setup(width);
        let vec = vec![random_bytes(), random_bytes(), random_bytes()];
        let vec = vec.iter().collect::<Vec<_>>();
        tree.commit(vec.as_slice());
        tree.print_tree();

        assert_eq!(tree.height.unwrap(), 2);
    }

    #[test]
    fn it_tests_compute_path() {
        let width = 2;
        let height = 2;
        let index = 1;
        let tree_path = MerkleTree::compute_path(index, height, width);
        println!("Tree path: {:?}", tree_path);
    }

    #[test]
    fn it_opens() {
        let width = 2;
        let mut tree = MerkleTree::setup(width);
        let vec = vec![random_bytes(), random_bytes(), random_bytes()];
        let vec = vec.iter().collect::<Vec<_>>();
        tree.commit(vec.as_slice());

        let (value, _) = tree.open(1);
        assert_eq!(value, vec[1].clone());
    }

    #[test]
    fn it_verifies() {
        let width = 2;
        let mut tree = MerkleTree::setup(width);
        let vec = vec![random_bytes(), random_bytes(), random_bytes()];
        let vec = vec.iter().collect::<Vec<_>>();
        tree.commit(vec.as_slice());

        let (value, proof) = tree.open(1);
        assert!(tree.verify(1, &value, &proof));
    }

    #[test]
    fn it_rejects_false_statement() {
        let width = 2;
        let mut tree = MerkleTree::setup(width);
        let vec = vec![random_bytes(), random_bytes(), random_bytes()];
        let vec = vec.iter().collect::<Vec<_>>();
        tree.commit(vec.as_slice());

        let (value, proof) = tree.open(1);
        assert_eq!(tree.verify(0, &value, &proof), false);
    }

    #[test]
    fn it_verifies_bigger_vector() {
        let width = 4;
        let vec_length = 100;
        let mut tree = MerkleTree::setup(width);
        let vec = (0..vec_length).map(|_| random_bytes()).collect::<Vec<_>>();
        let vec = vec.iter().collect::<Vec<_>>();
        tree.commit(vec.as_slice());

        let random_index = rand::thread_rng().gen_range(0..vec_length);
        let (value, proof) = tree.open(random_index);
        assert!(tree.verify(random_index, &value, &proof));
    }

    fn random_bytes() -> Value {
        let mut rng = rand::thread_rng();
        (0..32).map(|_| rng.gen()).collect()
    }
}
