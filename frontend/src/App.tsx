import { useState } from 'react'
import { useTransaction } from './hooks/useTransaction'
import { SearchBar } from './components/SearchBar'
import { TxMeta } from './components/TxMeta'
import { ScriptPanel } from './components/ScriptPanel'
import { LandingHero } from './components/LandingHero'
import { OpcodeDictionary } from './components/OpcodeDictionary'
import './App.css'

export default function App() {
  const { transaction, loading, error, fetch } = useTransaction()
  const [activeTab, setActiveTab] = useState<'outputs' | 'inputs'>('outputs')
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
            <div className="animate-fade-up">
              <TxMeta tx={transaction} />

              {/* ── Tab switcher ── */}
              <div className="tab-bar">
                <button
                  className={`tab-btn ${activeTab === 'outputs' ? 'active' : ''}`}
                  onClick={() => setActiveTab('outputs')}
                >
                  <span className="tab-label">Outputs</span>
                  <span className="tab-count">{transaction.output_count}</span>
                  <span className="tab-hint">locking scripts</span>
                </button>
                <button
                  className={`tab-btn ${activeTab === 'inputs' ? 'active' : ''}`}
                  onClick={() => setActiveTab('inputs')}
                >
                  <span className="tab-label">Inputs</span>
                  <span className="tab-count">{transaction.input_count}</span>
                  <span className="tab-hint">unlocking scripts</span>
                </button>
              </div>

              {/* ── Explanation banner ── */}
              <div className="concept-banner">
                {activeTab === 'outputs' ? (
                  <>
                    <span className="concept-tag">scriptPubKey</span>
                    <span className="concept-text">
                      A <strong>locking script</strong> — set by the sender, defines conditions
                      that must be satisfied to spend this output. Think of it as a padlock.
                    </span>
                  </>
                ) : (
                  <>
                    <span className="concept-tag">scriptSig</span>
                    <span className="concept-text">
                      An <strong>unlocking script</strong> — provided by the spender, supplies
                      the data that satisfies the locking conditions. The key to the padlock.
                    </span>
                  </>
                )}
              </div>

              {/* ── Script panels ── */}
              <div className="script-list">
                {activeTab === 'outputs'
                  ? transaction.outputs.map((output) => (
                      <ScriptPanel key={output.index} script={output} kind="output" />
                    ))
                  : transaction.inputs.map((input) => (
                      <ScriptPanel key={input.index} script={input} kind="input" />
                    ))
                }
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
