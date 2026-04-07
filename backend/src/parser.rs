// =============================================================================
// parser.rs — Converts raw script bytes into Vec<OpCode>
// =============================================================================
//
// HOW BITCOIN SCRIPT ENCODING WORKS
// -----------------------------------
// A Bitcoin script is just a flat byte array. There's no type tagging,
// no length prefix for the whole script, no framing. You parse it left-to-right,
// one byte at a time, using the byte value itself to decide what comes next.
//
// The encoding rules for the opcodes we care about:
//
//   Byte value     Meaning
//   ──────────     ──────────────────────────────────────────────────────────
//   0x00           OP_0 — push empty array
//   0x01–0x4b      PUSH N — push the next N bytes onto the stack
//   0x4c NN ...    OP_PUSHDATA1 — next byte is N, then push N bytes
//   0x4d NN NN ... OP_PUSHDATA2 — next 2 bytes (little-endian) are N, push N bytes
//   0x4e NN NN NN NN ... OP_PUSHDATA4 — next 4 bytes (little-endian) are N
//   0x51–0x60      OP_1 through OP_16 — push the number 1 through 16
//   0x6a           OP_RETURN
//   0x76           OP_DUP
//   0x87           OP_EQUAL
//   0x88           OP_EQUALVERIFY
//   0xa9           OP_HASH160
//   0xac           OP_CHECKSIG
//
// WHY LITTLE-ENDIAN FOR PUSHDATA?
// Bitcoin uses little-endian byte order for multi-byte integers, meaning the
// least significant byte comes first. So the 2-byte value 0x0102 in
// little-endian is stored as [0x02, 0x01].
//
// EXAMPLE: A P2PKH scriptPubKey
// The hex: 76 a9 14 89abcdef...20bytes... 88 ac
//   76     → OP_DUP
//   a9     → OP_HASH160
//   14     → push 20 bytes (0x14 = decimal 20)
//   89...  → the 20-byte pubkey hash
//   88     → OP_EQUALVERIFY
//   ac     → OP_CHECKSIG

use crate::opcode::OpCode;

// `pub fn` — public function, callable from other modules.
// `bytes: &[u8]` — a borrowed slice of bytes. We don't own the data;
// we just read it. `[u8]` is an unsized slice; `&[u8]` is a "fat pointer"
// containing a pointer + length.
// `-> Vec<OpCode>` — we return an owned, heap-allocated vector.

pub fn parse_script(bytes: &[u8]) -> Vec<OpCode> {
    let mut ops = Vec::new();  // Vec::new() = empty growable array
    let mut i = 0;             // current byte position

    // `while i < bytes.len()` — iterate until we've consumed all bytes.
    // We use index-based iteration (not iterator-based) because some opcodes
    // consume multiple bytes and we need to advance `i` by more than 1.
    while i < bytes.len() {
        let byte = bytes[i];

        // Match on the byte value to determine the opcode.
        // `match` in Rust is exhaustive — we must handle every possible u8 (0..=255).
        // The `_` arm catches everything we haven't explicitly listed.
        let op = match byte {

            // ── Special single-byte opcodes ───────────────────────────────────

            0x00 => OpCode::OpZero,
            0x6a => OpCode::OpReturn,
            0x76 => OpCode::OpDup,
            0x87 => OpCode::OpEqual,
            0x88 => OpCode::OpEqualVerify,
            0xa9 => OpCode::OpHash160,
            0xac => OpCode::OpCheckSig,

            // ── Direct push: bytes 0x01 through 0x4b ─────────────────────────
            //
            // The byte value IS the number of bytes to push.
            // e.g., 0x14 (= 20 decimal) means "the next 20 bytes are data".
            //
            // `n @ 0x01..=0x4b` is a range pattern with binding.
            // The `@` binds the matched value to `n` so we can use it.
            // Without `@` we'd just know it matched, not what value it was.

            n @ 0x01..=0x4b => {
                let size = n as usize;
                let start = i + 1;
                let end = start + size;

                // Guard against malformed scripts that claim more bytes than exist.
                if end > bytes.len() {
                    ops.push(OpCode::Unknown(byte));
                    break;
                }

                // `bytes[start..end]` is a slice — a borrowed view into the array.
                // `.to_vec()` copies those bytes into a new heap-allocated Vec<u8>.
                // We need an owned Vec because OpCode::Push owns its data.
                let data = bytes[start..end].to_vec();
                i += size; // advance past the data bytes (we'll do i += 1 below for the opcode byte)
                OpCode::Push(data)
            }

            // ── OP_PUSHDATA1 (0x4c) ───────────────────────────────────────────
            //
            // One extra byte follows telling us how many bytes to push.
            // Used for data 76–255 bytes long.

            0x4c => {
                if i + 1 >= bytes.len() { ops.push(OpCode::Unknown(byte)); break; }
                let size = bytes[i + 1] as usize;
                let start = i + 2;
                let end = start + size;
                if end > bytes.len() { ops.push(OpCode::Unknown(byte)); break; }
                let data = bytes[start..end].to_vec();
                i += 1 + size;
                OpCode::Push(data)
            }

            // ── OP_PUSHDATA2 (0x4d) ───────────────────────────────────────────
            //
            // Two extra bytes (little-endian u16) tell us how many bytes to push.
            // Used for data 256–65535 bytes long.
            //
            // Little-endian u16 reconstruction:
            //   low byte  = bytes[i+1]
            //   high byte = bytes[i+2]
            //   value     = low_byte | (high_byte << 8)

            0x4d => {
                if i + 2 >= bytes.len() { ops.push(OpCode::Unknown(byte)); break; }
                let size = (bytes[i + 1] as usize) | ((bytes[i + 2] as usize) << 8);
                let start = i + 3;
                let end = start + size;
                if end > bytes.len() { ops.push(OpCode::Unknown(byte)); break; }
                let data = bytes[start..end].to_vec();
                i += 2 + size;
                OpCode::Push(data)
            }

            // ── OP_PUSHDATA4 (0x4e) ───────────────────────────────────────────
            //
            // Four extra bytes (little-endian u32). Rarely used in practice.

            0x4e => {
                if i + 4 >= bytes.len() { ops.push(OpCode::Unknown(byte)); break; }
                let size = (bytes[i+1] as usize)
                    | ((bytes[i+2] as usize) << 8)
                    | ((bytes[i+3] as usize) << 16)
                    | ((bytes[i+4] as usize) << 24);
                let start = i + 5;
                let end = start + size;
                if end > bytes.len() { ops.push(OpCode::Unknown(byte)); break; }
                let data = bytes[start..end].to_vec();
                i += 4 + size;
                OpCode::Push(data)
            }

            // ── OP_1 through OP_16 (0x51–0x60) ───────────────────────────────
            //
            // Push the small integer 1–16.
            // OP_1 = 0x51, OP_2 = 0x52, ..., OP_16 = 0x60
            // The value is (byte - 0x50).
            // Used in multisig: "OP_2 <key1> <key2> OP_2 OP_CHECKMULTISIG"
            // means "2-of-2 multisig".

            n @ 0x51..=0x60 => {
                let value = n - 0x50;
                // Represent the integer as a 1-byte push of that value.
                OpCode::Push(vec![value])
            }

            // ── Everything else ───────────────────────────────────────────────
            //
            // We haven't implemented this opcode. Store the raw byte.
            _ => OpCode::Unknown(byte),
        };

        ops.push(op);
        i += 1; // always advance past the current opcode byte
    }

    ops
}

// ─── Tests ────────────────────────────────────────────────────────────────────
//
// `#[cfg(test)]` means this block is only compiled when running `cargo test`.
// Tests live alongside the code they test in Rust — no separate test directory needed.

#[cfg(test)]
mod tests {
    use super::*; // import everything from the parent module

    #[test]
    fn test_p2pkh_script() {
        // A minimal P2PKH locking script: OP_DUP OP_HASH160 PUSH[1] 0xAB OP_EQUALVERIFY OP_CHECKSIG
        let bytes = vec![0x76, 0xa9, 0x01, 0xab, 0x88, 0xac];
        let ops = parse_script(&bytes);

        assert_eq!(ops.len(), 5);

        // Pattern matching in tests — check the variant AND its data
        assert!(matches!(ops[0], OpCode::OpDup));
        assert!(matches!(ops[1], OpCode::OpHash160));
        assert!(matches!(ops[2], OpCode::Push(ref d) if d == &[0xab]));
        assert!(matches!(ops[3], OpCode::OpEqualVerify));
        assert!(matches!(ops[4], OpCode::OpCheckSig));
    }

    #[test]
    fn test_pushdata1() {
        // 0x4c 0x03 0xAA 0xBB 0xCC = OP_PUSHDATA1, push 3 bytes [AA, BB, CC]
        let bytes = vec![0x4c, 0x03, 0xaa, 0xbb, 0xcc];
        let ops = parse_script(&bytes);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], OpCode::Push(ref d) if d == &[0xaa, 0xbb, 0xcc]));
    }

    #[test]
    fn test_truncated_script_doesnt_panic() {
        // A push that claims 20 bytes but the script ends after 3 — should not panic
        let bytes = vec![0x14, 0x01, 0x02, 0x03]; // 0x14 = push 20, but only 3 bytes follow
        let ops = parse_script(&bytes);
        // Should produce an Unknown opcode and not crash
        assert!(!ops.is_empty());
    }
}
