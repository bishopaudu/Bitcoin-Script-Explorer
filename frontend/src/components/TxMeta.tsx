import type { Transaction } from '../types'
import './TxMeta.css'

interface Props {
  tx: Transaction
}

export function TxMeta({ tx }: Props) {
  const totalOut = tx.outputs.reduce((sum, o) => sum + o.value_sat, 0)

  return (
    <div className="tx-meta">
      {/* txid — full width */}
      <div className="txid-row">
        <span className="txid-label mono">txid</span>
        <span className="txid-value mono">{tx.txid}</span>
        <a
          className="txid-link"
          href={`https://mempool.space/tx/${tx.txid}`}
          target="_blank"
          rel="noreferrer"
        >
          mempool.space ↗
        </a>
      </div>

      {/* Stat cards */}
      <div className="meta-cards">
        <StatCard label="Status" value={tx.confirmed ? `Block ${tx.block_height?.toLocaleString()}` : 'Unconfirmed'} accent={tx.confirmed ? 'green' : 'amber'} />
        <StatCard label="Version" value={`v${tx.version}`} note={tx.version === 2 ? 'RBF / timelocks' : ''} />
        <StatCard label="Locktime" value={tx.locktime_human} />
        <StatCard label="Fee" value={tx.fee_sat != null ? `${tx.fee_sat.toLocaleString()} sat` : '—'} />
        <StatCard label="Size" value={tx.size_bytes != null ? `${tx.size_bytes} bytes` : '—'} />
        <StatCard label="Inputs" value={String(tx.input_count)} />
        <StatCard label="Outputs" value={String(tx.output_count)} />
        <StatCard
          label="Total output"
          value={`${(totalOut / 1e8).toFixed(8)} BTC`}
          note={`${totalOut.toLocaleString()} sat`}
          accent="amber"
        />
      </div>
    </div>
  )
}

function StatCard({
  label, value, note, accent
}: {
  label: string
  value: string
  note?: string
  accent?: 'green' | 'amber'
}) {
  return (
    <div className={`stat-card ${accent ? `stat-${accent}` : ''}`}>
      <div className="stat-label">{label}</div>
      <div className="stat-value mono">{value}</div>
      {note && <div className="stat-note mono">{note}</div>}
    </div>
  )
}
