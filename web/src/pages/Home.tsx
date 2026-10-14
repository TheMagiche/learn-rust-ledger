import { BudgetBar } from "../components/BudgetBar"
import { CategoryChart } from "../components/CategoryChart"
import { ExpenseForm } from "../components/ExpenseForm"
import { ExpenseTable } from "../components/ExpenseTable"
import { Filters } from "../components/Filters"
import { LearnPanel } from "../components/LearnPanel"
import { Summary } from "../components/Summary"
import { useExpenses } from "../hooks/useExpenses"

export function Home() {
  const state = useExpenses()

  return (
    <div className="grid items-start gap-6 lg:grid-cols-[minmax(0,1.2fr)_minmax(320px,0.8fr)]">
      <div className="grid min-w-0 gap-4">
        <ExpenseForm saving={state.saving} onSubmit={state.addExpense} />
        <Summary summary={state.summary} count={state.expenses.length} loading={state.loading} />
        <div className="grid min-w-0 gap-4 xl:grid-cols-2">
          <CategoryChart rows={state.summary?.byCategory ?? []} />
          <BudgetBar budgets={state.summary?.budgets ?? []} saving={state.saving} onSetBudget={state.setBudget} />
        </div>
        <Filters filters={state.filters} onChange={state.setFilters} />
        <ExpenseTable
          expenses={state.expenses}
          loading={state.loading}
          refreshing={state.refreshing}
          error={state.error}
          onRetry={() => void state.refresh()}
          onDelete={state.removeExpense}
        />
        <p className="text-sm lg:hidden">
          <a className="font-semibold text-copper" href="#learn">
            See the Rust for the latest request
          </a>
        </p>
      </div>
      <LearnPanel traces={state.traces} />
    </div>
  )
}
