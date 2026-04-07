import { useState } from 'react'
import type { TxOutput, TxInput, ScriptExecution } from '../types'
import { ExecutionTrace } from './ExecutionTrace'
import './ScriptPanel.css'

type Script = TxOutput | TxInput
type Kind = 'output' | 'input'

interface Props {
  script: Script
  kind: Kind
}

// Determine script type label and color
function getTypeStyle(type: string | null | undefined): { label: string; cls: string } {
  switch (type) {
    case 'p2pkh':     return { label: 'P2PKH',   cls: 'type-p2pkh' }
    case 'p2sh':      return { label: 'P2SH',    cls: 'type-p2sh' }
    case 'v0_p2wpkh': return { label: 'P2WPKH',  cls: 'type-segwit' }
    case 'v0_p2wsh':  return { label: 'P2WSH',   cls: 'type-segwit' }
    case 'v1_p2tr':   return { label: 'Taproot',  cls: 'type-taproot' }
    case 'op_return': return { label: 'OP_RETURN',cls: 'type-return' }
    case 'p2pk':      return { label: 'P2PK',    cls: 'type-p2pk' }
    default:          return { label: type ?? 'unknown', cls: 'type-unknown' }
  }
}

function isOutput(_s: Script, kind: Kind): _s is TxOutput {
  return kind === 'output'
}

export function ScriptPanel({ script, kind }: Props) {
  const [expanded, setExpanded] = useState(false)

  const scriptType = isOutput(script, kind)
    ? script.script_type
    : (script as TxInput).prevout?.script_type ?? null

  const { label, cls } = getTypeStyle(scriptType)

  const hex = isOutput(script, kind)
    ? script.script_hex
    : (script as TxInput).script_hex

  const asm = isOutput(script, kind)
    ? script.script_asm
    : (script as TxInput).script_asm

  const execution = script.execution as ScriptExecution | null

  const isCoinbase = !isOutput(script, kind) && (script as TxInput).is_coinbase
  const isSegwit = !isOutput(script, kind) && (script as TxInput).witness?.length > 0 && !hex

  // Value display
  let valueDisplay = ''
  if (isOutput(script, kind)) {
    valueDisplay = `${(script.value_btc).toFixed(8)} BTC`
  } else {
    const inp = script as TxInput
    if (inp.prevout) {
      valueDisplay = `${inp.prevout.value_btc.toFixed(8)} BTC`
    }
  }

  // Address display
  const address = isOutput(script, kind)
    ? script.address
    : (script as TxInput).prevout?.address ?? null

  return (
    <div className={`script-panel ${expanded ? 'expanded' : ''}`}>
      {/* ── Header ── */}
      <div className="sp-header" onClick={() => setExpanded(!expanded)} role="button" tabIndex={0}
        onKeyDown={(e) => e.key === 'Enter' && setExpanded(!expanded)}>

        <div className="sp-index mono">#{script.index}</div>

        <div className={`sp-type ${cls}`}>{label}</div>

        <div className="sp-address">
          {address
            ? <span className="mono truncate">{address}</span>
            : isCoinbase
              ? <span className="sp-note">coinbase — new BTC created</span>
              : isSegwit
                ? <span className="sp-note">SegWit — witness data (not shown)</span>
                : <span className="sp-note">no address</span>
          }
        </div>

        {valueDisplay && (
          <div className="sp-value mono">{valueDisplay}</div>
        )}

        {execution && (
          <div className={`sp-status ${
            execution.is_locking ? 'lock' : execution.success ? 'ok' : 'fail'
          }`}>
            <span className="sp-status-dot" />
            {execution.is_locking
              ? 'lock'
              : execution.success ? 'valid' : 'failed'}
          </div>
        )}

        <div className={`sp-chevron ${expanded ? 'open' : ''}`}>›</div>
      </div>

      {/* ── Expanded content ── */}
      {expanded && (
        <div className="sp-body animate-fade-up">

          {/* Special cases */}
          {isCoinbase && (
            <div className="sp-notice amber">
              This is a <strong>coinbase input</strong> — it creates new Bitcoin
              out of thin air as a miner reward. It has no previous output to spend.
              The scriptSig contains the miner's arbitrary data (e.g. block height,
              extra nonce, pool tag).
            </div>
          )}

          {isSegwit && (
            <div className="sp-notice blue">
              This is a <strong>SegWit input</strong>. The unlocking data lives in the
              witness field (separate from scriptSig), which is why the script is empty.
              SegWit was introduced in 2017 (BIP 141) to fix transaction malleability
              and reduce fees by discounting witness bytes.
              {(script as TxInput).witness?.length > 0 && (
                <div className="witness-data">
                  <div className="sp-field-label">Witness items ({(script as TxInput).witness.length})</div>
                  {(script as TxInput).witness.map((w, i) => (
                    <div key={i} className="mono witness-item">
                      <span className="witness-idx">[{i}]</span> {w}
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Raw hex */}
          {hex && (
            <div className="sp-field">
              <div className="sp-field-label">Raw script hex</div>
              <div className="sp-hex mono">{hex}</div>
            </div>
          )}

          {/* ASM */}
          {asm && (
            <div className="sp-field">
              <div className="sp-field-label">Disassembly (ASM)</div>
              <div className="sp-asm mono">{asm}</div>
            </div>
          )}

          {/* Execution trace */}
          {execution ? (
            <ExecutionTrace execution={execution} />
          ) : hex && !isCoinbase ? (
            <div className="sp-notice">No execution data available for this script.</div>
          ) : null}

        </div>
      )}
    </div>
  )
}
