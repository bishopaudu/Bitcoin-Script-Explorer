// =============================================================================
// engine.rs — The Bitcoin Script stack machine
// =============================================================================
//
// HOW A STACK MACHINE WORKS
// --------------------------
// A stack is a last-in, first-out (LIFO) data structure.
// Two operations: push (add to top) and pop (remove from top).
//
// Bitcoin Script executes left-to-right through the opcodes:
//   - Push opcodes add data to the top of the stack
//   - Operator opcodes (DUP, HASH160, etc.) pop inputs and push outputs
//   - The script SUCCEEDS if, after all opcodes, the top of the stack is TRUE (non-zero, non-empty)
//   - The script FAILS if the top is FALSE (zero or empty), or if any VERIFY opcode failed
//
// THE COMPLETE P2PKH EXECUTION FLOW
// ------------------------------------
// Locking script (scriptPubKey):
//   OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG
//
// Unlocking script (scriptSig):
//   <signature> <pubKey>
//
// Bitcoin actually concatenates them: scriptSig + scriptPubKey runs as one.
// (With a security check between them in modern Bitcoin to prevent attacks.)
//
// Step-by-step execution, stack shown as [bottom ... top]:
//
//   PUSH <sig>          stack: [sig]
//   PUSH <pubkey>       stack: [sig, pubkey]
//   OP_DUP              stack: [sig, pubkey, pubkey]
//   OP_HASH160          stack: [sig, pubkey, hash160(pubkey)]
//   PUSH <pubKeyHash>   stack: [sig, pubkey, hash160(pubkey), pubKeyHash]
//   OP_EQUALVERIFY      checks hash160(pubkey) == pubKeyHash, pops both
//                       stack: [sig, pubkey]
//   OP_CHECKSIG         checks sig is valid over tx using pubkey, pops both
//                       stack: [1]           ← TRUE = script succeeds!

use crate::opcode::OpCode;
use crate::crypto;

// ─── ExecutionStep ────────────────────────────────────────────────────────────
//
// A snapshot of the interpreter state after one opcode executes.
// We collect these into a Vec so the display module can show the full trace.

#[derive(Debug)]
pub struct ExecutionStep {
    // Which opcode just ran
    pub opcode: String,

    // Human-readable description of what happened
    pub description: String,

    // The entire stack after this step (bottom to top)
    // Each item is hex-encoded for display
    pub stack: Vec<String>,

    // Whether this step caused a failure
    pub failed: bool,

    // The teaching explanation for this opcode
    pub explanation: &'static str,
}

// ─── ExecutionResult ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ExecutionResult {
    pub steps: Vec<ExecutionStep>,

    // Did the script succeed? (top of stack is truthy after all opcodes)
    pub success: bool,

    // If it failed, why?
    pub failure_reason: Option<String>,
}

// ─── execute ──────────────────────────────────────────────────────────────────
//
// Runs the script and returns a full trace.
//
// `ops: &[OpCode]` — we borrow a slice, not an owned Vec.
// This is idiomatic Rust: prefer slices over Vec references in function signatures
// because &[T] works with both Vec<T> and arrays.

pub fn execute(ops: &[OpCode]) -> ExecutionResult {
    // The stack stores raw byte arrays.
    // `Vec<Vec<u8>>` = a vector of byte vectors.
    // The outer Vec is the stack; each inner Vec<u8> is one stack item.
    let mut stack: Vec<Vec<u8>> = Vec::new();

    let mut steps: Vec<ExecutionStep> = Vec::new();
    let mut failed = false;
    let mut failure_reason: Option<String> = None;

    for op in ops {
        if failed {
            break;
        }

        // Each arm handles one opcode and produces a description string.
        // We use a closure pattern: compute the step, push it, continue.
        let (description, step_failed, fail_msg) = match op {

            // ── PUSH ─────────────────────────────────────────────────────────
            //
            // The simplest opcode: just copy the data onto the stack.
            // `.clone()` because the data lives in the OpCode which we borrowed;
            // we need an owned copy for the stack.

            OpCode::Push(data) => {
                let hex = hex::encode(data);
                let short = if hex.len() > 16 { format!("{}…", &hex[..14]) } else { hex.clone() };
                stack.push(data.clone());
                (format!("Pushed {} byte{}: 0x{}", data.len(),
                    if data.len() == 1 { "" } else { "s" }, short),
                 false, None)
            }

            // ── OP_DUP ───────────────────────────────────────────────────────
            //
            // `.last()` returns Option<&Vec<u8>> — None if stack is empty.
            // `.cloned()` converts Option<&Vec<u8>> to Option<Vec<u8>> by cloning.
            // We use `if let` to destructure the Option — runs the block only if Some.

            OpCode::OpDup => {
                if let Some(top) = stack.last().cloned() {
                    stack.push(top);
                    ("Duplicated top stack item".into(), false, None)
                } else {
                    (String::new(), true, Some("OP_DUP: stack is empty".into()))
                }
            }

            // ── OP_HASH160 ───────────────────────────────────────────────────
            //
            // `.pop()` removes and returns the top item: Option<Vec<u8>>.
            // We call our real crypto module — no mocks here.

            OpCode::OpHash160 => {
                if let Some(data) = stack.pop() {
                    let hash = crypto::hash160(&data);
                    let hash_hex = hex::encode(&hash);
                    let desc = format!("RIPEMD160(SHA256(top)) = 0x{}…", &hash_hex[..12]);
                    stack.push(hash);
                    (desc, false, None)
                } else {
                    (String::new(), true, Some("OP_HASH160: stack is empty".into()))
                }
            }

            // ── OP_EQUALVERIFY ───────────────────────────────────────────────
            //
            // Pop two items, compare them, fail if not equal.
            // Note: we pop `b` first (top), then `a` (second from top).
            // Both are consumed — nothing is pushed on success.

            OpCode::OpEqualVerify => {
                if stack.len() < 2 {
                    (String::new(), true, Some("OP_EQUALVERIFY: need 2 stack items, have {}".into()))
                } else {
                    let b = stack.pop().unwrap(); // top
                    let a = stack.pop().unwrap(); // second
                    if a == b {
                        ("Hashes match — EQUALVERIFY passed".into(), false, None)
                    } else {
                        let msg = format!(
                            "OP_EQUALVERIFY failed: 0x{} ≠ 0x{}",
                            hex::encode(&a), hex::encode(&b)
                        );
                        (String::new(), true, Some(msg))
                    }
                }
            }

            // ── OP_EQUAL ─────────────────────────────────────────────────────
            //
            // Like EQUALVERIFY but pushes 1/0 instead of failing.
            // 0x01 = TRUE, 0x00 = FALSE in Bitcoin script.

            OpCode::OpEqual => {
                if stack.len() < 2 {
                    (String::new(), true, Some("OP_EQUAL: need 2 stack items".into()))
                } else {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let equal = a == b;
                    stack.push(if equal { vec![0x01] } else { vec![0x00] });
                    (format!("Compared items: {}", if equal { "equal → pushed 0x01" } else { "not equal → pushed 0x00" }),
                     false, None)
                }
            }

            // ── OP_CHECKSIG ───────────────────────────────────────────────────
            //
            // In real Bitcoin:
            //   1. Pop the public key
            //   2. Pop the signature (last byte is SIGHASH_TYPE, strip it)
            //   3. Compute the sighash: SHA256d of the serialized transaction
            //      with the scriptPubKey of the input being spent substituted in
            //   4. Verify the ECDSA signature over the sighash using secp256k1
            //   5. Push 1 (TRUE) or 0 (FALSE)
            //
            // We MOCK step 3 and 4 because:
            //   a) We don't have the full transaction context in our engine
            //   b) secp256k1 ECDSA requires the `secp256k1` or `k256` crate
            //      which adds significant complexity
            //   c) For LEARNING the script execution flow, the mock is sufficient
            //
            // To add real verification, you would:
            //   cargo add secp256k1 --features bitcoin_hashes
            // Then use: secp256k1::Secp256k1::new().verify_ecdsa(...)

            OpCode::OpCheckSig => {
                if stack.len() < 2 {
                    (String::new(), true, Some("OP_CHECKSIG: need 2 stack items".into()))
                } else {
                    let pubkey = stack.pop().unwrap();
                    let sig    = stack.pop().unwrap();
                    let pk_hex = hex::encode(&pubkey);
                    let sig_hex = hex::encode(&sig);
                    // Mock: always TRUE
                    stack.push(vec![0x01]);
                    (format!("Checked sig (0x{}…) against pubkey (0x{}…) → TRUE (mock)",
                        &sig_hex[..std::cmp::min(8, sig_hex.len())],
                        &pk_hex[..std::cmp::min(8, pk_hex.len())]),
                     false, None)
                }
            }

            // ── OP_RETURN ─────────────────────────────────────────────────────

            OpCode::OpReturn => {
                ("OP_RETURN encountered — script is provably unspendable".into(),
                 true, Some("OP_RETURN: script terminated".into()))
            }

            // ── OP_0 ─────────────────────────────────────────────────────────

            OpCode::OpZero => {
                stack.push(vec![]); // empty array = FALSE in Bitcoin script
                ("Pushed empty array (FALSE / 0)".into(), false, None)
            }

            // ── Unknown ───────────────────────────────────────────────────────

            OpCode::Unknown(b) => {
                (format!("Unknown opcode 0x{:02x} — skipped", b), false, None)
            }
        };

        // Snapshot the stack state AFTER this opcode
        let stack_snapshot: Vec<String> = stack.iter()
            .map(|item| {
                let h = hex::encode(item);
                if h.is_empty() { "(empty)".into() } else { format!("0x{}", h) }
            })
            .collect();

        if step_failed {
            failed = true;
            failure_reason = fail_msg.clone();
        }

        steps.push(ExecutionStep {
            opcode: op.name(),
            description: if step_failed { fail_msg.unwrap_or_default() } else { description },
            stack: stack_snapshot,
            failed: step_failed,
            explanation: op.explain(),
        });
    }

    // ── Determine final result ────────────────────────────────────────────────
    //
    // A script succeeds if:
    //   1. No opcode failed (no EQUALVERIFY mismatch, no stack underflow, etc.)
    //   2. The stack is non-empty after execution
    //   3. The top item is "truthy" — not empty AND not all-zero bytes
    //
    // Bitcoin's exact truthiness rules:
    //   - Empty vector []         → FALSE
    //   - [0x00]                  → FALSE (zero)
    //   - [0x00, 0x00]            → FALSE (still zero in script's numeric encoding)
    //   - [0x80]                  → FALSE (negative zero)
    //   - [0x01] or any non-zero  → TRUE
    //   - [0x81]                  → TRUE (negative one, but non-zero)

    let success = if failed {
        false
    } else if let Some(top) = stack.last() {
        is_truthy(top)
    } else {
        false // empty stack = failure
    };

    ExecutionResult {
        steps,
        success,
        failure_reason,
    }
}

// ─── is_truthy ────────────────────────────────────────────────────────────────
//
// Implements Bitcoin's stack truthiness evaluation.
// An item is FALSE if it is:
//   - Empty
//   - All zero bytes
//   - All zero bytes with a final 0x80 (negative zero in Bitcoin's number encoding)

fn is_truthy(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }
    for (i, &byte) in data.iter().enumerate() {
        let is_last = i == data.len() - 1;
        if is_last {
            // Last byte: 0x00 or 0x80 (sign byte) both count as zero
            if byte != 0x00 && byte != 0x80 {
                return true;
            }
        } else if byte != 0x00 {
            return true;
        }
    }
    false
}
