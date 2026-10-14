export const CATEGORIES = [
  "food",
  "transport",
  "rent",
  "utilities",
  "fun",
  "health",
  "other",
] as const

export type CategoryName = (typeof CATEGORIES)[number]

export type MoneyView = {
  amount: number
  display: string
}

export type Expense = {
  id: number
  amount: number
  display: string
  category: CategoryName
  note: string | null
  date: string
}

export type CategoryTotal = {
  category: CategoryName
  amount: number
  display: string
}

export type BudgetStatus = {
  category: CategoryName
  budget: MoneyView
  spent: MoneyView
  over: boolean
}

export type Summary = {
  total: MoneyView
  byCategory: CategoryTotal[]
  budgets: BudgetStatus[]
}

export type Filters = {
  category: string
  month: string
  search: string
}

export type NewExpenseInput = {
  amount: string
  category: string
  note: string
  date: string
}

export type Trace = {
  method: string
  url: string
  requestBody?: unknown
  status: number
  durationMs: number
  responseBody: unknown
  handler: string | null
  concepts: string[]
}
