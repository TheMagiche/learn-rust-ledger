import { useState } from "react"

import type { Expense } from "../types"

type ExpenseTableProps = {
  expenses: Expense[]
  loading: boolean
  refreshing: boolean
  error: string | null
  onRetry: () => void
  onDelete: (id: number) => Promise<boolean>
}

export function ExpenseTable({ expenses, loading, refreshing, error, onRetry, onDelete }: ExpenseTableProps) {
  const [pendingId, setPendingId] = useState<number | null>(null)

  if (loading) {
    return <p className="rounded-xl border border-dashed border-line px-4 py-8 text-muted dark:border-night-line dark:text-night-muted">Loading expenses…</p>
  }

  return (
    <div className="overflow-hidden rounded-xl border border-line bg-card dark:border-night-line dark:bg-night-card">
      {error ? (
        <div role="alert" className="flex flex-wrap items-center justify-between gap-3 border-b border-line bg-copper-soft px-4 py-3 text-sm text-ink dark:border-night-line dark:bg-night-soft dark:text-night-ink">
          <span>{error}</span>
          <button type="button" className="font-semibold underline" onClick={onRetry}>
            Retry
          </button>
        </div>
      ) : null}
      <div className="flex items-center justify-between px-4 py-3 text-sm text-muted dark:text-night-muted">
        <span>{refreshing ? "Updating the list…" : `${expenses.length} shown`}</span>
      </div>
      {expenses.length === 0 ? (
        <p className="px-4 pb-8 text-ink dark:text-night-ink">
          No expenses match these filters. Record one and the handler that stores it will show up in the Rust panel.
        </p>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full min-w-[640px] border-collapse text-left text-sm">
            <caption className="sr-only">Expenses</caption>
            <thead className="text-xs uppercase tracking-wide text-muted dark:text-night-muted">
              <tr>
                <th className="px-4 py-2 font-semibold">Id</th>
                <th className="px-4 py-2 font-semibold">Date</th>
                <th className="px-4 py-2 font-semibold">Category</th>
                <th className="px-4 py-2 text-right font-semibold">Amount</th>
                <th className="px-4 py-2 font-semibold">Note</th>
                <th className="px-4 py-2 font-semibold"> </th>
              </tr>
            </thead>
            <tbody>
              {expenses.map((expense) => (
                <tr key={expense.id} className="border-t border-line dark:border-night-line">
                  <td className="px-4 py-3 font-mono">{expense.id}</td>
                  <td className="px-4 py-3">{expense.date}</td>
                  <td className="px-4 py-3 capitalize">{expense.category}</td>
                  <td className="px-4 py-3 text-right font-mono">{expense.display}</td>
                  <td className="px-4 py-3">{expense.note ?? "—"}</td>
                  <td className="px-4 py-3 text-right">
                    {pendingId === expense.id ? (
                      <span className="inline-flex gap-2">
                        <button
                          type="button"
                          className="font-semibold text-danger"
                          onClick={() => {
                            void onDelete(expense.id).then((ok) => {
                              if (ok) {
                                setPendingId(null)
                              }
                            })
                          }}
                        >
                          Confirm
                        </button>
                        <button type="button" className="text-muted dark:text-night-muted" onClick={() => setPendingId(null)}>
                          Keep
                        </button>
                      </span>
                    ) : (
                      <button type="button" className="font-semibold text-copper" onClick={() => setPendingId(expense.id)}>
                        Delete
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
