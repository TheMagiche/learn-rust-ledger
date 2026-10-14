import { useMemo, useState } from "react"

import { ConceptCard } from "../components/ConceptCard"
import { concepts } from "../learn/concepts"

export function Glossary() {
  const [query, setQuery] = useState("")
  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase()
    if (!needle) {
      return concepts
    }
    return concepts.filter((concept) =>
      [concept.id, concept.title, concept.summary, concept.explanation].join(" ").toLowerCase().includes(needle),
    )
  }, [query])

  return (
    <div className="grid gap-4">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h2 className="font-serif text-4xl">Glossary</h2>
          <p className="mt-1 text-muted dark:text-night-muted">{concepts.length} ideas used in this repo.</p>
        </div>
        <label className="grid gap-1 text-sm font-semibold">
          Search
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="borrow, Result, cents"
            className="w-64 max-w-full rounded-md border border-line bg-card px-3 py-2 dark:border-night-line dark:bg-night dark:text-night-ink"
          />
        </label>
      </header>
      {filtered.length === 0 ? (
        <p>No concept matches “{query}”.</p>
      ) : (
        <div className="grid gap-3 md:grid-cols-2">
          {filtered.map((concept) => (
            <ConceptCard key={concept.id} concept={concept} />
          ))}
        </div>
      )}
    </div>
  )
}
