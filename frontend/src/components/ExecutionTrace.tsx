import { useState, useEffect, useRef } from 'react'
import type { ScriptExecution } from '../types'
import './ExecutionTrace.css'

interface Props {
  execution: ScriptExecution
}

// Color each opcode type consistently
function opcodeClass(name: string): string {
  if (name === 'PUSH' || name.startsWith('PUSH[')) return 'op-push'
  if (name === 'OP_HASH160') return 'op-hash'
  if (name === 'OP_EQUALVERIFY' || name === 'OP_EQUAL') return 'op-verify'
  if (name === 'OP_CHECKSIG' || name === 'OP_CHECKMULTISIG') return 'op-sig'
  if (name === 'OP_DUP' || name === 'OP_0') return 'op-flow'
  if (name.startsWith('ERROR') || name.startsWith('UNKNOWN')) return 'op-error'
  return 'op-other'
}

export function ExecutionTrace({ execution }: Props) {
  // We use standard React state for playback
  const [currentStepIndex, setCurrentStepIndex] = useState<number>(-1) // -1 means "Before execution starts"
  const [isPlaying, setIsPlaying] = useState(false)
  
  // Ref for the auto-scroll of the stack container
  const stackEndRef = useRef<HTMLDivElement>(null)

  const stepsCount = execution.steps ? execution.steps.length : 0

  // Handle auto-play
  useEffect(() => {
    if (!isPlaying) return

    const intervalId = setInterval(() => {
      setCurrentStepIndex((prev) => {
        if (prev >= stepsCount - 1) {
          setIsPlaying(false)
          return prev
        }
        return prev + 1
      })
    }, 1200) // 1.2 seconds per step for easy reading

    return () => clearInterval(intervalId)
  }, [isPlaying, stepsCount])

  // Scroll to bottom of stack when it changes
  useEffect(() => {
    if (stackEndRef.current) {
      stackEndRef.current.scrollIntoView({ behavior: 'smooth' })
    }
  }, [currentStepIndex])

  // Reset playback if transaction changes
  useEffect(() => {
    setCurrentStepIndex(-1)
    setIsPlaying(false)
  }, [execution])

  // ── Locking script (scriptPubKey) ──────────────────────────────────────────
  if (execution.is_locking) {
    return (
      <div className="exec-trace">
        <div className="exec-header">
          <div className="exec-title">Script opcodes</div>
          <div className="exec-result locking">
            <span className="exec-result-dot" />
            Locking script — conditions set by sender
          </div>
        </div>

        <div className="op-pipeline">
          {execution.opcodes.map((op, i) => (
            <div key={i} className={`pipeline-op ${opcodeClass(op.name)}`} title={op.explain}>
              <span className="pipeline-op-name">{op.name}</span>
              {op.data && (
                <span className="pipeline-op-data">
                  {op.data.length > 8 ? op.data.slice(0, 8) + '…' : op.data}
                </span>
              )}
            </div>
          ))}
          <div className="pipeline-result locking">LOCK</div>
        </div>

        <div className="locking-note">
          <span className="locking-note-icon">🔒</span>
          <span>
            This <strong>scriptPubKey</strong> defines the spending conditions placed
            on this output. It cannot be evaluated alone — Bitcoin concatenates the
            spender's <strong>scriptSig</strong> (unlocking script) before this script
            and runs them as one combined program. The inputs tab shows the unlocking
            side of matching transactions.
          </span>
        </div>
      </div>
    )
  }

  // ── Unlocking script (scriptSig) ───────────────────────────────────────────
  
  // Decide what to display based on currentStepIndex
  const hasStarted = currentStepIndex >= 0
  const isFinished = currentStepIndex === stepsCount - 1
  const currentStep = hasStarted && currentStepIndex < stepsCount 
                      ? execution.steps[currentStepIndex] 
                      : null
  
  // Calculate what opcodes are active based on steps executed so far.
  // Note: the `execution.opcodes` array has exactly `stepsCount` items in normal cases.
  // We highlight opcodes up to `currentStepIndex`.
  const activeOpcodeIndex = currentStepIndex

  const currentStack = currentStep ? currentStep.stack : []

  return (
    <div className="exec-trace exec-trace-animated">
      
      {/* ── Top Controls & Pipeline ── */}
      <div className="player-top">
        <div className="player-header">
          <div className="exec-title">Virtual Machine Playback</div>
          <div className="playback-controls">
            <button 
              className="ctrl-btn ctrl-reset" 
              onClick={() => { setCurrentStepIndex(-1); setIsPlaying(false); }}
              title="Restart"
              disabled={!hasStarted}
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
            </button>
            <button 
              className="ctrl-btn ctrl-step" 
              onClick={() => { setIsPlaying(false); setCurrentStepIndex(p => Math.max(-1, p - 1)); }}
              disabled={!hasStarted}
              title="Step Back"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="m15 18-6-6 6-6"/></svg>
            </button>
            <button 
              className="ctrl-btn ctrl-play" 
              onClick={() => {
                if (isFinished) setCurrentStepIndex(-1);
                setIsPlaying(!isPlaying);
              }}
              title={isPlaying ? "Pause" : "Play"}
            >
              {isPlaying ? (
                <svg viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
              ) : (
                <svg viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>
              )}
            </button>
            <button 
              className="ctrl-btn ctrl-step" 
              onClick={() => { setIsPlaying(false); setCurrentStepIndex(p => Math.min(stepsCount - 1, p + 1)); }}
              disabled={isFinished}
              title="Step Forward"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="m9 18 6-6-6-6"/></svg>
            </button>
          </div>
          
          {isFinished && (
            <div className={`exec-result ${execution.success ? 'ok' : 'fail'}`}>
              <span className="exec-result-dot" />
              {execution.success ? 'Valid' : 'Failed'}
            </div>
          )}
        </div>

        <div className="op-pipeline pipeline-interactive">
          {execution.opcodes.map((op, i) => {
            const isExecuted = i <= activeOpcodeIndex
            const isCurrent = i === activeOpcodeIndex
            return (
              <button
                key={i}
                className={`pipeline-op ${opcodeClass(op.name)} ${isExecuted ? 'executed' : 'pending'} ${isCurrent ? 'current-pulse' : ''}`}
                onClick={() => { setIsPlaying(false); setCurrentStepIndex(i); }}
              >
                <span className="pipeline-op-name">{op.name}</span>
                {op.data && (
                  <span className="pipeline-op-data">
                    {op.data.length > 8 ? op.data.slice(0, 8) + '…' : op.data}
                  </span>
                )}
              </button>
            )
          })}
          <div className={`pipeline-result ${isFinished ? (execution.success ? 'ok' : 'fail') : 'pending'}`}>
            {isFinished ? (execution.success ? 'TRUE' : 'FAIL') : '...'}
          </div>
        </div>
      </div>

      {/* ── Lower Panels: Visualizer & Text ── */}
      <div className="player-body">
        
        {/* Left pane: Plain English Explanation */}
        <div className="player-explanation">
          {!hasStarted ? (
            <div className="explain-empty">
              Click <strong>Play</strong> or <strong>Step Forward</strong> to watch the virtual machine evaluate this script.
            </div>
          ) : (
             <div className="animate-fade-in explain-content">
               <div className="explain-step-num">Step {currentStepIndex + 1} of {stepsCount}</div>
               <div className={`step-op-badge ${opcodeClass(currentStep!.opcode)}`}>
                 {currentStep!.opcode}
               </div>
               <div className="explain-action">
                 {currentStep!.description}
               </div>
               {currentStep!.failed && (
                 <div className="explain-fail-msg">
                   ⚠️ Script marked as INVALID.
                 </div>
               )}
               <div className="explain-theory">
                 <strong>Theory:</strong> {currentStep!.explanation}
               </div>
             </div>
          )}
        </div>

        {/* Right pane: Animated Stack Viewer */}
        <div className="player-stack-view">
          <div className="stack-header">Stack State (Top is Bottom)</div>
          <div className="stack-container">
            {currentStack.map((item, j) => {
              const isTop = j === currentStack.length - 1;
              return (
                <div key={`${j}-${item}`} className="stack-row animate-slide-up">
                  <div className="stack-idx mono">[{j}]</div>
                  <div className={`stack-val mono ${isTop ? 'stack-val-top' : ''}`}>
                    0x{item.length > 40 ? item.slice(0, 38) + '…' : item}
                  </div>
                  {isTop && <div className="stack-pointer">← TOP</div>}
                </div>
              );
            })}
            <div ref={stackEndRef} />
            
            {hasStarted && currentStack.length === 0 && (
              <div className="stack-empty-msg animate-fade-in">Empty stack</div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

