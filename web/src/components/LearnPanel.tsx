import { useState } from "react"

import { compilerErrorByConcepts } from "../learn/compilerErrors"
import { conceptById } from "../learn/concepts"
import type { Trace } from "../types"
import { CompilerErrorCard } from "./CompilerErrorCard"
import { ConceptCard } from "./ConceptCard"
import { RequestInspector } from "./RequestInspector"
import { RustCodeViewer } from "./RustCodeViewer"

type LearnPanelProps = {
  traces: Trace[]
}

export function LearnPanel({ traces }: LearnPanelProps) {
  const [open, setOpen] = useState(true)
  const [selected, setSelected] = useState(0)
  const [whatIf, setWhatIf] = useState(false)
  const [focusedConcept, setFocusedConcept] = useState<string | null>(null)
  const index = Math.min(selected, Math.max(traces.length - 1, 0))
  const trace = traces[index]

  return (
    <aside id="learn" className="min-w-0 lg:sticky lg:top-4 lg:max-h-[calc(100vh-2rem)] lg:overflow-auto">
      <div className="rounded-xl border border-line bg-card dark:border-night-line dark:bg-night-card">
        <div className="flex items-center justify-between gap-3 border-b border-line px-4 py-3 dark:border-night-line">
          <div>
            <h2 className="font-serif text-2xl">What Rust just did</h2>
            <p className="text-sm text-muted dark:text-night-muted">Request, handler, and the ideas in that code.</p>
          </div>
          <button type="button" className="text-sm font-semibold text-copper" onClick={() => setOpen((current) => !current)} aria-expanded={open}>
            {open ? "Hide" : "Show"}
          </button>
        </div>
        {open ? (
          <div className="grid min-w-0 gap-4 p-4">
            {trace ? (
              <>
                <RequestInspector trace={trace} />
                <div className="min-w-0">
                  <h3 className="font-serif text-xl">Handler</h3>
                  <p className="text-xs font-mono text-muted dark:text-night-muted">
                    X-Rust-Handler: {trace.handler ?? "—"}
                  </p>
                  <div className="mt-2 min-w-0">
                    <RustCodeViewer handler={trace.handler} onConcept={setFocusedConcept} />
                  </div>
                </div>
                <div className="grid gap-3">
                  <h3 className="font-serif text-xl">Concepts</h3>
                  {trace.concepts.length === 0 ? (
                    <p className="text-sm text-muted dark:text-night-muted">This response did not name any concepts.</p>
                  ) : (
                    trace.concepts.map((id) => {
                      const concept = conceptById(id)
                      return concept ? (
                        <ConceptCard key={id} concept={concept} />
                      ) : (
                        <p key={id} className="text-sm text-danger">
                          Missing concept {id}
                        </p>
                      )
                    })
                  )}
                  {focusedConcept && !trace.concepts.includes(focusedConcept) ? (
                    <Focused conceptId={focusedConcept} />
                  ) : null}
                </div>
                <WhatIf concepts={trace.concepts} open={whatIf} onToggle={() => setWhatIf((current) => !current)} />
              </>
            ) : (
              <p className="text-sm text-muted dark:text-night-muted">Traces show up here after the first request.</p>
            )}
            <History traces={traces} selected={index} onSelect={setSelected} />
          </div>
        ) : null}
      </div>
    </aside>
  )
}

function Focused({ conceptId }: { conceptId: string }) {
  const concept = conceptById(conceptId)
  if (!concept) {
    return <p className="text-sm text-danger">No glossary entry for {conceptId}.</p>
  }
  return (
    <div>
      <p className="text-xs font-semibold uppercase tracking-wide text-muted dark:text-night-muted">From the LEARN comment</p>
      <div className="mt-2">
        <ConceptCard concept={concept} />
      </div>
    </div>
  )
}

function WhatIf({ concepts, open, onToggle }: { concepts: string[]; open: boolean; onToggle: () => void }) {
  const example = compilerErrorByConcepts(concepts)
  if (!example) {
    return null
  }
  return (
    <div>
      <button type="button" className="text-sm font-semibold text-copper" onClick={onToggle} aria-pressed={open}>
        {open ? "Hide the broken version" : "What if this borrow was wrong?"}
      </button>
      {open ? (
        <div className="mt-3">
          <CompilerErrorCard example={example} />
        </div>
      ) : null}
    </div>
  )
}

function History({ traces, selected, onSelect }: { traces: Trace[]; selected: number; onSelect: (index: number) => void }) {
  if (traces.length === 0) {
    return null
  }
  return (
    <div>
      <h3 className="font-serif text-xl">Recent requests</h3>
      <ul className="mt-2 grid gap-1">
        {traces.map((trace, index) => (
          <li key={`${trace.method}-${trace.url}-${index}`}>
            <button
              type="button"
              onClick={() => onSelect(index)}
              className={`flex w-full items-baseline justify-between gap-2 rounded-md px-2 py-1 text-left font-mono text-xs ${
                index === selected ? "bg-copper-soft dark:bg-night-soft" : "hover:bg-paper dark:hover:bg-night"
              }`}
            >
              <span>
                {trace.method} {trace.url}
              </span>
              <span>{trace.status}</span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  )
}
