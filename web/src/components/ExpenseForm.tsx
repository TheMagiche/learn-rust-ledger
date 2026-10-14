import { useState, type FormEvent } from "react"

import { todayISO } from "../lib/dates"
import { validateExpenseAmount } from "../lib/money"
import { CATEGORIES, type NewExpenseInput } from "../types"

const fieldClass =
  "w-full rounded-md border border-line bg-card px-3 py-2 text-ink outline-none ring-copper/30 placeholder:text-muted focus:ring-2 dark:border-night-line dark:bg-night dark:text-night-ink dark:placeholder:text-night-muted"

type ExpenseFormProps = {
  saving: boolean
  onSubmit: (input: NewExpenseInput) => Promise<boolean>
}

export function ExpenseForm({ saving, onSubmit }: ExpenseFormProps) {
  const [amount, setAmount] = useState("")
  const [category, setCategory] = useState<string>(CATEGORIES[0])
  const [note, setNote] = useState("")
  const [date, setDate] = useState(todayISO)
  const [fieldError, setFieldError] = useState<string | null>(null)

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const parsed = validateExpenseAmount(amount)
    if (!parsed.ok) {
      setFieldError(parsed.error)
      return
    }
    if (!date) {
      setFieldError("Choose a date.")
      return
    }
    setFieldError(null)
    const saved = await onSubmit({ amount: amount.trim(), category, note, date })
    if (saved) {
      setAmount("")
      setNote("")
    }
  }

  return (
    <form onSubmit={handleSubmit} className="grid gap-3 rounded-xl border border-line bg-card p-4 shadow-sm dark:border-night-line dark:bg-night-card">
      <div className="flex items-baseline justify-between gap-3">
        <h2 className="font-serif text-2xl text-ink dark:text-night-ink">Add an expense</h2>
        <p className="text-sm text-muted dark:text-night-muted">450 means 450.00</p>
      </div>
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <label className="grid gap-1 text-sm font-semibold">
          Amount
          <input
            className={fieldClass}
            inputMode="decimal"
            value={amount}
            onChange={(event) => setAmount(event.target.value)}
            placeholder="12.50"
            aria-invalid={fieldError !== null}
            required
          />
        </label>
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
          Note
          <input className={fieldClass} value={note} onChange={(event) => setNote(event.target.value)} placeholder="lunch" />
        </label>
        <label className="grid gap-1 text-sm font-semibold">
          Date
          <input className={fieldClass} type="date" value={date} onChange={(event) => setDate(event.target.value)} required />
        </label>
      </div>
      {fieldError ? (
        <p role="alert" className="text-sm text-danger">
          {fieldError}
        </p>
      ) : null}
      <div>
        <button
          type="submit"
          disabled={saving}
          className="rounded-md bg-copper px-4 py-2 font-semibold text-white disabled:opacity-60"
        >
          {saving ? "Saving…" : "Record expense"}
        </button>
      </div>
    </form>
  )
}
