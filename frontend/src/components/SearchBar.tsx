import { useState, type FormEvent } from 'react'
import './SearchBar.css'

interface Props {
  onSearch: (txid: string) => void
  loading: boolean
  compact?: boolean
}

export function SearchBar({ onSearch, loading, compact }: Props) {
  const [value, setValue] = useState('')

  function handleSubmit(e: FormEvent) {
    e.preventDefault()
    const trimmed = value.trim()
    if (trimmed.length === 64) onSearch(trimmed)
  }

  const invalid = value.length > 0 && value.length !== 64

  return (
    <form className={`search-form ${compact ? 'compact' : ''}`} onSubmit={handleSubmit}>
      <div className={`search-wrap ${invalid ? 'invalid' : ''}`}>
        <span className="search-icon mono">#</span>
        <input
          className="search-input mono"
          type="text"
          placeholder="Paste a Bitcoin transaction ID (64 hex chars)…"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          spellCheck={false}
          autoComplete="off"
          maxLength={64}
        />
        {value.length > 0 && (
          <span className={`search-counter ${invalid ? 'bad' : 'good'}`}>
            {value.length}/64
          </span>
        )}
      </div>
      <button
        className="search-btn"
        type="submit"
        disabled={loading || value.length !== 64}
      >
        {loading ? <span className="spinner" /> : 'Inspect'}
      </button>
    </form>
  )
}
