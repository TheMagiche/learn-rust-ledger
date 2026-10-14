import { useState } from "react"

import type { CompilerErrorExample } from "../learn/compilerErrors"

type CompilerErrorCardProps = {
  example: CompilerErrorExample
}

export function CompilerErrorCard({ example }: CompilerErrorCardProps) {
  const [showFix, setShowFix] = useState(false)

  return (
    <article className="rounded-xl border border-line bg-card p-4 dark:border-night-line dark:bg-night-card">
      <header className="flex flex-wrap items-baseline justify-between gap-2">
        <h3 className="font-serif text-2xl">{example.title}</h3>
        <span className="font-mono text-sm text-copper">{example.code}</span>
      </header>
      <p className="mt-2 text-sm text-muted dark:text-night-muted">{example.explanation}</p>
      <p className="mt-4 text-xs font-semibold uppercase tracking-wide text-muted dark:text-night-muted">Broken</p>
      <pre className="mt-1 overflow-x-auto rounded-md bg-paper p-3 font-mono text-xs dark:bg-night">{example.broken}</pre>
      <p className="mt-4 text-xs font-semibold uppercase tracking-wide text-muted dark:text-night-muted">rustc</p>
      <pre className="mt-1 overflow-x-auto rounded-md bg-ink p-3 font-mono text-xs text-paper dark:bg-black">{example.rustc}</pre>
      <button type="button" className="mt-3 text-sm font-semibold text-pine" onClick={() => setShowFix((current) => !current)}>
        {showFix ? "Hide the fix" : "Show the fix"}
      </button>
      {showFix ? <pre className="mt-2 overflow-x-auto rounded-md bg-paper p-3 font-mono text-xs dark:bg-night">{example.fix}</pre> : null}
    </article>
  )
}
