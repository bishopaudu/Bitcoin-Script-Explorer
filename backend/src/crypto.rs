// =============================================================================
// crypto.rs — Real SHA-256 and RIPEMD-160 hashing
// =============================================================================
//
// WHY DOES BITCOIN USE THESE TWO HASH FUNCTIONS?
// ------------------------------------------------
// Bitcoin uses a two-step hash: RIPEMD160(SHA256(data)), called HASH160.
//
// SHA-256 (Secure Hash Algorithm 256-bit):
//   - Designed by the NSA, standardised by NIST (FIPS 180-4)
//   - 256-bit (32-byte) output
//   - Collision resistance: finding two inputs with the same output requires
//     ~2^128 operations (computationally infeasible)
//
// RIPEMD-160 (RACE Integrity Primitives Evaluation Message Digest):
//   - Designed by European academics as an open alternative to SHA
//   - 160-bit (20-byte) output — shorter, meaning smaller transaction sizes
//   - Applied AFTER SHA-256 as a second layer of protection
//
// The combination means:
//   1. An attacker would need to break BOTH algorithms simultaneously
//   2. The 20-byte output produces shorter addresses (saving blockchain space)
//   3. Even if SHA-256 were broken, finding a valid public key from just the
//      160-bit RIPEMD-160 hash is still computationally infeasible
//
// HOW THE CRATES WORK
// --------------------
// The `sha2` and `ripemd` crates implement the Digest trait from the `digest`
// crate. They all follow the same API:
//
//   1. Create a hasher:  let mut hasher = Sha256::new();
//   2. Feed data:        hasher.update(data);
//   3. Finalize:         let result = hasher.finalize();
//
// OR the one-shot version:   Sha256::digest(data)
//
// `finalize()` returns a `GenericArray<u8, N>` — a fixed-size array whose
// length is known at compile time. For SHA256, N=32. For RIPEMD160, N=20.
// We call `.to_vec()` to convert to a heap-allocated Vec<u8> for easier handling.

use sha2::{Sha256, Digest as Sha2Digest};
use ripemd::{Ripemd160, Digest as RipemdDigest};

// ─── hash160 ─────────────────────────────────────────────────────────────────
//
// RIPEMD160(SHA256(data)) — the Bitcoin HASH160 operation.
//
// Input:  any byte slice
// Output: 20-byte Vec<u8>
//
// This is called in OP_HASH160 execution. It's also used to:
//   - Derive a Bitcoin address from a public key
//   - Derive the script hash for P2SH addresses
//
// Note how we pass the output of SHA256 directly into RIPEMD160.
// Chaining is safe because the output of SHA256 is treated as opaque bytes
// by RIPEMD160 — there's no special interaction between the two algorithms.

pub fn hash160(data: &[u8]) -> Vec<u8> {
    // Step 1: SHA256
    // Sha256::digest() is a one-shot convenience method.
    // It returns a GenericArray<u8, U32> — a 32-byte fixed array.
    let sha256_result = Sha256::digest(data);

    // Step 2: RIPEMD160 of the SHA256 output
    // We pass a reference to the GenericArray, which implements Deref<Target=[u8]>
    // so it can be used anywhere a &[u8] is expected.
    let ripemd_result = Ripemd160::digest(&sha256_result);

    // Convert to Vec<u8> for owned, heap-allocated storage
    ripemd_result.to_vec()
}

// ─── sha256d ──────────────────────────────────────────────────────────────────
//
// SHA256(SHA256(data)) — "double SHA256", used for:
//   - Transaction IDs (txid = SHA256d of the serialized transaction)
//   - Block hashes
//   - The sighash (the message that OP_CHECKSIG actually signs)
//   - Merkle tree nodes
//
// This function isn't used in our script execution (we mock CheckSig)
// but it's here because understanding it is essential for Bitcoin.

pub fn sha256d(data: &[u8]) -> Vec<u8> {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    second.to_vec()
}

// ─── sha256 ───────────────────────────────────────────────────────────────────
//
// Single SHA256. Exposed for completeness.

pub fn sha256(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}

// ─── Tests ────────────────────────────────────────────────────────────────────
//
// We test against known vectors from the Bitcoin protocol specification.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        // SHA256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let result = sha256(&[]);
        assert_eq!(
            hex::encode(result),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_hash160_known_vector() {
        // Known HASH160 of the bytes [0x00]:
        // SHA256([0x00]) = 6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d
        // RIPEMD160 of that = 9f7fd096d37ed2c0e3f7f0cfc924beef4ffaea18
        let result = hash160(&[0x00]);
        assert_eq!(hex::encode(result), "9f7fd096d37ed2c0e3f7f0cfc924beef4ffaea18");
    }
}
