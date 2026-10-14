import type { Concept } from "../learn/concepts"

type ConceptCardProps = {
  concept: Concept
}

export function ConceptCard({ concept }: ConceptCardProps) {
  return (
    <article id={`concept-${concept.id}`} className="min-w-0 rounded-lg border border-line bg-paper p-3 dark:border-night-line dark:bg-night">
      <h3 className="font-serif text-xl">{concept.title}</h3>
      <p className="mt-1 text-sm font-semibold">{concept.summary}</p>
      <p className="mt-2 text-sm text-muted dark:text-night-muted">{concept.explanation}</p>
      <pre className="mt-2 max-w-full overflow-x-auto rounded-md bg-card px-3 py-2 font-mono text-xs dark:bg-night-card">{concept.example}</pre>
      <p className="mt-2 flex flex-wrap gap-3 text-sm">
        <a className="font-semibold text-copper underline" href={concept.bookLink} target="_blank" rel="noreferrer">
          Rust Book
        </a>
        <a className="font-semibold text-pine underline" href={concept.exampleLink} target="_blank" rel="noreferrer">
          Rust by Example
        </a>
      </p>
      {concept.usedIn.length > 0 ? (
        <p className="mt-2 font-mono text-xs text-muted dark:text-night-muted">{concept.usedIn.slice(0, 4).join(" · ")}</p>
      ) : null}
    </article>
  )
}
