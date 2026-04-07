// =============================================================================
// main.rs — Axum HTTP server
// =============================================================================
//
// HOW AXUM WORKS
// ---------------
// Axum is a web framework built on top of tokio + hyper (the HTTP library).
// You define a "Router" — a mapping from URL paths to handler functions.
// Each handler is an async function that returns something implementing IntoResponse.
//
// CORS (Cross-Origin Resource Sharing)
// --------------------------------------
// Browsers block JavaScript from calling APIs on different domains by default.
// Our React frontend (localhost:5173) calling our backend (localhost:3001) is
// a "cross-origin" request. The backend must send CORS headers to allow it.
// tower-http's CorsLayer handles this automatically.

mod fetcher;
mod opcode;
mod parser;
mod crypto;
mod engine;

use axum::{
    routing::get,
    Router,
    extract::Path,
    Json,
    http::StatusCode,
};
use tower_http::cors::{CorsLayer, Any};
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    // Build the CORS layer — allow any origin (fine for development)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the router.
    // `.route("/api/tx/:txid", get(handle_tx))` means:
    //   HTTP GET requests to /api/tx/<anything> → call handle_tx()
    //   The `:txid` segment is a URL parameter, extracted by Path<String>.
    let app = Router::new()
        .route("/api/tx/:txid", get(handle_tx))
        .route("/api/opcodes", get(handle_opcodes))
        .route("/api/health", get(|| async { Json(json!({ "ok": true })) }))
        .layer(cors);

   // let addr = "0.0.0.0:3001";
   let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
let addr = format!("0.0.0.0:{}", port);
    println!("Bitcoin Script Explorer backend running on http://{}", addr);

    // axum::serve replaces the old axum::Server in axum 0.7+
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

async fn handle_opcodes() -> (StatusCode, Json<Value>) {
    let dict = opcode::OpCode::dictionary();
    
    let opcodes_json: Vec<Value> = dict.iter().map(|op| {
        json!({
            "name": op.name(),
            "explain": op.explain(),
            "hex": match op {
                opcode::OpCode::OpDup => Some("0x76"),
                opcode::OpCode::OpHash160 => Some("0xa9"),
                opcode::OpCode::OpEqualVerify => Some("0x88"),
                opcode::OpCode::OpCheckSig => Some("0xac"),
                opcode::OpCode::OpEqual => Some("0x87"),
                opcode::OpCode::OpReturn => Some("0x6a"),
                opcode::OpCode::OpZero => Some("0x00"),
                opcode::OpCode::Push(_) => Some("0x01-0x4b"),
                opcode::OpCode::Unknown(_) => None,
            }
        })
    }).collect();

    (StatusCode::OK, Json(json!({ "opcodes": opcodes_json })))
}

//
// `Path(txid): Path<String>` — Axum extracts the :txid URL segment automatically.
// Returns `(StatusCode, Json<Value>)` — a tuple Axum knows how to turn into an HTTP response.
// Json<Value> serializes a serde_json::Value to JSON with the right Content-Type header.

async fn handle_tx(
    Path(txid): Path<String>,
) -> (StatusCode, Json<Value>) {

    // Validate txid length
    if txid.len() != 64 || !txid.chars().all(|c| c.is_ascii_hexdigit()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid txid: must be 64 hex characters" })),
        );
    }

    // Fetch from mempool.space
    let tx = match fetcher::fetch_transaction(&txid).await {
        Ok(tx) => tx,
        Err(e) => return (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("Failed to fetch transaction: {}", e) })),
        ),
    };

    // Build the response — process all outputs and inputs
    let outputs: Vec<Value> = tx.vout.iter().enumerate().map(|(i, output)| {
        // Outputs are LOCKING scripts (scriptPubKey). They set spending conditions
        // but cannot be executed in isolation — they need a matching scriptSig to
        // provide inputs onto the stack first. We parse the opcodes for display
        // but skip engine execution to avoid misleading "FAILED" results.
        let script_result = parse_locking_script(&output.scriptpubkey);
        json!({
            "index": i,
            "value_sat": output.value,
            "value_btc": output.value as f64 / 100_000_000.0,
            "address": output.scriptpubkey_address,
            "script_type": output.scriptpubkey_type,
            "script_hex": output.scriptpubkey,
            "script_asm": output.scriptpubkey_asm,
            "execution": script_result,
        })
    }).collect();

    let inputs: Vec<Value> = tx.vin.iter().enumerate().map(|(i, input)| {
        let is_coinbase = input.is_coinbase.unwrap_or(false);
        let script_hex = input.scriptsig.clone().unwrap_or_default();

        let execution = if is_coinbase || script_hex.is_empty() {
            json!(null)
        } else {
            let prev_locking_hex = input.prevout.as_ref().map(|p| p.scriptpubkey.as_str());
            json!(process_script(&script_hex, prev_locking_hex))
        };

        json!({
            "index": i,
            "is_coinbase": is_coinbase,
            "prev_txid": input.txid,
            "prev_vout": input.vout,
            "script_hex": script_hex,
            "script_asm": input.scriptsig_asm,
            "witness": input.witness,
            "sequence": input.sequence,
            "prevout": input.prevout.as_ref().map(|p| json!({
                "value_sat": p.value,
                "value_btc": p.value as f64 / 100_000_000.0,
                "address": p.scriptpubkey_address,
                "script_type": p.scriptpubkey_type,
            })),
            "execution": execution,
        })
    }).collect();

    let response = json!({
        "txid": tx.txid,
        "version": tx.version,
        "locktime": tx.locktime,
        "locktime_human": format_locktime(tx.locktime),
        "fee_sat": tx.fee,
        "size_bytes": tx.size,
        "weight": tx.weight,
        "confirmed": tx.status.as_ref().map(|s| s.confirmed),
        "block_height": tx.status.as_ref().and_then(|s| s.block_height),
        "input_count": tx.vin.len(),
        "output_count": tx.vout.len(),
        "inputs": inputs,
        "outputs": outputs,
    });

    (StatusCode::OK, Json(response))
}

// ─── parse_locking_script ────────────────────────────────────────────────────
//
// Parse a locking script (scriptPubKey) without executing it.
// Locking scripts define spending CONDITIONS — they are never meant to run
// alone. Running them through the engine always fails because there's nothing
// on the stack yet (the scriptSig hasn't run). We return the parsed opcodes
// for display in the opcode pipeline, with is_locking: true so the frontend
// can show the correct educational context.

fn parse_locking_script(hex: &str) -> Value {
    let bytes = match hex::decode(hex) {
        Ok(b) => b,
        Err(_) => return json!({ "error": "Invalid hex in script" }),
    };

    let ops = parser::parse_script(&bytes);

    let ops_json: Vec<Value> = ops.iter().map(|op| {
        json!({
            "name": op.name(),
            "explain": op.explain(),
            "data": match op {
                opcode::OpCode::Push(data) => Some(hex::encode(data)),
                _ => None,
            }
        })
    }).collect();

    json!({
        "opcodes": ops_json,
        "steps": [],
        "success": null,
        "failure_reason": null,
        "is_locking": true,
    })
}

// ─── process_script ──────────────────────────────────────────────────────────
//
// Parse AND execute an unlocking script (scriptSig) combined with its
// corresponding locking script (scriptPubKey) from the previous output.
// Bitcoin validates an input by running these sequentially.

fn process_script(unlocking_hex: &str, locking_hex: Option<&str>) -> Value {
    let mut combined_ops = Vec::new();

    // 1. Parse unlocking script (scriptSig)
    if let Ok(bytes) = hex::decode(unlocking_hex) {
        combined_ops.extend(parser::parse_script(&bytes));
    } else {
        return json!({ "error": "Invalid hex in unlocking script" });
    }

    // 2. Parse locking script (scriptPubKey)
    if let Some(l_hex) = locking_hex {
        if let Ok(bytes) = hex::decode(l_hex) {
            combined_ops.extend(parser::parse_script(&bytes));
        }
    }

    let result = engine::execute(&combined_ops);

    let ops_json: Vec<Value> = combined_ops.iter().map(|op| {
        json!({
            "name": op.name(),
            "explain": op.explain(),
            "data": match op {
                opcode::OpCode::Push(data) => Some(hex::encode(data)),
                _ => None,
            }
        })
    }).collect();

    let steps_json: Vec<Value> = result.steps.iter().map(|step| {
        json!({
            "opcode": step.opcode,
            "description": step.description,
            "stack": step.stack,
            "failed": step.failed,
            "explanation": step.explanation,
        })
    }).collect();

    json!({
        "opcodes": ops_json,
        "steps": steps_json,
        "success": result.success,
        "failure_reason": result.failure_reason,
        "is_locking": false,
    })
}

fn format_locktime(lt: u32) -> String {
    if lt == 0 {
        "No lock (immediate)".into()
    } else if lt < 500_000_000 {
        format!("Block height {}", lt)
    } else {
        format!("Unix timestamp {}", lt)
    }
}
