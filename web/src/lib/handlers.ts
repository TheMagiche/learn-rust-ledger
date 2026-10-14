export const HANDLER_CALLS: Record<string, string[]> = {
  list_expenses: ["Ledger::list"],
  create_expense: ["Ledger::add"],
  get_expense: ["Ledger::get"],
  delete_expense: ["Ledger::remove"],
  summary: ["Ledger::total", "Ledger::by_category", "Ledger::budget_status"],
  set_budget: ["Ledger::set_budget"],
  health: [],
}

export const CATEGORY_COLORS: Record<string, string> = {
  food: "#c4562a",
  transport: "#2f6f8f",
  rent: "#1d6a52",
  utilities: "#c4a15a",
  fun: "#a24b73",
  health: "#6b8f71",
  other: "#8a8178",
}
