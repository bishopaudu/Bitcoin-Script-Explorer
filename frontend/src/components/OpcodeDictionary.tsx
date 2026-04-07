import { useState, useEffect } from 'react'
import './OpcodeDictionary.css'

interface OpcodeDef {
  name: string;
  explain: string;
  hex: string | null;
}

interface Props {
  isOpen: boolean;
  onClose: () => void;
}

export function OpcodeDictionary({ isOpen, onClose }: Props) {
  const [opcodes, setOpcodes] = useState<OpcodeDef[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  // Fetch dictionary from backend
  useEffect(() => {
    if (isOpen && opcodes.length === 0) {
      setIsLoading(true)
      fetch('http://localhost:3001/api/opcodes')
        .then(res => res.json())
        .then(data => {
          setOpcodes(data.opcodes)
          setIsLoading(false)
        })
        .catch(err => {
          console.error("Failed to load opcodes", err)
          setIsLoading(false)
        })
    }
  }, [isOpen, opcodes.length])

  // Escape key closes modal
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    if (isOpen) window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, onClose])

  if (!isOpen) return null;

  const filteredOpcodes = opcodes.filter(op => 
    op.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    op.explain.toLowerCase().includes(searchQuery.toLowerCase())
  )

  return (
    <div className="dictionary-overlay" onClick={onClose}>
      <div className="dictionary-modal" onClick={e => e.stopPropagation()}>
        
        <div className="dict-header">
          <div className="dict-title">📘 Opcode Glossary</div>
          <button className="dict-close" onClick={onClose}>×</button>
        </div>

        <div className="dict-search-wrapper">
          <input 
            type="text" 
            className="dict-search" 
            placeholder="Search opcodes or explanations..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            autoFocus
          />
        </div>

        <div className="dict-body">
          {isLoading ? (
            <div className="dict-loading">Loading opcodes from backend...</div>
          ) : filteredOpcodes.length === 0 ? (
            <div className="dict-empty">No opcodes found matching "{searchQuery}"</div>
          ) : (
            <div className="dict-grid">
              {filteredOpcodes.map(op => (
                <div key={op.name} className="dict-card animate-fade-up">
                  <div className="dict-card-header">
                    <span className="dict-op-name mono">{op.name}</span>
                    {op.hex && <span className="dict-op-hex mono">{op.hex}</span>}
                  </div>
                  <div className="dict-op-explain">
                    {op.explain}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

      </div>
    </div>
  )
}
