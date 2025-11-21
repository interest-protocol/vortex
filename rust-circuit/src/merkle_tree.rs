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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Path<const N: usize> {
    pub path: [(Fr, Fr); N],
}

impl<const N: usize> Path<N> {
    /// Creates a new empty path.
    pub fn empty() -> Self {
        Self {
            path: [(Fr::ZERO, Fr::ZERO); N],
        }
    }

    /// Takes in an expected `root_hash` and leaf-level data (i.e. hashes of
    /// secrets) for a leaf and checks that the leaf belongs to a tree having
    /// the expected hash.
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

    /// Recompute the Merkle root from a leaf and a path.
    ///
    /// This mirrors the Move `append_pair` logic and the circuit gadget `PathVar::root_hash`:
    ///
    /// - Level 0: path stores (leaf_left, leaf_right) - the pair of leaves
    /// - Levels 1 to N-1: path stores (left_sibling, right_sibling) at each level
    ///
    /// We start from the leaf, verify it's in the pair, hash the pair, then walk up using siblings.
    pub fn calculate_root(&self, leaf: &Fr, hasher: &PoseidonOptimized) -> anyhow::Result<Fr> {
        // This must match PathVar::root_hash exactly
        // Start with the leaf and iterate through all path levels
        let mut previous_hash = *leaf;

        for (p_left_hash, p_right_hash) in self.path.iter() {
            // Check if previous_hash matches the left hash
            let previous_is_left = previous_hash == *p_left_hash;

            // Select left and right based on which side previous_hash is on
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

    /// Given leaf data determine what the index of this leaf must be
    /// in the Merkle tree it belongs to. Before doing so check that the leaf
    /// does indeed belong to a tree with the given `root_hash`
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

        // Determine if leaf is left (0) or right (1) in the pair
        let is_left = *leaf == self.path[0].0;
        let mut index = if is_left { Fr::ZERO } else { Fr::from(1u64) };

        // Start from the pair hash (level 0 already processed)
        let mut prev = hasher.hash2(&self.path[0].0, &self.path[0].1);

        // Check levels 1 to N-1
        for (level, (left_hash, right_hash)) in self.path.iter().enumerate().skip(1) {
            // Check if the previous hash is for a left node or right node
            if &prev != left_hash {
                // Current is right child - set bit at position 'level'
                let bit_value = Fr::from(1u64 << level);
                index += bit_value;
            }
            prev = hasher.hash2(left_hash, right_hash);
        }

        Ok(index)
    }
}

/// Sparse Merkle tree using Tornado Cash Nova's paired insertion strategy.
/// Inserts two leaves at once for better efficiency and privacy.
#[derive(Debug, Clone, PartialEq)]
pub struct SparseMerkleTree<const N: usize> {
    /// Stored leaves (in insertion order)
    pub leaves: Vec<Fr>,
    /// Cached left subtrees at each level (indices 0 to N-1)
    /// Matches Nova's filledSubtrees array
    /// Note: subtrees[0] is initialized but never used (Nova quirk)
    subtrees: Vec<Fr>,
    /// An array of default hashes for each level
    empty_hashes: [Fr; N],
    /// Current root
    root: Fr,
}

impl<const N: usize> SparseMerkleTree<N> {
    /// Creates a new Sparse Merkle Tree using Nova's paired insertion strategy
    ///
    /// # Arguments
    /// * `leaf_pairs` - Pairs of leaves to insert
    /// * `hasher` - Poseidon hasher instance
    /// * `empty_leaf` - Zero value for empty leaves
    pub fn new(
        leaf_pairs: &[(Fr, Fr)],
        hasher: &PoseidonOptimized,
        empty_leaf: &Fr,
    ) -> anyhow::Result<Self> {
        // Build empty hashes array (levels 0 to N-1)
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

        // Initialize subtrees[0..N-1] (matching Nova's filledSubtrees)
        // Simply clone the empty_hashes array
        let subtrees = empty_hashes.to_vec();

        // Root of empty tree is empty_hashes[N-1]
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

    /// Creates a new empty Sparse Merkle Tree
    pub fn new_empty(hasher: &PoseidonOptimized, empty_leaf: &Fr) -> Self {
        Self::new(&[], hasher, empty_leaf).expect("Failed to create empty tree")
    }

    /// Insert a pair of leaves (Nova style)
    ///
    /// # Arguments
    /// * `leaf1` - First leaf to insert
    /// * `leaf2` - Second leaf to insert
    /// * `hasher` - Poseidon hasher instance
    pub fn insert_pair(
        &mut self,
        leaf1: Fr,
        leaf2: Fr,
        hasher: &PoseidonOptimized,
    ) -> anyhow::Result<()> {
        let max_leaves = 1usize << N;
        if self.leaves.len() + 2 > max_leaves {
            return Err(anyhow!(
                "Merkle tree is full. No more leaves can be added (capacity: {})",
                max_leaves
            ));
        }

        // Store both leaves
        self.leaves.push(leaf1);
        self.leaves.push(leaf2);

        // Start by hashing the leaf pair (level 0)
        let mut current_index = (self.leaves.len() - 2) / 2;
        let mut current_level_hash = hasher.hash2(&leaf1, &leaf2);

        // Process levels 1 to N-1 (matching Nova: for i = 1; i < levels)
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

    /// Insert a single leaf (for backward compatibility)
    /// Pairs it with a zero leaf
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

    /// Insert batch of leaves (must be even number)
    pub fn bulk_insert(&mut self, leaves: &[Fr], hasher: &PoseidonOptimized) -> anyhow::Result<()> {
        if leaves.len() % 2 != 0 {
            return Err(anyhow!("Must insert even number of leaves (pairs)"));
        }

        for i in (0..leaves.len()).step_by(2) {
            self.insert_pair(leaves[i], leaves[i + 1], hasher)?;
        }

        Ok(())
    }

    /// Returns the Merkle tree root.
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

    /// Generate membership proof for a leaf at given index
    ///
    /// # Arguments
    /// * `index` - Index of the leaf (0-based)
    ///
    /// # Returns
    /// A Path containing siblings at each level:
    /// - Level 0: (sibling_leaf, empty_hash) or (empty_hash, sibling_leaf) depending on position
    /// - Levels 1 to N-1: (left_sibling, right_sibling) at each level
    pub fn generate_membership_proof(&self, index: usize) -> anyhow::Result<Path<N>> {
        if index >= self.leaves.len() {
            return Err(anyhow!("Index out of bounds"));
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

        // Store the pair at level 0
        path[0] = (leaf_left, leaf_right);

        // Compute pair hash (this is what gets inserted at level 1)
        let mut current_hash = hasher.hash2(&leaf_left, &leaf_right);
        let mut current_index = pair_index;

        // Levels 1 to N-1: Store siblings at each level
        // Simulate Move append_pair for all pairs to rebuild tree state
        let num_pairs = self.leaves.len().div_ceil(2);
        let mut pair_hashes = Vec::with_capacity(num_pairs);
        for p in 0..num_pairs {
            let left = self.leaves[p * 2];
            let right = if p * 2 + 1 < self.leaves.len() {
                self.leaves[p * 2 + 1]
            } else {
                self.empty_hashes[0]
            };
            pair_hashes.push(hasher.hash2(&left, &right));
        }

        // Rebuild tree state by simulating Move insertion sequentially
        // This matches the exact Move append_pair logic
        let mut sim_subtrees = self.empty_hashes.to_vec();
        let mut level_hashes: Vec<Vec<Fr>> = Vec::new();

        // For each level 1 to N-1, build the tree by inserting pairs in order
        for level in 1..N {
            // Reset subtrees for this level's computation
            let mut level_subtrees = self.empty_hashes.to_vec();
            let mut level_data = Vec::new();

            // Insert each pair sequentially (matching Move's append_pair)
            for pair_idx in 0..num_pairs {
                let mut pos = pair_idx;
                let mut hash = pair_hashes[pair_idx];

                // Walk up levels 1 to this level
                for l in 1..=level {
                    let is_left = pos % 2 == 0;
                    let sibling = if is_left {
                        self.empty_hashes[l]
                    } else {
                        level_subtrees[l]
                    };

                    hash = hasher.hash2(
                        if is_left { &hash } else { &sibling },
                        if is_left { &sibling } else { &hash },
                    );

                    // Update subtrees if we're a left child (matching Move logic)
                    if is_left {
                        level_subtrees[l] = hash;
                    }

                    pos /= 2;
                }

                // Store the hash at the final position for this level
                let final_pos = pair_idx >> level; // pair_idx / (2^level)
                if level_data.len() <= final_pos {
                    level_data.resize(final_pos + 1, self.empty_hashes[level]);
                }
                level_data[final_pos] = hash;
            }

            level_hashes.push(level_data);
        }

        // Extract siblings from rebuilt tree
        for (level, path_elem) in path.iter_mut().enumerate().skip(1) {
            let is_left = current_index % 2 == 0;
            let level_idx = level - 1; // level_hashes is indexed from 0 for level 1
            let level_data = &level_hashes[level_idx];

            let sibling = if is_left {
                let sibling_pos = current_index + 1;
                level_data
                    .get(sibling_pos)
                    .copied()
                    .unwrap_or(self.empty_hashes[level])
            } else {
                if current_index > 0 {
                    level_data
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

/// Gadgets for one Merkle tree path
#[derive(Debug, Clone)]
pub struct PathVar<const N: usize> {
    path: [(FpVar<Fr>, FpVar<Fr>); N],
}

impl<const N: usize> PathVar<N> {
    /// check whether path belongs to merkle path (does not check if indexes
    /// match)
    pub fn check_membership(
        &self,
        root: &FpVar<Fr>,
        leaf: &FpVar<Fr>,
        hasher: &PoseidonOptimizedVar,
    ) -> Result<Boolean<Fr>, SynthesisError> {
        let computed_root = self.root_hash(leaf, hasher)?;

        root.is_eq(&computed_root)
    }

    /// Creates circuit to calculate merkle root and deny any invalid paths
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
                    panic!("Expected a Vec of length {} but it was {}", N, v.len())
                },
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_r1cs_std::R1CSVar;
    use ark_relations::r1cs::ConstraintSystem;

    #[test]
    fn test_path_verification_matches_circuit() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = Fr::from(0u64);

        // Test that Path::calculate_root matches PathVar::root_hash
        // This is the critical requirement for production
        let leaf = Fr::from(100u64);
        let sibling_leaf = Fr::from(200u64);

        // Create a simple path: level 0 has the pair, level 1+ have siblings
        let pair_hash = hasher.hash2(&leaf, &sibling_leaf);
        let empty_hash_1 = hasher.hash2(&empty_leaf, &empty_leaf);
        let level1_hash = hasher.hash2(&pair_hash, &empty_hash_1);

        // Path structure: (leaf_left, leaf_right) at level 0, then siblings
        let mut path = Path::<4>::empty();
        path.path[0] = (leaf, sibling_leaf); // Level 0: the pair
        path.path[1] = (pair_hash, empty_hash_1); // Level 1: (our hash, sibling)
        path.path[2] = (level1_hash, empty_hash_1); // Level 2: continue up
        path.path[3] = (hasher.hash2(&level1_hash, &empty_hash_1), empty_hash_1); // Level 3

        // Calculate root from path
        let computed_root = path.calculate_root(&leaf, &hasher).unwrap();

        // Verify in circuit
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
        let empty_leaf = Fr::from(0u64);

        let leaf_pairs = vec![
            (Fr::from(1u64), Fr::from(2u64)),
            (Fr::from(3u64), Fr::from(4u64)),
        ];

        let tree = SparseMerkleTree::<4>::new(&leaf_pairs, &hasher, &empty_leaf).unwrap();
        let root = tree.root();

        println!("Tree root: {}", root);
        println!("Tree has {} leaves", tree.len());

        // Generate proof for first leaf
        let path = tree.generate_membership_proof(0).unwrap();
        let leaf = Fr::from(1u64);

        // Verify membership (native)
        assert!(path.check_membership(&root, &leaf, &hasher).unwrap());
        println!("✓ Path verification successful");
    }

    #[test]
    fn test_bulk_insert() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = Fr::from(0u64);

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
        let empty_leaf = Fr::from(0u64);

        let leaf_pairs = vec![(Fr::from(1u64), Fr::from(2u64))];

        let tree = SparseMerkleTree::<4>::new(&leaf_pairs, &hasher, &empty_leaf).unwrap();
        let root = tree.root();
        let path = tree.generate_membership_proof(0).unwrap();
        let leaf = Fr::from(1u64);

        // Allocate variables
        let root_var = FpVar::new_input(cs.clone(), || Ok(root)).unwrap();
        let leaf_var = FpVar::new_witness(cs.clone(), || Ok(leaf)).unwrap();
        let path_var = PathVar::new_witness(cs.clone(), || Ok(path)).unwrap();

        // Create hasher for constraints
        let hasher_var = PoseidonOptimizedVar::new_t3();

        // Check membership in circuit
        let is_member = path_var
            .check_membership(&root_var, &leaf_var, &hasher_var)
            .unwrap();

        // Should be true
        assert!(is_member.value().unwrap());

        // Constraints should be satisfied
        assert!(cs.is_satisfied().unwrap());

        println!(
            "Merkle path verification constraints: {}",
            cs.num_constraints()
        );
    }

    #[test]
    fn test_single_insert_backward_compat() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = Fr::from(0u64);

        let mut tree = SparseMerkleTree::<4>::new_empty(&hasher, &empty_leaf);

        tree.insert(Fr::from(100u64), &hasher).unwrap();

        assert_eq!(tree.len(), 2); // Inserted as pair with zero
        println!("✓ Single insert (backward compat) successful");
    }

    #[test]
    fn test_tree_full() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = Fr::from(0u64);

        let mut tree = SparseMerkleTree::<2>::new_empty(&hasher, &empty_leaf); // Capacity = 4

        tree.insert_pair(Fr::from(1u64), Fr::from(2u64), &hasher)
            .unwrap();
        tree.insert_pair(Fr::from(3u64), Fr::from(4u64), &hasher)
            .unwrap();

        assert!(tree.is_full());

        // Should fail
        let result = tree.insert_pair(Fr::from(5u64), Fr::from(6u64), &hasher);
        assert!(result.is_err());
        println!("✓ Tree full check successful");
    }

    /// Ensure that for every leaf, the generated path recomputes the root
    /// correctly using the native `Path::calculate_root` and `check_membership`.
    #[test]
    fn test_path_roundtrip_all_leaves_native() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = Fr::from(0u64);

        // Build a small tree with multiple pairs
        let leaf_pairs = vec![
            (Fr::from(1u64), Fr::from(2u64)),
            (Fr::from(3u64), Fr::from(4u64)),
            (Fr::from(5u64), Fr::from(6u64)),
            (Fr::from(7u64), Fr::from(8u64)),
        ];

        let tree = SparseMerkleTree::<4>::new(&leaf_pairs, &hasher, &empty_leaf).unwrap();
        let root = tree.root();

        // For every actual leaf, generate a path and ensure it recomputes the root
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

    /// Reference implementation of the Move `append_pair` logic,
    /// adapted to a generic height N and using the same Poseidon hasher.
    fn move_style_root<const N: usize>(
        leaf_pairs: &[(Fr, Fr)],
        hasher: &PoseidonOptimized,
        empty_leaf: &Fr,
    ) -> Fr {
        assert!(N >= 2, "Tree height must be at least 2");

        // Build empty_subtree_hashes[0..=N] like in Move constants:
        // empty_subtree_hashes[0] = empty_leaf
        // empty_subtree_hashes[i] = Poseidon(hash_{i-1}, hash_{i-1})
        let mut empty_subtree_hashes = vec![Fr::ZERO; N + 1];
        empty_subtree_hashes[0] = *empty_leaf;
        let mut h = *empty_leaf;
        for i in 1..=N {
            h = hasher.hash2(&h, &h);
            empty_subtree_hashes[i] = h;
        }

        // subtrees[i] initialized to empty_subtree_hashes[i], like Move::new
        let mut subtrees = vec![Fr::ZERO; N];
        for i in 0..N {
            subtrees[i] = empty_subtree_hashes[i];
        }

        let mut next_index: u64 = 0;
        let mut root = empty_subtree_hashes[N]; // empty root in Move

        for (commitment0, commitment1) in leaf_pairs {
            // Capacity check: (1u64 << HEIGHT) > next_index
            assert!(
                (1u64 << (N as u32)) > next_index,
                "Merkle tree overflow in reference Move-style implementation"
            );

            let mut current_index = next_index / 2;
            let mut current_level_hash = hasher.hash2(commitment0, commitment1);

            // Move: for i in 1..HEIGHT  (macro range_do_eq!(1, HEIGHT - 1))
            // Here we treat HEIGHT == N.
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

    /// Check that the Rust SparseMerkleTree root matches the Move-style
    /// reference implementation for a given height.
    #[test]
    fn test_roots_match_move_style_reference_n4() {
        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = Fr::from(0u64);

        let leaf_pairs = vec![
            (Fr::from(1u64), Fr::from(2u64)),
            (Fr::from(3u64), Fr::from(4u64)),
            (Fr::from(5u64), Fr::from(6u64)),
        ];

        // Rust implementation
        let tree = SparseMerkleTree::<4>::new(&leaf_pairs, &hasher, &empty_leaf).unwrap();
        let rust_root = tree.root();

        // Move-style reference
        let move_root = move_style_root::<4>(&leaf_pairs, &hasher, &empty_leaf);

        assert_eq!(rust_root, move_root, "Rust root != Move-style root");
    }

    /// Explicitly check that the circuit gadget's computed root matches the
    /// native `Path::calculate_root`, which in turn matches the tree root.
    #[test]
    fn test_native_and_gadget_root_match() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        let hasher = PoseidonOptimized::new_t3();
        let empty_leaf = Fr::from(0u64);

        let leaf_pairs = vec![
            (Fr::from(10u64), Fr::from(20u64)),
            (Fr::from(30u64), Fr::from(40u64)),
        ];

        let tree = SparseMerkleTree::<4>::new(&leaf_pairs, &hasher, &empty_leaf).unwrap();
        let root = tree.root();

        let index = 1usize;
        let path = tree.generate_membership_proof(index).unwrap();
        let leaf = tree.leaves()[index];

        // Native root via Path
        let native_root = path.calculate_root(&leaf, &hasher).unwrap();
        assert_eq!(native_root, root);

        // Allocate variables in circuit
        let root_var = FpVar::new_input(cs.clone(), || Ok(root)).unwrap();
        let leaf_var = FpVar::new_witness(cs.clone(), || Ok(leaf)).unwrap();
        let path_var = PathVar::new_witness(cs.clone(), || Ok(path)).unwrap();
        let hasher_var = PoseidonOptimizedVar::new_t3();

        // Compute root in-circuit and compare
        let computed_root_var = path_var.root_hash(&leaf_var, &hasher_var).unwrap();
        computed_root_var.enforce_equal(&root_var).unwrap();

        assert!(cs.is_satisfied().unwrap());
    }
}
