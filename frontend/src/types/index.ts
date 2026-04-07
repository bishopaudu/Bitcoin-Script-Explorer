// types/index.ts — All TypeScript types mirroring the Rust backend JSON output

export interface ExecutionStep {
  opcode: string;
  description: string;
  stack: string[];
  failed: boolean;
  explanation: string;
}

export interface ParsedOpcode {
  name: string;
  explain: string;
  data: string | null;
}

export interface ScriptExecution {
  opcodes: ParsedOpcode[];
  steps: ExecutionStep[];
  /** true = locking script (scriptPubKey), engine not run, success is null */
  is_locking: boolean;
  /** null when is_locking=true (no execution ran), bool otherwise */
  success: boolean | null;
  failure_reason: string | null;
}

export interface TxOutput {
  index: number;
  value_sat: number;
  value_btc: number;
  address: string | null;
  script_type: string | null;
  script_hex: string;
  script_asm: string | null;
  execution: ScriptExecution | null;
}

export interface TxInput {
  index: number;
  is_coinbase: boolean;
  prev_txid: string | null;
  prev_vout: number | null;
  script_hex: string;
  script_asm: string | null;
  witness: string[];
  sequence: number | null;
  prevout: {
    value_sat: number;
    value_btc: number;
    address: string | null;
    script_type: string | null;
  } | null;
  execution: ScriptExecution | null;
}

export interface Transaction {
  txid: string;
  version: number;
  locktime: number;
  locktime_human: string;
  fee_sat: number | null;
  size_bytes: number | null;
  weight: number | null;
  confirmed: boolean | null;
  block_height: number | null;
  input_count: number;
  output_count: number;
  inputs: TxInput[];
  outputs: TxOutput[];
}

export type ScriptType =
  | 'p2pkh'
  | 'p2sh'
  | 'v0_p2wpkh'
  | 'v0_p2wsh'
  | 'v1_p2tr'
  | 'op_return'
  | 'p2pk'
  | 'unknown';
