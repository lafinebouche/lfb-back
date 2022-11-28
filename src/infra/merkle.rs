use hex_literal::hex;
use merkletree::hash::{Algorithm, Hashable};
use merkletree::merkle::{Element, MerkleTree};
use merkletree::store::Store;
use std::hash::Hasher;
use tiny_keccak::{Hasher as kHasher, Keccak};
use typenum::Unsigned;

// TODO currently creation of a tree based on [u8;SIZE] is not working

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Default)]
pub struct Item([u8; SIZE]);
pub const SIZE: usize = 0x20;

impl AsRef<[u8]> for Item {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Element for Item {
    fn byte_len() -> usize {
        SIZE
    }
    fn from_slice(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), SIZE);
        let mut b = [0u8; SIZE];
        b.copy_from_slice(bytes);
        Item(b)
    }

    fn copy_to_slice(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.0)
    }
}

pub struct Keccak256Hasher {
    engine: Keccak,
}

impl Keccak256Hasher {
    pub fn new() -> Self {
        return Keccak256Hasher {
            engine: Keccak::v256(),
        };
    }
}

impl Hasher for Keccak256Hasher {
    fn finish(&self) -> u64 {
        unimplemented!(
            "Hasher's contract (finish function is not used) is deliberately broken by design"
        )
    }

    fn write(&mut self, bytes: &[u8]) {
        self.engine.update(bytes)
    }
}

impl Default for Keccak256Hasher {
    fn default() -> Self {
        return Keccak256Hasher::new();
    }
}

impl Algorithm<Item> for Keccak256Hasher {
    fn hash(&mut self) -> Item {
        let mut result = Item::default();
        let item_size = result.0.len();
        let mut output = [0u8; SIZE];
        self.engine.clone().finalize(&mut output);
        result.0.copy_from_slice(&output.as_slice()[0..item_size]);
        result
    }
}

// generate dataset of iterable elements
pub fn generate_vector_of_elements<E: Element>(domains: Vec<[u8; SIZE]>) -> Vec<E> {
    let result = domains.into_iter().map(|x| E::from_slice(&x));
    result.collect()
}

pub fn keccak256(bytes: &[u8]) -> [u8; SIZE] {
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    let mut output = [0u8; SIZE];
    hasher.finalize(&mut output);
    output
}

pub fn get_namehash(domain: String) -> [u8; SIZE] {
    domain.rsplit('.').fold([0u8; SIZE], |node, label| {
        keccak256(&[node, keccak256(label.as_bytes())].concat())
    })
}

pub fn get_merkle_tree<E: Element, A: Algorithm<E>, S: Store<E>, U: Unsigned>(leaves: Vec<String>) {
    let hashes = leaves.into_iter().map(|x| get_namehash(x)).collect();
    let elems: Vec<Item> = generate_vector_of_elements(hashes);
    let tree: MerkleTree<E, A, S, U> = MerkleTree::new(elems).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namehash_eth() {
        assert_eq!(
            hex!("93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae"),
            get_namehash(String::from("eth"))
        );
    }

    #[test]
    fn test_namehash_alice_eth() {
        assert_eq!(
            hex!("787192fc5378cc32aa956ddfdedbf26b24e8d78e40109add0eea2c1a012c3dec"),
            get_namehash(String::from("alice.eth"))
        );
    }
}
