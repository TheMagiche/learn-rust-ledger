import { CATEGORIES, type BudgetStatus, type CategoryName, type Expense, type Summary } from "../types"

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}

export function isCategory(value: unknown): value is CategoryName {
  return typeof value === "string" && CATEGORIES.some((category) => category === value)
}

function isMoneyView(value: unknown): value is { amount: number; display: string } {
  return isRecord(value) && typeof value.amount === "number" && typeof value.display === "string"
}

export function parseExpense(value: unknown): Expense | null {
  if (!isRecord(value) || !isCategory(value.category)) {
    return null
  }
  if (typeof value.id !== "number" || typeof value.amount !== "number" || typeof value.display !== "string") {
    return null
  }
  if (typeof value.date !== "string") {
    return null
  }
  const note = value.note === undefined || value.note === null ? null : value.note
  if (note !== null && typeof note !== "string") {
    return null
  }
  return {
    id: value.id,
    amount: value.amount,
    display: value.display,
    category: value.category,
    note,
    date: value.date,
  }
}

export function parseExpenseList(value: unknown): Expense[] | null {
  if (!Array.isArray(value)) {
    return null
  }
  const expenses: Expense[] = []
  for (const item of value) {
    const expense = parseExpense(item)
    if (!expense) {
      return null
    }
    expenses.push(expense)
  }
  return expenses
}

export function parseSummary(value: unknown): Summary | null {
  if (!isRecord(value) || !isMoneyView(value.total) || !Array.isArray(value.byCategory) || !Array.isArray(value.budgets)) {
    return null
  }
  const byCategory = []
  for (const item of value.byCategory) {
    if (!isRecord(item) || !isCategory(item.category) || typeof item.amount !== "number" || typeof item.display !== "string") {
      return null
    }
    byCategory.push({ category: item.category, amount: item.amount, display: item.display })
  }
  const budgets: BudgetStatus[] = []
  for (const item of value.budgets) {
    if (!isRecord(item) || !isCategory(item.category) || typeof item.over !== "boolean") {
      return null
    }
    if (!isMoneyView(item.budget) || !isMoneyView(item.spent)) {
      return null
    }
    budgets.push({
      category: item.category,
      budget: item.budget,
      spent: item.spent,
      over: item.over,
    })
  }
  return { total: value.total, byCategory, budgets }
}

export function errorMessage(value: unknown, fallback: string): string {
  if (isRecord(value) && typeof value.error === "string") {
    return value.error
  }
  return fallback
}
