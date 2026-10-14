import { useState, type FormEvent } from "react"

import { parseAmount } from "../lib/money"
import { CATEGORIES, type BudgetStatus } from "../types"

const fieldClass =
  "w-full rounded-md border border-line bg-card px-3 py-2 outline-none ring-copper/30 focus:ring-2 dark:border-night-line dark:bg-night dark:text-night-ink"

type BudgetBarProps = {
  budgets: BudgetStatus[]
  saving: boolean
  onSetBudget: (category: string, amount: string) => Promise<boolean>
}

export function BudgetBar({ budgets, saving, onSetBudget }: BudgetBarProps) {
  const [category, setCategory] = useState<string>(CATEGORIES[0])
  const [amount, setAmount] = useState("")
  const [fieldError, setFieldError] = useState<string | null>(null)

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const parsed = parseAmount(amount)
    if (!parsed.ok) {
      setFieldError(parsed.error)
      return
    }
    setFieldError(null)
    const saved = await onSetBudget(category, amount.trim())
    if (saved) {
      setAmount("")
    }
  }

  return (
    <section className="rounded-xl border border-line bg-card p-4 dark:border-night-line dark:bg-night-card">
      <h2 className="font-serif text-2xl">Budgets</h2>
      {budgets.length === 0 ? (
        <p className="mt-3 text-sm text-muted dark:text-night-muted">No caps yet. Set a monthly amount and spending turns the bar red when it passes the cap.</p>
      ) : (
        <ul className="mt-3 grid gap-3">
          {budgets.map((budget) => {
            const ratio = budget.budget.amount === 0 ? (budget.spent.amount > 0 ? 1 : 0) : Math.min(budget.spent.amount / budget.budget.amount, 1)
            return (
              <li key={budget.category}>
                <div className="flex items-baseline justify-between gap-3 text-sm">
                  <span className="capitalize font-semibold">{budget.category}</span>
                  <span className={budget.over ? "font-semibold text-danger" : "text-muted dark:text-night-muted"}>
                    {budget.spent.display} / {budget.budget.display}
                    {budget.over ? " · Over" : ""}
                  </span>
                </div>
                <div className="mt-1 h-2 overflow-hidden rounded-full bg-line dark:bg-night-line" aria-hidden="true">
                  <div
                    className={budget.over ? "h-full bg-danger" : "h-full bg-pine"}
                    style={{ width: `${Math.round(ratio * 100)}%` }}
                  />
                </div>
              </li>
            )
          })}
        </ul>
      )}
      <form onSubmit={handleSubmit} className="mt-4 grid gap-2 sm:grid-cols-[1fr_1fr_auto] sm:items-end">
        <label className="grid gap-1 text-sm font-semibold">
          Category
          <select className={fieldClass} value={category} onChange={(event) => setCategory(event.target.value)}>
            {CATEGORIES.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
        </label>
        <label className="grid gap-1 text-sm font-semibold">
          Monthly cap
          <input className={fieldClass} value={amount} onChange={(event) => setAmount(event.target.value)} placeholder="150.00" inputMode="decimal" />
        </label>
        <button type="submit" disabled={saving} className="rounded-md bg-pine px-3 py-2 font-semibold text-white disabled:opacity-60">
          Set budget
        </button>
      </form>
      {fieldError ? (
        <p role="alert" className="mt-2 text-sm text-danger">
          {fieldError}
        </p>
      ) : null}
    </section>
  )
}
