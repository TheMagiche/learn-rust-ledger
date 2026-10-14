import type { Summary as SummaryData } from "../types"

type SummaryProps = {
  summary: SummaryData | null
  count: number
  loading: boolean
}

export function Summary({ summary, count, loading }: SummaryProps) {
  const biggest = summary?.byCategory.reduce<(typeof summary.byCategory)[number] | null>((best, row) => {
    if (!best || row.amount > best.amount) {
      return row
    }
    return best
  }, null)

  const cards = [
    {
      label: "Month total",
      value: loading && !summary ? "…" : (summary?.total.display ?? "0.00"),
      detail: "From the summary handler",
    },
    {
      label: "Showing",
      value: loading && !summary ? "…" : String(count),
      detail: "Rows matching the filters",
    },
    {
      label: "Biggest category",
      value: biggest ? biggest.category : "—",
      detail: biggest ? biggest.display : "Nothing spent this month",
    },
  ]

  return (
    <section className="grid gap-3 sm:grid-cols-3" aria-label="Month summary">
      {cards.map((card) => (
        <article key={card.label} className="rounded-xl border border-line bg-card px-4 py-3 dark:border-night-line dark:bg-night-card">
          <p className="text-xs font-semibold uppercase tracking-wide text-muted dark:text-night-muted">{card.label}</p>
          <p className="mt-1 font-serif text-3xl capitalize text-ink dark:text-night-ink">{card.value}</p>
          <p className="text-sm text-muted dark:text-night-muted">{card.detail}</p>
        </article>
      ))}
    </section>
  )
}
