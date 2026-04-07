// =============================================================================
// opcode.rs — The OpCode enum
// =============================================================================
//
// WHAT IS AN ENUM IN RUST?
// -------------------------
// Rust enums are "sum types" — a value that can be one of several variants.
// Unlike C enums (which are just integers), Rust enum variants can carry data.
//
// This is fundamentally different from a class hierarchy in OOP.
// Instead of:
//   class OpCode {}
//   class OpDup extends OpCode {}
//   class Push extends OpCode { byte[] data; }
//
// We write:
//   enum OpCode {
//       OpDup,
//       Push(Vec<u8>),
//   }
//
// And use `match` to handle each case — the compiler FORCES you to handle
// every variant. If you add a new variant later, every match in your
// entire codebase will fail to compile until you handle it. That's a feature.
//
// BITCOIN OPCODES
// ----------------
// A Bitcoin script is a sequence of opcodes. The Bitcoin VM is a simple
// stack machine — opcodes pop values off a stack, do work, and push results.
// There is no heap, no loops, no jumps (mostly). It's intentionally simple
// to avoid Turing-completeness and make scripts easy to reason about.
//
// The complete opcode set is defined in Bitcoin's script.h. We implement
// the subset needed for P2PKH (Pay-to-Public-Key-Hash), which covers the
// vast majority of all Bitcoin transactions ever made.
//
// Full reference: https://en.bitcoin.it/wiki/Script

// `#[derive(Debug, Clone)]` auto-generates:
//   - Debug: allows printing with {:?} and {:#?}
//   - Clone: allows .clone() to make a copy (needed because Vec<u8> is heap data)

#[derive(Debug, Clone)]
pub enum OpCode {

    // ── Push opcodes (0x01–0x4b) ─────────────────────────────────────────────
    //
    // The byte 0x14 (decimal 20) means "push the next 20 bytes onto the stack".
    // Any byte from 0x01 to 0x4b is a "push N bytes" instruction.
    //
    // The Vec<u8> holds those bytes. In P2PKH, this is typically a 20-byte
    // public key hash (the actual Bitcoin address payload).
    //
    // `Vec<u8>` = a heap-allocated, growable array of bytes (unsigned 8-bit integers).
    Push(Vec<u8>),

    // ── OP_DUP (0x76) ────────────────────────────────────────────────────────
    //
    // Duplicates the top item on the stack.
    //
    // WHY DOES P2PKH USE THIS?
    // The spender provides a public key. We need to both:
    //   1. Hash it and compare against the stored hash (to verify it's the right key)
    //   2. Keep a copy for signature verification
    // OP_DUP solves this — duplicate the pubkey before hashing it.
    OpDup,

    // ── OP_HASH160 (0xa9) ─────────────────────────────────────────────────────
    //
    // Pops the top stack item, computes RIPEMD160(SHA256(data)), pushes the result.
    //
    // WHY HASH160 AND NOT JUST SHA256?
    // Bitcoin uses double hashing for extra security against length-extension attacks
    // and as a hedge against weaknesses in either algorithm.
    // RIPEMD160 produces a 20-byte (160-bit) output, which is shorter than SHA256's
    // 32 bytes. Shorter = smaller transactions = lower fees.
    //
    // HASH160 is also what converts a public key into a Bitcoin address:
    //   address = Base58Check(version_byte || RIPEMD160(SHA256(pubkey)))
    OpHash160,

    // ── OP_EQUALVERIFY (0x88) ─────────────────────────────────────────────────
    //
    // Pops the top two items. If they're not equal, the script FAILS immediately.
    // If they are equal, execution continues (nothing is pushed — the items are consumed).
    //
    // In P2PKH, this checks:
    //   HASH160(provided_pubkey) == stored_pubkey_hash
    // i.e., "is this actually the public key that corresponds to this address?"
    OpEqualVerify,

    // ── OP_CHECKSIG (0xac) ────────────────────────────────────────────────────
    //
    // Pops a public key and a signature. Verifies the signature against
    // the transaction hash using ECDSA on the secp256k1 elliptic curve.
    // Pushes 1 (TRUE) if valid, 0 (FALSE) if not.
    //
    // REAL CHECKSIG IS COMPLEX:
    // The actual message being signed is not the raw transaction — it's a
    // special serialization of the transaction with the input's scriptPubKey
    // substituted in, hashed with SHA256d (double SHA256). This is called
    // the "sighash". The SIGHASH_TYPE byte appended to the signature determines
    // which parts of the transaction are committed to (ALL, NONE, SINGLE, etc.).
    //
    // We implement this as a MOCK for learning purposes. Real secp256k1
    // verification requires either the `secp256k1` crate or `k256` crate.
    OpCheckSig,

    // ── OP_EQUAL (0x87) ───────────────────────────────────────────────────────
    //
    // Like OP_EQUALVERIFY but DOESN'T fail the script — pushes 1 or 0 instead.
    // Used in P2SH (Pay-to-Script-Hash) and other advanced script types.
    OpEqual,

    // ── OP_RETURN (0x6a) ──────────────────────────────────────────────────────
    //
    // Immediately marks the script as INVALID and terminates execution.
    // Outputs with OP_RETURN are unspendable — they're used to embed arbitrary
    // data in the blockchain (e.g., colored coins, timestamps, OP_RETURN protocols).
    // Miners include them for the fee; the output value is typically 0.
    OpReturn,

    // ── OP_0 (0x00) ───────────────────────────────────────────────────────────
    //
    // Pushes an empty byte array (which the Bitcoin VM evaluates as FALSE/zero).
    // Used in multisig scripts as a workaround for a historical bug (CHECKMULTISIG
    // consumes one extra stack item — OP_0 is that dummy item).
    OpZero,

    // ── Unknown opcodes ───────────────────────────────────────────────────────
    //
    // Opcodes we don't recognise. We store the raw byte so we can display it.
    // In the real Bitcoin VM, an unknown opcode makes the script fail.
    Unknown(u8),
}

impl OpCode {
    // Returns a short name string for display purposes.
    // `&self` means this method borrows the OpCode — it doesn't take ownership.
    // `-> &'static str` means it returns a string slice with 'static lifetime
    // (i.e., a string literal baked into the binary).
    pub fn name(&self) -> String {
        match self {
            OpCode::Push(data) => format!("PUSH[{}]", data.len()),
            OpCode::OpDup        => "OP_DUP".into(),
            OpCode::OpHash160    => "OP_HASH160".into(),
            OpCode::OpEqualVerify => "OP_EQUALVERIFY".into(),
            OpCode::OpCheckSig   => "OP_CHECKSIG".into(),
            OpCode::OpEqual      => "OP_EQUAL".into(),
            OpCode::OpReturn     => "OP_RETURN".into(),
            OpCode::OpZero       => "OP_0".into(),
            OpCode::Unknown(b)   => format!("UNKNOWN(0x{:02x})", b),
        }
    }

    // Returns a plain-English explanation of what this opcode does.
    // This is the "teaching" layer — what a learner needs to understand.
    pub fn explain(&self) -> &'static str {
        match self {
            OpCode::Push(_) =>
                "Pushes raw bytes onto the stack. In P2PKH outputs this is the 20-byte \
                 pubkey hash. In inputs, this carries the DER-encoded signature and \
                 the compressed public key.",
            OpCode::OpDup =>
                "Duplicates the top stack item. In P2PKH, the spender pushes their \
                 public key; OP_DUP copies it so we can both hash it (for address \
                 verification) and keep it for signature checking.",
            OpCode::OpHash160 =>
                "Pops the top item, computes RIPEMD160(SHA256(data)), pushes the \
                 20-byte result. This is the same operation used to derive a Bitcoin \
                 address from a public key. Here it converts the provided pubkey into \
                 a hash for comparison.",
            OpCode::OpEqualVerify =>
                "Pops the top two items and compares them. If they differ, the script \
                 fails immediately. In P2PKH, this verifies that the pubkey you \
                 provided actually hashes to the address in the locking script.",
            OpCode::OpCheckSig =>
                "Pops a public key and a signature. Verifies the ECDSA signature over \
                 the transaction sighash using the secp256k1 curve. Pushes 1 if valid. \
                 (Simplified here — real verification requires secp256k1 crypto.)",
            OpCode::OpEqual =>
                "Pops two items, pushes 1 if equal, 0 if not. Unlike OP_EQUALVERIFY, \
                 does not abort the script on mismatch.",
            OpCode::OpReturn =>
                "Immediately marks the script invalid. Outputs using OP_RETURN are \
                 provably unspendable and are used to embed data in the blockchain.",
            OpCode::OpZero =>
                "Pushes an empty byte array (interpreted as FALSE/0). Used as a dummy \
                 item for OP_CHECKMULTISIG due to a historical off-by-one bug.",
            OpCode::Unknown(_) =>
                "Unrecognised opcode. In the real Bitcoin VM, this would cause the \
                 script to fail.",
        }
    }

    /// Returns a list of all supported opcodes for the dictionary API endpoint
    pub fn dictionary() -> Vec<OpCode> {
        vec![
            OpCode::Push(vec![]),
            OpCode::OpDup,
            OpCode::OpHash160,
            OpCode::OpEqualVerify,
            OpCode::OpCheckSig,
            OpCode::OpEqual,
            OpCode::OpReturn,
            OpCode::OpZero,
            OpCode::Unknown(0xff),
        ]
    }
}
