import { useState } from 'react'
import { useTransaction } from './hooks/useTransaction'
import { SearchBar } from './components/SearchBar'
import { TxMeta } from './components/TxMeta'
import { ExecutionTrace } from './components/ExecutionTrace'
import { LandingHero } from './components/LandingHero'
import { OpcodeDictionary } from './components/OpcodeDictionary'
import './App.css'

export default function App() {
  const { transaction, loading, error, fetch } = useTransaction()
  const [activeScript, setActiveScript] = useState<{ kind: 'input' | 'output', index: number }>({ kind: 'input', index: 0 })
  const [isDictionaryOpen, setIsDictionaryOpen] = useState(false)

  return (
    <div className="app">
      {/* ── Header ── */}
      <header className="app-header">
        <div className="container header-inner">
          <div className="logo">
            <span className="logo-symbol">₿</span>
            <div className="logo-text">
              <span className="logo-title">Script Explorer</span>
              <span className="logo-sub">Bitcoin VM · Real Transactions · Real Crypto</span>
            </div>
          </div>
          <div className="header-actions">
            <button 
              className="dict-nav-btn" 
              onClick={() => setIsDictionaryOpen(true)}
              title="Open Opcode Dictionary"
            >
              📘 Opcode Glossary
            </button>
            <div className="header-search">
              <SearchBar onSearch={fetch} loading={loading} compact />
            </div>
          </div>
        </div>
      </header>

      <main className="app-main">
        <div className="container">

          {/* ── No transaction yet — show landing ── */}
          {!transaction && !loading && !error && (
            <LandingHero onSearch={fetch} loading={loading} />
          )}

          {/* ── Loading ── */}
          {loading && (
            <div className="loading-state animate-fade-up">
              <div className="spinner" />
              <span>Fetching transaction from mempool.space…</span>
            </div>
          )}

          {/* ── Error ── */}
          {error && (
            <div className="error-state animate-fade-up">
              <div className="error-icon">!</div>
              <div>
                <div className="error-title">Could not load transaction</div>
                <div className="error-msg">{error}</div>
              </div>
            </div>
          )}

          {/* ── Transaction loaded ── */}
          {transaction && !loading && (
            <div className="workspace animate-fade-up">
              <TxMeta tx={transaction} />

              <div className="workspace-grid">
                {/* ── LEFT SIDEBAR ── */}
                <aside className="workspace-sidebar">
                  {transaction.inputs.length > 0 && (
                    <div className="sidebar-section">
                      <h3 className="sidebar-title">Unlocking Scripts (Inputs)</h3>
                      <div className="sidebar-list">
                        {transaction.inputs.map((input) => (
                          <button
                            key={input.index}
                            className={`sidebar-item ${activeScript.kind === 'input' && activeScript.index === input.index ? 'active' : ''}`}
                            onClick={() => setActiveScript({ kind: 'input', index: input.index })}
                          >
                            <span className="sidebar-item-label">Input #{input.index}</span>
                            <span className="sidebar-item-type">{input.is_coinbase ? 'Coinbase' : (input.witness && input.witness.length > 0 ? 'Witness' : 'Legacy')}</span>
                          </button>
                        ))}
                      </div>
                    </div>
                  )}

                  {transaction.outputs.length > 0 && (
                    <div className="sidebar-section">
                      <h3 className="sidebar-title">Locking Scripts (Outputs)</h3>
                      <div className="sidebar-list">
                        {transaction.outputs.map((output) => (
                          <button
                            key={output.index}
                            className={`sidebar-item ${activeScript.kind === 'output' && activeScript.index === output.index ? 'active' : ''}`}
                            onClick={() => setActiveScript({ kind: 'output', index: output.index })}
                          >
                            <span className="sidebar-item-label">Output #{output.index}</span>
                            <span className="sidebar-item-type">{output.script_type || 'Unknown'}</span>
                          </button>
                        ))}
                      </div>
                    </div>
                  )}
                </aside>

                {/* ── MAIN STAGE ── */}
                <section className="workspace-main">
                  {activeScript.kind === 'input' && transaction.inputs[activeScript.index]?.execution ? (
                    <ExecutionTrace key={`in-${activeScript.index}`} execution={transaction.inputs[activeScript.index].execution!} />
                  ) : activeScript.kind === 'output' && transaction.outputs[activeScript.index]?.execution ? (
                    <ExecutionTrace key={`out-${activeScript.index}`} execution={transaction.outputs[activeScript.index].execution!} />
                  ) : (
                    <div className="missing-script">Select a script from the sidebar</div>
                  )}
                </section>
              </div>
            </div>
          )}
        </div>
      </main>

      <OpcodeDictionary 
        isOpen={isDictionaryOpen} 
        onClose={() => setIsDictionaryOpen(false)} 
      />

      <footer className="app-footer">
        <div className="container">
          <span>Data via mempool.space · Crypto: real SHA256 + RIPEMD160 · OP_CHECKSIG simplified</span>
        </div>
      </footer>
    </div>
  )
}
