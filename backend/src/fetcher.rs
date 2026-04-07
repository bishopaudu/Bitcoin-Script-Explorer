// =============================================================================
// fetcher.rs — Fetches real Bitcoin transactions from mempool.space
// =============================================================================
//
// WHAT THIS MODULE DOES
// ----------------------
// It calls https://mempool.space/api/tx/{txid} and deserializes the JSON
// response into Rust structs. That's it — but there's a lot to learn here
// about how Rust handles JSON and async HTTP.
//
// HOW SERDE WORKS
// ----------------
// serde is a compile-time framework. When you write:
//
//   #[derive(Deserialize)]
//   struct Foo { bar: String }
//
// The Rust compiler (via a "proc macro") generates code equivalent to:
//
//   impl<'de> serde::Deserialize<'de> for Foo {
//       fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Foo, D::Error> {
//           // reads JSON key "bar", expects a string, assigns to bar field
//       }
//   }
//
// This happens at compile time — at runtime there is zero overhead from
// reflection or dynamic dispatch. It's as fast as hand-written parsing.
//
// The `#[serde(rename_all = "snake_case")]` attribute tells serde that JSON
// keys follow snake_case naming, matching our Rust field names exactly.
//
// `Option<T>` means a field may or may not be present in the JSON.
// serde maps a missing JSON key → None, and a present one → Some(value).

use serde::Deserialize;

// ─── Top-level transaction ────────────────────────────────────────────────────
//
// This mirrors the JSON structure returned by mempool.space.
// Every field is `pub` so other modules can read it.
// Fields we don't need can simply be omitted — serde ignores unknown fields
// by default (controlled by #[serde(deny_unknown_fields)] if you want strictness).

#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub txid: String,

    // Transaction version (1 or 2). Version 2 enables relative timelocks (BIP 68).
    pub version: u32,

    // Locktime: the transaction cannot be included in a block until this
    // block height (if < 500_000_000) or Unix timestamp (if >= 500_000_000) is reached.
    // Most transactions set this to 0, meaning "mine immediately".
    pub locktime: u32,

    // `vin` = "vector of inputs". Each input spends a previous output (UTXO).
    pub vin: Vec<TxInput>,

    // `vout` = "vector of outputs". Each output creates a new UTXO (unspent output).
    pub vout: Vec<TxOutput>,

    // Size in bytes of the serialized transaction.
    #[serde(default)]
    pub size: Option<u32>,

    // Weight units (introduced by SegWit, BIP 141).
    // weight = base_size * 3 + total_size  (simplified)
    #[serde(default)]
    pub weight: Option<u32>,

    // Fee in satoshis (1 BTC = 100,000,000 satoshis).
    // Not directly in the raw transaction — mempool.space calculates it for us.
    #[serde(default)]
    pub fee: Option<u64>,

    // Confirmation status
    pub status: Option<TxStatus>,
}

// ─── Transaction Input ────────────────────────────────────────────────────────
//
// An input "spends" a UTXO. It references it by:
//   txid  — the transaction that created the UTXO
//   vout  — which output index within that transaction
//
// Then it provides the `scriptsig` to satisfy the locking conditions.
//
// COINBASE INPUTS are special: the first transaction in every block is the
// coinbase, which creates new BTC from thin air. Its `txid` is all zeros,
// `vout` is 0xFFFFFFFF, and `scriptsig` contains the miner's arbitrary data.

#[derive(Debug, Deserialize)]
pub struct TxInput {
    // Previous output being spent (all zeros for coinbase)
    pub txid: Option<String>,

    // Index of the output in that previous transaction
    pub vout: Option<u32>,

    // The unlocking script, in hex. For P2PKH: contains <signature> <pubkey>.
    // Empty or missing for SegWit inputs (witness data is separate).
    pub scriptsig: Option<String>,

    // Human-readable disassembly of scriptsig, provided by mempool.space.
    pub scriptsig_asm: Option<String>,

    // Witness data for SegWit inputs. Each item is a hex-encoded byte array.
    // For P2WPKH (native SegWit), this contains [signature, pubkey].
    #[serde(default)]
    pub witness: Vec<String>,

    // Sequence number. Used for opt-in RBF (Replace-By-Fee) and relative timelocks.
    // 0xFFFFFFFF means "final" (no RBF, no timelock).
    pub sequence: Option<u32>,

    // True only for the coinbase input.
    pub is_coinbase: Option<bool>,

    // The previous output being spent (mempool.space fills this in for us).
    // In a raw transaction this data is NOT present — you'd need to look it up.
    pub prevout: Option<PrevOut>,
}

// ─── Previous Output (embedded in inputs by mempool.space) ───────────────────

#[derive(Debug, Deserialize)]
pub struct PrevOut {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: Option<String>,
    pub scriptpubkey_type: Option<String>,
    pub scriptpubkey_address: Option<String>,

    // Value in satoshis
    pub value: u64,
}

// ─── Transaction Output ───────────────────────────────────────────────────────
//
// An output creates a new UTXO — an "unspent transaction output".
// `value` satoshis are locked by `scriptpubkey` until someone provides
// a valid `scriptsig` (or witness) to spend it.

#[derive(Debug, Deserialize)]
pub struct TxOutput {
    // The locking script in hex. This is what we parse and execute.
    pub scriptpubkey: String,

    // Human-readable disassembly, e.g. "OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY OP_CHECKSIG"
    pub scriptpubkey_asm: Option<String>,

    // Script type as classified by mempool.space: "p2pkh", "p2sh", "v0_p2wpkh", etc.
    pub scriptpubkey_type: Option<String>,

    // The Bitcoin address (Base58Check or Bech32) derived from the script.
    // Not present for non-standard scripts like OP_RETURN.
    pub scriptpubkey_address: Option<String>,

    // Value in satoshis. u64 because Bitcoin's total supply (21M BTC = 2.1 * 10^15 sat)
    // fits easily in u64 (max ~1.8 * 10^19).
    pub value: u64,
}

// ─── Transaction Status ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TxStatus {
    pub confirmed: bool,
    pub block_height: Option<u32>,
    pub block_hash: Option<String>,
    pub block_time: Option<u64>,
}

// ─── The actual fetch function ────────────────────────────────────────────────
//
// `async fn` returns a Future — a value representing a computation that
// hasn't happened yet. Calling this function starts the computation;
// `.await` in the caller resumes it when the network responds.
//
// `Result<Transaction, reqwest::Error>` means:
//   Ok(tx)  — success, we got a Transaction
//   Err(e)  — something failed: network error, bad JSON, 404, etc.

pub async fn fetch_transaction(txid: &str) -> Result<Transaction, reqwest::Error> {
    let url = format!("https://mempool.space/api/tx/{}", txid);

    // reqwest::get() sends a GET request and returns a Response.
    // `.await` pauses HERE and gives control back to the Tokio runtime until
    // the HTTP response headers arrive.
    let response = reqwest::get(&url).await?;

    // `.json::<Transaction>()` reads the response body, deserializes it as JSON
    // into our Transaction struct using serde.
    // `.await` pauses again until the full body is received.
    //
    // The turbofish syntax `::<Transaction>` tells the compiler which type to
    // deserialize into. Rust could often infer this from context, but being
    // explicit makes the code easier to read.
    let tx = response.json::<Transaction>().await?;

    Ok(tx)
}
