import { CATEGORIES, type Filters as FilterState } from "../types"

const fieldClass =
  "w-full rounded-md border border-line bg-card px-3 py-2 text-ink outline-none ring-copper/30 focus:ring-2 dark:border-night-line dark:bg-night dark:text-night-ink"

type FiltersProps = {
  filters: FilterState
  onChange: (filters: FilterState) => void
}

export function Filters({ filters, onChange }: FiltersProps) {
  return (
    <div className="grid gap-3 sm:grid-cols-3">
      <label className="grid gap-1 text-sm font-semibold">
        Category
        <select
          className={fieldClass}
          value={filters.category}
          onChange={(event) => onChange({ ...filters, category: event.target.value })}
        >
          <option value="">All categories</option>
          {CATEGORIES.map((category) => (
            <option key={category} value={category}>
              {category}
            </option>
          ))}
        </select>
      </label>
      <label className="grid gap-1 text-sm font-semibold">
        Month
        <input
          className={fieldClass}
          type="month"
          value={filters.month}
          onChange={(event) => onChange({ ...filters, month: event.target.value })}
        />
      </label>
      <label className="grid gap-1 text-sm font-semibold">
        Search notes
        <input
          className={fieldClass}
          value={filters.search}
          placeholder="lunch"
          onChange={(event) => onChange({ ...filters, search: event.target.value })}
        />
      </label>
    </div>
  )
}
