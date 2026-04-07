import { SearchBar } from './SearchBar'
import './LandingHero.css'

interface Props {
  onSearch: (txid: string) => void
  loading: boolean
}

const EXAMPLES = [
  { label: 'First BTC transfer', txid: 'f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16', note: 'Satoshi → Hal Finney' },
  { label: 'Pizza transaction', txid: 'a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d', note: '10,000 BTC for 2 pizzas' },
  { label: 'Genesis coinbase', txid: '4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b', note: 'Block 0 · The beginning' },
]

// The P2PKH execution steps shown in the landing diagram
const P2PKH_STEPS = [
  { op: 'PUSH <sig>',       stack: ['<sig>'],                              color: 'var(--op-push)',   note: 'Spender provides signature' },
  { op: 'PUSH <pubkey>',    stack: ['<sig>', '<pubkey>'],                  color: 'var(--op-push)',   note: 'Spender provides public key' },
  { op: 'OP_DUP',           stack: ['<sig>', '<pubkey>', '<pubkey>'],      color: 'var(--op-flow)',   note: 'Duplicate pubkey for hashing' },
  { op: 'OP_HASH160',       stack: ['<sig>', '<pubkey>', 'hash160(pk)'],   color: 'var(--op-hash)',   note: 'Hash the pubkey copy' },
  { op: 'PUSH <pkHash>',    stack: ['<sig>', '<pubkey>', 'hash160(pk)', '<pkHash>'], color: 'var(--op-push)', note: 'Address hash from locking script' },
  { op: 'OP_EQUALVERIFY',   stack: ['<sig>', '<pubkey>'],                  color: 'var(--op-verify)', note: 'Verify hashes match → addresses match' },
  { op: 'OP_CHECKSIG',      stack: ['TRUE'],                              color: 'var(--op-sig)',    note: 'Verify signature → script succeeds' },
]

export function LandingHero({ onSearch, loading }: Props) {
  return (
    <div className="hero">
      <div className="hero-left">
        <div className="hero-eyebrow">Bitcoin Script VM</div>
        <h1 className="hero-title">
          See every opcode.<br />
          <em>Understand every byte.</em>
        </h1>
        <p className="hero-desc">
          Paste any Bitcoin transaction ID. Get a full step-by-step execution trace
          of its locking and unlocking scripts — with real SHA-256 + RIPEMD-160 crypto
          and plain-English explanations of what each opcode is doing and why.
        </p>

        <SearchBar onSearch={onSearch} loading={loading} />

        <div className="examples">
          <div className="examples-label">Try these:</div>
          {EXAMPLES.map((ex) => (
            <button key={ex.txid} className="example-btn" onClick={() => onSearch(ex.txid)}>
              <span className="example-name">{ex.label}</span>
              <span className="example-note">{ex.note}</span>
              <span className="example-txid mono">{ex.txid.slice(0, 10)}…</span>
            </button>
          ))}
        </div>
      </div>

      <div className="hero-right">
        <div className="diagram-label">P2PKH execution flow</div>
        <div className="p2pkh-diagram">
          {P2PKH_STEPS.map((step, i) => (
            <div key={i} className="diagram-step" style={{ animationDelay: `${i * 60}ms` }}>
              <div className="diagram-op" style={{ borderColor: step.color, color: step.color }}>
                <span className="mono">{step.op}</span>
              </div>
              <div className="diagram-stack">
                {step.stack.map((item, j) => (
                  <span
                    key={j}
                    className={`stack-item mono ${j === step.stack.length - 1 ? 'stack-top' : ''}`}
                  >
                    {item}
                  </span>
                ))}
              </div>
              <div className="diagram-note">{step.note}</div>
            </div>
          ))}
          <div className="diagram-result">
            <span className="result-dot" />
            Stack top is TRUE — script valid, output can be spent
          </div>
        </div>
      </div>
    </div>
  )
}
