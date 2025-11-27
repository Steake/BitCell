//! Merkle tree implementation for state commitments
//!
//! Binary Merkle tree with SHA-256 hashing.

use crate::Hash256;
use serde::{Deserialize, Serialize};

/// Merkle tree for state commitments
#[derive(Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    leaves: Vec<Hash256>,
    nodes: Vec<Vec<Hash256>>,
}

impl MerkleTree {
    /// Create a new Merkle tree from leaves
    pub fn new(leaves: Vec<Hash256>) -> Self {
        if leaves.is_empty() {
            return Self {
                leaves: vec![Hash256::zero()],
                nodes: vec![vec![Hash256::zero()]],
            };
        }

        let mut current_level = leaves.clone();
        let mut nodes = vec![current_level.clone()];

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for i in (0..current_level.len()).step_by(2) {
                let left = current_level[i];
                let right = if i + 1 < current_level.len() {
                    current_level[i + 1]
                } else {
                    left // Duplicate if odd number
                };

                let parent = Hash256::hash_multiple(&[left.as_bytes(), right.as_bytes()]);
                next_level.push(parent);
            }

            nodes.push(next_level.clone());
            current_level = next_level;
        }

        Self { leaves, nodes }
    }

    /// Get the root hash
    pub fn root(&self) -> Hash256 {
        self.nodes.last().and_then(|level| level.first()).copied()
            .unwrap_or(Hash256::zero())
    }

    /// Generate a Merkle proof for a leaf at the given index
    pub fn prove(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_index = index;

        for level in &self.nodes[..self.nodes.len() - 1] {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            let sibling = if sibling_index < level.len() {
                level[sibling_index]
            } else {
                level[current_index] // Duplicate if odd
            };

            proof.push(sibling);
            current_index /= 2;
        }

        Some(MerkleProof {
            index,
            leaf: self.leaves[index],
            path: proof,
        })
    }

    /// Verify a Merkle proof against a root
    pub fn verify_proof(root: Hash256, proof: &MerkleProof) -> bool {
        let mut current = proof.leaf;
        let mut index = proof.index;

        for sibling in &proof.path {
            current = if index % 2 == 0 {
                Hash256::hash_multiple(&[current.as_bytes(), sibling.as_bytes()])
            } else {
                Hash256::hash_multiple(&[sibling.as_bytes(), current.as_bytes()])
            };
            index /= 2;
        }

        current == root
    }
}

/// Merkle proof for a leaf
#[derive(Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub index: usize,
    pub leaf: Hash256,
    pub path: Vec<Hash256>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_single_leaf() {
        let leaves = vec![Hash256::hash(b"leaf0")];
        let tree = MerkleTree::new(leaves.clone());
        
        assert_eq!(tree.root(), leaves[0]);
    }

    #[test]
    fn test_merkle_tree_multiple_leaves() {
        let leaves = vec![
            Hash256::hash(b"leaf0"),
            Hash256::hash(b"leaf1"),
            Hash256::hash(b"leaf2"),
            Hash256::hash(b"leaf3"),
        ];
        let tree = MerkleTree::new(leaves);
        
        assert_ne!(tree.root(), Hash256::zero());
    }

    #[test]
    fn test_merkle_proof() {
        let leaves = vec![
            Hash256::hash(b"leaf0"),
            Hash256::hash(b"leaf1"),
            Hash256::hash(b"leaf2"),
            Hash256::hash(b"leaf3"),
        ];
        let tree = MerkleTree::new(leaves);
        let root = tree.root();

        // Test proof for each leaf
        for i in 0..4 {
            let proof = tree.prove(i).unwrap();
            assert!(MerkleTree::verify_proof(root, &proof));
        }
    }

    #[test]
    fn test_merkle_proof_invalid() {
        let leaves = vec![
            Hash256::hash(b"leaf0"),
            Hash256::hash(b"leaf1"),
        ];
        let tree = MerkleTree::new(leaves);
        let root = tree.root();

        let mut proof = tree.prove(0).unwrap();
        proof.leaf = Hash256::hash(b"wrong");

        assert!(!MerkleTree::verify_proof(root, &proof));
    }

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::new(vec![]);
        assert_eq!(tree.root(), Hash256::zero());
    }
}
