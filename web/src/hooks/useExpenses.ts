import { useCallback, useEffect, useRef, useState } from "react"

import { apiRequest, expensesQuery, isAbort, summaryQuery } from "../api/client"
import { parseExpense, parseExpenseList, parseSummary } from "../lib/guards"
import { currentMonth } from "../lib/dates"
import type { Expense, Filters, NewExpenseInput, Summary, Trace } from "../types"

const TRACE_LIMIT = 20

export function pushTraces(current: Trace[], incoming: Trace[]): Trace[] {
  return [...incoming, ...current].slice(0, TRACE_LIMIT)
}

export function useExpenses() {
  const [filters, setFilters] = useState<Filters>(() => ({
    category: "",
    month: currentMonth(),
    search: "",
  }))
  const [expenses, setExpenses] = useState<Expense[]>([])
  const [summary, setSummary] = useState<Summary | null>(null)
  const [traces, setTraces] = useState<Trace[]>([])
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const requestId = useRef(0)
  const loadedOnce = useRef(false)

  const remember = useCallback((incoming: Trace[]) => {
    setTraces((current) => pushTraces(current, incoming))
  }, [])

  const refresh = useCallback(
    async (next: Filters, signal?: AbortSignal) => {
      const id = requestId.current + 1
      requestId.current = id
      if (loadedOnce.current) {
        setRefreshing(true)
      } else {
        setLoading(true)
      }
      try {
        const [list, sum] = await Promise.all([
          apiRequest<unknown>({ method: "GET", url: expensesQuery(next), signal }),
          apiRequest<unknown>({ method: "GET", url: summaryQuery(next.month), signal }),
        ])
        if (signal?.aborted || id !== requestId.current) {
          return
        }
        const parsedExpenses = parseExpenseList(list.data)
        const parsedSummary = parseSummary(sum.data)
        if (!parsedExpenses || !parsedSummary) {
          setError("The API returned a shape this page does not understand.")
          remember([list.trace, sum.trace])
          return
        }
        setExpenses(parsedExpenses)
        setSummary(parsedSummary)
        setError(null)
        remember([list.trace, sum.trace])
        loadedOnce.current = true
      } catch (err) {
        if (isAbort(err) || signal?.aborted || id !== requestId.current) {
          return
        }
        if (err instanceof Error && "trace" in err) {
          const traced = err as Error & { trace: Trace }
          remember([traced.trace])
          setError(traced.message)
        } else {
          setError("Could not reach the spend API. Is `spend serve` running?")
        }
      } finally {
        if (!signal?.aborted && id === requestId.current) {
          setLoading(false)
          setRefreshing(false)
        }
      }
    },
    [remember],
  )

  useEffect(() => {
    const controller = new AbortController()
    void refresh(filters, controller.signal)
    return () => controller.abort()
  }, [filters, refresh])

  const addExpense = useCallback(
    async (input: NewExpenseInput) => {
      setSaving(true)
      setError(null)
      try {
        const note = input.note.trim()
        const result = await apiRequest<unknown>({
          method: "POST",
          url: "/api/expenses",
          body: {
            amount: input.amount.trim(),
            category: input.category,
            note: note.length > 0 ? note : undefined,
            date: input.date,
          },
        })
        if (!parseExpense(result.data)) {
          remember([result.trace])
          setError("Saved, but the response was not an expense.")
          return false
        }
        remember([result.trace])
        await refresh(filters)
        return true
      } catch (err) {
        if (err instanceof Error && "trace" in err) {
          const traced = err as Error & { trace: Trace }
          remember([traced.trace])
          setError(traced.message)
        } else {
          setError("Could not save that expense.")
        }
        return false
      } finally {
        setSaving(false)
      }
    },
    [filters, refresh, remember],
  )

  const removeExpense = useCallback(
    async (id: number) => {
      setSaving(true)
      setError(null)
      try {
        const result = await apiRequest<unknown>({ method: "DELETE", url: `/api/expenses/${id}` })
        remember([result.trace])
        await refresh(filters)
        return true
      } catch (err) {
        if (err instanceof Error && "trace" in err) {
          const traced = err as Error & { trace: Trace }
          remember([traced.trace])
          setError(traced.message)
        } else {
          setError("Could not delete that expense.")
        }
        return false
      } finally {
        setSaving(false)
      }
    },
    [filters, refresh, remember],
  )

  const setBudget = useCallback(
    async (category: string, amount: string) => {
      setSaving(true)
      setError(null)
      try {
        const month = filters.month ? `?month=${encodeURIComponent(filters.month)}` : ""
        const result = await apiRequest<unknown>({
          method: "PUT",
          url: `/api/budgets/${encodeURIComponent(category)}${month}`,
          body: { amount: amount.trim() },
        })
        remember([result.trace])
        await refresh(filters)
        return true
      } catch (err) {
        if (err instanceof Error && "trace" in err) {
          const traced = err as Error & { trace: Trace }
          remember([traced.trace])
          setError(traced.message)
        } else {
          setError("Could not save that budget.")
        }
        return false
      } finally {
        setSaving(false)
      }
    },
    [filters, refresh, remember],
  )

  return {
    filters,
    setFilters,
    expenses,
    summary,
    traces,
    loading,
    refreshing,
    saving,
    error,
    refresh: () => refresh(filters),
    addExpense,
    removeExpense,
    setBudget,
  }
}
