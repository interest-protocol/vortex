use std::borrow::Borrow;

use anyhow::{anyhow, Context};
use ark_bn254::Fr;
use ark_ff::AdditiveGroup;
use ark_r1cs_std::{
    fields::fp::FpVar,
    prelude::{AllocVar, AllocationMode, Boolean, EqGadget},
    select::CondSelectGadget,
};
use ark_relations::r1cs::{Namespace, SynthesisError};

use crate::poseidon_opt::{PoseidonOptimized, PoseidonOptimizedVar};

/// Merkle tree path structure
/// Each level contains (left_hash, right_hash) pair
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Path<const N: usize> {
    pub path: [(Fr, Fr); N],
}

impl<const N: usize> Path<N> {
    /// Creates a new empty path
    pub fn empty() -> Self {
        Self {
            path: [(Fr::ZERO, Fr::ZERO); N],
        }
    }

    /// Check if leaf belongs to tree with given root
    pub fn check_membership(
        &self,
        root_hash: &Fr,
        leaf: &Fr,
        hasher: &PoseidonOptimized,
    ) -> anyhow::Result<bool> {
        let root = self
            .calculate_root(leaf, hasher)
            .context("Failed to calculate Merkle root during membership check")?;
        Ok(root == *root_hash)
    }

    /// Calculate Merkle root from leaf and path
    ///
    /// This matches PathVar::root_hash and Move's append_pair logic:
    /// - Level 0: path stores (leaf_left, leaf_right)
    /// - Levels 1 to N-1: path stores (left_sibling, right_sibling)
    pub fn calculate_root(&self, leaf: &Fr, hasher: &PoseidonOptimized) -> anyhow::Result<Fr> {
        let mut previous_hash = *leaf;

        for (p_left_hash, p_right_hash) in self.path.iter() {
            let previous_is_left = previous_hash == *p_left_hash;

            let left_hash = if previous_is_left {
                previous_hash
            } else {
                *p_left_hash
            };
            let right_hash = if previous_is_left {
                *p_right_hash
            } else {
                previous_hash
            };

            previous_hash = hasher.hash2(&left_hash, &right_hash);
        }

        Ok(previous_hash)
    }

    /// Get the index of a leaf in the tree
    pub fn get_index(
        &self,
        root_hash: &Fr,
        leaf: &Fr,
        hasher: &PoseidonOptimized,
    ) -> anyhow::Result<Fr> {
        if !self.check_membership(root_hash, leaf, hasher)? {
            return Err(anyhow!(
                "Cannot get index: leaf is not a member of tree with given root"
            ));
        }

        let is_left = *leaf == self.path[0].0;
        let mut index = if is_left { Fr::ZERO } else { Fr::from(1u64) };

        let mut prev = hasher.hash2(&self.path[0].0, &self.path[0].1);

        for (level, (left_hash, right_hash)) in self.path.iter().enumerate().skip(1) {
            if &prev != left_hash {
                let bit_value = Fr::from(1u64 << level);
                index += bit_value;
            }
            prev = hasher.hash2(left_hash, right_hash);
        }

        Ok(index)
    }
}

/// Sparse Merkle Tree using Nova's paired insertion strategy
#[derive(Debug, Clone, PartialEq)]
pub struct SparseMerkleTree<const N: usize> {
    /// Stored leaves (in insertion order)
    pub leaves: Vec<Fr>,
    /// Cached left subtrees at each level
    subtrees: Vec<Fr>,
    /// Default empty hashes for each level
    empty_hashes: [Fr; N],
    /// Current root
    root: Fr,
}

impl<const N: usize> SparseMerkleTree<N> {
    /// Create new tree with initial leaf pairs
    pub fn new(
        leaf_pairs: &[(Fr, Fr)],
        hasher: &PoseidonOptimized,
        empty_leaf: &Fr,
    ) -> anyhow::Result<Self> {
        // Build empty hashes array
        let empty_hashes = {
            let mut empty_hashes = [Fr::ZERO; N];
            empty_hashes[0] = *empty_leaf;

            let mut empty_hash = *empty_leaf;
            for hash in empty_hashes.iter_mut().skip(1) {
                empty_hash = hasher.hash2(&empty_hash, &empty_hash);
                *hash = empty_hash;
            }

            empty_hashes
        };

        // Initialize subtrees
        let subtrees = empty_hashes.to_vec();

        // Empty tree root
        let root = empty_hashes[N - 1];

        let mut smt = SparseMerkleTree {
            leaves: Vec::new(),
            subtrees,
            empty_hashes,
            root,
        };

        // Insert leaf pairs
        for (leaf1, leaf2) in leaf_pairs {
            smt.insert_pair(*leaf1, *leaf2, hasher)?;
        }

        Ok(smt)
    }

    /// Create new empty tree
    pub fn new_empty(hasher: &PoseidonOptimized, empty_leaf: &Fr) -> Self {
        Self::new(&[], hasher, empty_leaf).expect("Failed to create empty tree")
    }

    /// Insert a pair of leaves (Nova/Move style)
    pub fn insert_pair(
        &mut self,
        leaf1: Fr,
        leaf2: Fr,
        hasher: &PoseidonOptimized,
    ) -> anyhow::Result<()> {
        let max_leaves = 1usize << N;
        if self.leaves.len() + 2 > max_leaves {
            return Err(anyhow!("Merkle tree is full (capacity: {})", max_leaves));
        }

        // Store both leaves
        self.leaves.push(leaf1);
        self.leaves.push(leaf2);

        // Level 0: Hash the leaf pair
        let mut current_index = (self.leaves.len() - 2) / 2;
        let mut current_level_hash = hasher.hash2(&leaf1, &leaf2);

        // Levels 1 to N-1 (matching Move: for i in 1..HEIGHT)
        for i in 1..N {
            let left: Fr;
            let right: Fr;

            if current_index % 2 == 0 {
                // Current is left child
                left = current_level_hash;
                right = self.empty_hashes[i];
                self.subtrees[i] = current_level_hash; // Cache left subtree
            } else {
                // Current is right child
                left = self.subtrees[i]; // Get cached left subtree
                right = current_level_hash;
            }

            current_level_hash = hasher.hash2(&left, &right);
            current_index /= 2;
        }

        self.root = current_level_hash;
        Ok(())
    }

    /// Insert single leaf (pairs with zero)
    pub fn insert(&mut self, leaf: Fr, hasher: &PoseidonOptimized) -> anyhow::Result<()> {
        self.insert_pair(leaf, self.empty_hashes[0], hasher)
    }

    /// Insert batch of leaf pairs
    pub fn insert_batch(
        &mut self,
        leaf_pairs: &[(Fr, Fr)],
        hasher: &PoseidonOptimized,
    ) -> anyhow::Result<()> {
        for (leaf1, leaf2) in leaf_pairs {
            self.insert_pair(*leaf1, *leaf2, hasher)?;
        }
        Ok(())
    }

    /// Bulk insert (must be even number of leaves)
    pub fn bulk_insert(&mut self, leaves: &[Fr], hasher: &PoseidonOptimized) -> anyhow::Result<()> {
        if leaves.len() % 2 != 0 {
            return Err(anyhow!("Must insert even number of leaves (pairs)"));
        }

        for i in (0..leaves.len()).step_by(2) {
            self.insert_pair(leaves[i], leaves[i + 1], hasher)?;
        }

        Ok(())
    }

    /// Returns the Merkle tree root
    pub fn root(&self) -> Fr {
        self.root
    }

    /// Returns the number of leaves in the tree
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Returns true if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Returns true if the tree is full
    pub fn is_full(&self) -> bool {
        self.leaves.len() >= (1 << N)
    }

    /// Get all leaves
    pub fn leaves(&self) -> &[Fr] {
        &self.leaves
    }

    /// Generate membership proof for leaf at given index
    ///
    /// Returns a Path containing siblings at each level:
    /// - Level 0: (left_leaf, right_leaf) - the pair
    /// - Levels 1 to N-1: (left_sibling, right_sibling) at each level
    pub fn generate_membership_proof(&self, index: usize) -> anyhow::Result<Path<N>> {
        if index >= self.leaves.len() {
            return Err(anyhow!(
                "Index {} out of bounds (tree has {} leaves)",
                index,
                self.leaves.len()
            ));
        }

        let mut path = [(Fr::ZERO, Fr::ZERO); N];
        let hasher = PoseidonOptimized::new_t3();

        // Level 0: Store the pair of leaves
        let pair_index = index / 2;
        let leaf_left = self.leaves[pair_index * 2];
        let leaf_right = if pair_index * 2 + 1 < self.leaves.len() {
            self.leaves[pair_index * 2 + 1]
        } else {
            self.empty_hashes[0]
        };

        path[0] = (leaf_left, leaf_right);

        // Compute pair hash
        let mut current_hash = hasher.hash2(&leaf_left, &leaf_right);
        let mut current_index = pair_index;

        // Rebuild tree state by simulating all insertions up to this point
        // This matches the Move append_pair logic exactly
        let num_pairs = self.leaves.len().div_ceil(2);
        let mut pair_hashes = Vec::with_capacity(num_pairs);

        // Compute all pair hashes
        for p in 0..num_pairs {
            let left = self.leaves[p * 2];
            let right = if p * 2 + 1 < self.leaves.len() {
                self.leaves[p * 2 + 1]
            } else {
                self.empty_hashes[0]
            };
            pair_hashes.push(hasher.hash2(&left, &right));
        }

        // Rebuild tree state by simulating all insertions
        // We need to track the hash at each position at each level BEFORE combining with siblings
        // This allows us to extract the correct sibling for the path
        let mut level_child_hashes: Vec<Vec<Fr>> = Vec::new();

        for level in 1..N {
            // Initialize subtrees for this level (matching Move's subtrees array)
            let mut level_subtrees = self.empty_hashes.to_vec();
            // Track child hashes (before combining with siblings) at each position
            let mut child_hashes: Vec<Fr> = Vec::new();

            // Simulate inserting each pair sequentially (matching insert_pair logic)
            for (pair_idx, &pair_hash) in pair_hashes.iter().enumerate() {
                let mut pos = pair_idx;
                let mut hash = pair_hash;

                // Walk up from level 1 to current level
                for (empty_hash, subtree) in self.empty_hashes[1..level]
                    .iter()
                    .zip(level_subtrees[1..level].iter_mut())
                {
                    let is_left = pos % 2 == 0;
                    let left: Fr;
                    let right: Fr;

                    if is_left {
                        left = hash;
                        right = *empty_hash;
                        *subtree = hash; // Cache left subtree
                    } else {
                        left = *subtree; // Get cached left subtree
                        right = hash;
                    }

                    hash = hasher.hash2(&left, &right);
                    pos /= 2;
                }

                // At the current level, store the child hash (before combining with sibling)
                let level_pos = pair_idx >> (level - 1);
                if child_hashes.len() <= level_pos {
                    child_hashes.resize(level_pos + 1, self.empty_hashes[level]);
                }
                child_hashes[level_pos] = hash;
            }

            level_child_hashes.push(child_hashes);
        }

        // Extract siblings from rebuilt tree
        for (level, path_elem) in path.iter_mut().enumerate().skip(1) {
            let is_left = current_index % 2 == 0;
            let level_idx = level - 1;
            let child_hashes = &level_child_hashes[level_idx];

            let sibling = if is_left {
                // We're on the left, sibling is on the right
                let sibling_pos = current_index + 1;
                child_hashes
                    .get(sibling_pos)
                    .copied()
                    .unwrap_or(self.empty_hashes[level])
            } else {
                // We're on the right, sibling is on the left
                if current_index > 0 {
                    child_hashes
                        .get(current_index - 1)
                        .copied()
                        .unwrap_or(self.subtrees[level])
                } else {
                    self.subtrees[level]
                }
            };

            *path_elem = if is_left {
                (current_hash, sibling)
            } else {
                (sibling, current_hash)
            };

            current_hash = hasher.hash2(
                if is_left { &current_hash } else { &sibling },
                if is_left { &sibling } else { &current_hash },
            );
            current_index /= 2;
        }

        Ok(Path { path })
    }

    /// Verify a path leads to the expected root
    pub fn verify_path(&self, index: usize, path: &Path<N>) -> anyhow::Result<bool> {
        if index >= self.leaves.len() {
            return Ok(false);
        }

        let leaf = self.leaves[index];
        let hasher = PoseidonOptimized::new_t3();

        path.check_membership(&self.root, &leaf, &hasher)
    }
}

/// Circuit variable for Merkle path
#[derive(Debug, Clone)]
pub struct PathVar<const N: usize> {
    path: [(FpVar<Fr>, FpVar<Fr>); N],
}

impl<const N: usize> PathVar<N> {
    /// Check membership in circuit
    pub fn check_membership(
        &self,
        root: &FpVar<Fr>,
        leaf: &FpVar<Fr>,
        hasher: &PoseidonOptimizedVar,
    ) -> Result<Boolean<Fr>, SynthesisError> {
        let computed_root = self.root_hash(leaf, hasher)?;
        root.is_eq(&computed_root)
    }

    /// Calculate root hash in circuit
    pub fn root_hash(
        &self,
        leaf: &FpVar<Fr>,
        hasher: &PoseidonOptimizedVar,
    ) -> Result<FpVar<Fr>, SynthesisError> {
        assert_eq!(self.path.len(), N);
        let mut previous_hash = leaf.clone();

        for (p_left_hash, p_right_hash) in self.path.iter() {
            let previous_is_left = previous_hash.is_eq(p_left_hash)?;

            let left_hash =
                FpVar::conditionally_select(&previous_is_left, &previous_hash, p_left_hash)?;
            let right_hash =
                FpVar::conditionally_select(&previous_is_left, p_right_hash, &previous_hash)?;

            previous_hash = hasher.hash2(&left_hash, &right_hash)?;
        }

        Ok(previous_hash)
    }
}

impl<const N: usize> AllocVar<Path<N>, Fr> for PathVar<N> {
    fn new_variable<T: Borrow<Path<N>>>(
        cs: impl Into<Namespace<Fr>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();

        let mut path = Vec::new();
        let path_obj = f()?;
        for (l, r) in &path_obj.borrow().path {
            let l_hash =
                FpVar::<Fr>::new_variable(ark_relations::ns!(cs, "l_child"), || Ok(*l), mode)?;
            let r_hash =
                FpVar::<Fr>::new_variable(ark_relations::ns!(cs, "r_child"), || Ok(*r), mode)?;
            path.push((l_hash, r_hash));
        }

        Ok(PathVar {
            path: path.try_into().unwrap_or_else(
                #[allow(clippy::type_complexity)]
                |v: Vec<(FpVar<Fr>, FpVar<Fr>)>| {
                    panic!("Expected path of length {}, got {}", N, v.len())
                },
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::ZERO_VALUE;

    /// Convert ZERO_VALUE string constant to Fr field element
    fn zero_value() -> Fr {
        use num_bigint::BigUint;
        use std::str::FromStr;

        Fr::from(BigUint::from_str(ZERO_VALUE).expect("Failed to parse ZERO_VALUE"))
    }
    use ark_r1cs_std::R1CSVar;
    use ark_relations::r1cs::ConstraintSystem;

    #[test]
    fn test_path_verification_matches_circuit() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = zero_value();

        let leaf = Fr::from(100u64);
        let sibling_leaf = Fr::from(200u64);

        let pair_hash = hasher.hash2(&leaf, &sibling_leaf);
        let empty_hash_1 = hasher.hash2(&empty_leaf, &empty_leaf);
        let level1_hash = hasher.hash2(&pair_hash, &empty_hash_1);

        let mut path = Path::<4>::empty();
        path.path[0] = (leaf, sibling_leaf);
        path.path[1] = (pair_hash, empty_hash_1);
        path.path[2] = (level1_hash, empty_hash_1);
        path.path[3] = (hasher.hash2(&level1_hash, &empty_hash_1), empty_hash_1);

        let computed_root = path.calculate_root(&leaf, &hasher).unwrap();

        let cs = ConstraintSystem::<Fr>::new_ref();
        let root_var = FpVar::new_input(cs.clone(), || Ok(computed_root)).unwrap();
        let leaf_var = FpVar::new_witness(cs.clone(), || Ok(leaf)).unwrap();
        let path_var = PathVar::new_witness(cs.clone(), || Ok(path)).unwrap();
        let hasher_var = PoseidonOptimizedVar::new_t3();

        let circuit_root = path_var.root_hash(&leaf_var, &hasher_var).unwrap();
        circuit_root.enforce_equal(&root_var).unwrap();

        assert!(cs.is_satisfied().unwrap());
        println!("✓ Path verification matches circuit");
    }

    #[test]
    fn test_sparse_merkle_tree_nova_style() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = zero_value();

        let leaf_pairs = vec![
            (Fr::from(1u64), Fr::from(2u64)),
            (Fr::from(3u64), Fr::from(4u64)),
        ];

        let tree = SparseMerkleTree::<4>::new(&leaf_pairs, &hasher, &empty_leaf).unwrap();
        let root = tree.root();

        println!("Tree root: {}", root);
        println!("Tree has {} leaves", tree.len());

        let path = tree.generate_membership_proof(0).unwrap();
        let leaf = Fr::from(1u64);

        assert!(path.check_membership(&root, &leaf, &hasher).unwrap());
        println!("✓ Path verification successful");
    }

    #[test]
    fn test_bulk_insert() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = zero_value();

        let mut tree = SparseMerkleTree::<4>::new_empty(&hasher, &empty_leaf);

        let leaves = vec![
            Fr::from(10u64),
            Fr::from(20u64),
            Fr::from(30u64),
            Fr::from(40u64),
        ];

        tree.bulk_insert(&leaves, &hasher).unwrap();

        assert_eq!(tree.len(), 4);
        println!("✓ Bulk insert successful");
    }

    #[test]
    fn test_path_var_constraint_generation() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = zero_value();

        let leaf_pairs = vec![(Fr::from(1u64), Fr::from(2u64))];
        let tree = SparseMerkleTree::<4>::new(&leaf_pairs, &hasher, &empty_leaf).unwrap();
        let root = tree.root();
        let path = tree.generate_membership_proof(0).unwrap();
        let leaf = Fr::from(1u64);

        let root_var = FpVar::new_input(cs.clone(), || Ok(root)).unwrap();
        let leaf_var = FpVar::new_witness(cs.clone(), || Ok(leaf)).unwrap();
        let path_var = PathVar::new_witness(cs.clone(), || Ok(path)).unwrap();
        let hasher_var = PoseidonOptimizedVar::new_t3();

        let is_member = path_var
            .check_membership(&root_var, &leaf_var, &hasher_var)
            .unwrap();

        assert!(is_member.value().unwrap());
        assert!(cs.is_satisfied().unwrap());

        println!(
            "Merkle path verification constraints: {}",
            cs.num_constraints()
        );
    }

    #[test]
    fn test_single_insert_backward_compat() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = zero_value();

        let mut tree = SparseMerkleTree::<4>::new_empty(&hasher, &empty_leaf);
        tree.insert(Fr::from(100u64), &hasher).unwrap();

        assert_eq!(tree.len(), 2);
        println!("✓ Single insert (backward compat) successful");
    }

    #[test]
    fn test_tree_full() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = zero_value();

        let mut tree = SparseMerkleTree::<2>::new_empty(&hasher, &empty_leaf);

        tree.insert_pair(Fr::from(1u64), Fr::from(2u64), &hasher)
            .unwrap();
        tree.insert_pair(Fr::from(3u64), Fr::from(4u64), &hasher)
            .unwrap();

        assert!(tree.is_full());

        let result = tree.insert_pair(Fr::from(5u64), Fr::from(6u64), &hasher);
        assert!(result.is_err());
        println!("✓ Tree full check successful");
    }

    #[test]
    fn test_path_roundtrip_all_leaves_native() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = zero_value();

        let leaf_pairs = vec![
            (Fr::from(1u64), Fr::from(2u64)),
            (Fr::from(3u64), Fr::from(4u64)),
            (Fr::from(5u64), Fr::from(6u64)),
            (Fr::from(7u64), Fr::from(8u64)),
        ];

        let tree = SparseMerkleTree::<4>::new(&leaf_pairs, &hasher, &empty_leaf).unwrap();
        let root = tree.root();

        for (index, leaf) in tree.leaves().iter().enumerate() {
            let path = tree.generate_membership_proof(index).unwrap();
            let recomputed_root = path.calculate_root(leaf, &hasher).unwrap();

            assert_eq!(
                root, recomputed_root,
                "Recomputed root mismatch for leaf index {}",
                index
            );
            assert!(path.check_membership(&root, leaf, &hasher).unwrap());
        }
    }

    /// Reference Move-style implementation for testing
    fn move_style_root<const N: usize>(
        leaf_pairs: &[(Fr, Fr)],
        hasher: &PoseidonOptimized,
        empty_leaf: &Fr,
    ) -> Fr {
        assert!(N >= 2);

        let mut empty_subtree_hashes = vec![Fr::ZERO; N + 1];
        empty_subtree_hashes[0] = *empty_leaf;
        let mut h = *empty_leaf;
        for hash in empty_subtree_hashes.iter_mut().skip(1).take(N) {
            h = hasher.hash2(&h, &h);
            *hash = h;
        }

        let mut subtrees = vec![Fr::ZERO; N];
        subtrees.copy_from_slice(&empty_subtree_hashes[..N]);

        let mut next_index: u64 = 0;
        let mut root = empty_subtree_hashes[N];

        for (commitment0, commitment1) in leaf_pairs {
            assert!((1u64 << (N as u32)) > next_index);

            let mut current_index = next_index / 2;
            let mut current_level_hash = hasher.hash2(commitment0, commitment1);

            for i in 1..N {
                let subtree = &mut subtrees[i];
                let (left, right) = if current_index % 2 == 0 {
                    *subtree = current_level_hash;
                    (current_level_hash, empty_subtree_hashes[i])
                } else {
                    (*subtree, current_level_hash)
                };

                current_level_hash = hasher.hash2(&left, &right);
                current_index /= 2;
            }

            next_index += 2;
            root = current_level_hash;
        }

        root
    }

    #[test]
    fn test_roots_match_move_style_reference_n4() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = zero_value();

        let leaf_pairs = vec![
            (Fr::from(1u64), Fr::from(2u64)),
            (Fr::from(3u64), Fr::from(4u64)),
            (Fr::from(5u64), Fr::from(6u64)),
        ];

        let rust_tree = SparseMerkleTree::<4>::new(&leaf_pairs, &hasher, &empty_leaf).unwrap();
        let rust_root = rust_tree.root();

        let move_root = move_style_root::<4>(&leaf_pairs, &hasher, &empty_leaf);

        assert_eq!(rust_root, move_root, "Rust root != Move-style root");
        println!("✓ Rust root matches Move root exactly");
    }

    #[test]
    fn test_native_and_gadget_root_match() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = zero_value();

        let leaf_pairs = vec![
            (Fr::from(10u64), Fr::from(20u64)),
            (Fr::from(30u64), Fr::from(40u64)),
        ];

        let tree = SparseMerkleTree::<4>::new(&leaf_pairs, &hasher, &empty_leaf).unwrap();
        let root = tree.root();

        let index = 1usize;
        let path = tree.generate_membership_proof(index).unwrap();
        let leaf = tree.leaves()[index];

        let native_root = path.calculate_root(&leaf, &hasher).unwrap();
        assert_eq!(native_root, root);

        let root_var = FpVar::new_input(cs.clone(), || Ok(root)).unwrap();
        let leaf_var = FpVar::new_witness(cs.clone(), || Ok(leaf)).unwrap();
        let path_var = PathVar::new_witness(cs.clone(), || Ok(path)).unwrap();
        let hasher_var = PoseidonOptimizedVar::new_t3();

        let computed_root_var = path_var.root_hash(&leaf_var, &hasher_var).unwrap();
        computed_root_var.enforce_equal(&root_var).unwrap();

        assert!(cs.is_satisfied().unwrap());
        println!("✓ Native and circuit roots match");
    }
}
