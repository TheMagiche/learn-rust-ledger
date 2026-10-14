import { act, renderHook, waitFor } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"

import type { Expense, Summary } from "../types"
import { pushTraces, useExpenses } from "./useExpenses"
import type { Trace } from "../types"

const expense: Expense = {
  id: 1,
  amount: 1250,
  display: "12.50",
  category: "food",
  note: "lunch",
  date: "2026-10-14",
}

const summary: Summary = {
  total: { amount: 1250, display: "12.50" },
  byCategory: [{ category: "food", amount: 1250, display: "12.50" }],
  budgets: [],
}

function jsonResponse(body: unknown, status: number, handler: string, concepts: string) {
  return new Response(status === 204 ? null : JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json",
      "x-rust-handler": handler,
      "x-rust-concepts": concepts,
    },
  })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe("useExpenses", () => {
  it("loads expenses, records handler traces, and posts a new expense", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input)
      const method = init?.method ?? "GET"
      if (method === "POST") {
        return jsonResponse(expense, 201, "create_expense", "ownership,result")
      }
      if (url.startsWith("/api/summary")) {
        return jsonResponse(summary, 200, "summary", "iterators,btreemap,hashmap")
      }
      return jsonResponse([expense], 200, "list_expenses", "borrowing,iterators,closures,serde")
    })
    vi.stubGlobal("fetch", fetchMock)

    const { result } = renderHook(() => useExpenses())
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.expenses).toEqual([expense])
    expect(result.current.summary?.total.display).toBe("12.50")
    expect(result.current.traces.some((trace) => trace.handler === "list_expenses")).toBe(true)
    expect(result.current.error).toBeNull()

    let saved = false
    await act(async () => {
      saved = await result.current.addExpense({
        amount: "12.50",
        category: "food",
        note: "lunch",
        date: "2026-10-14",
      })
    })
    expect(saved).toBe(true)
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/expenses",
      expect.objectContaining({ method: "POST" }),
    )
    expect(result.current.traces.some((trace) => trace.handler === "create_expense")).toBe(true)
  })

  it("surfaces a 400 from the API", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input)
        if ((init?.method ?? "GET") === "POST") {
          return jsonResponse({ error: "invalid amount: abc" }, 400, "create_expense", "result")
        }
        if (url.startsWith("/api/summary")) {
          return jsonResponse(summary, 200, "summary", "iterators")
        }
        return jsonResponse([], 200, "list_expenses", "borrowing")
      }),
    )

    const { result } = renderHook(() => useExpenses())
    await waitFor(() => expect(result.current.loading).toBe(false))
    let saved = false
    await act(async () => {
      saved = await result.current.addExpense({
        amount: "abc",
        category: "food",
        note: "",
        date: "2026-10-14",
      })
    })
    expect(saved).toBe(false)
    expect(result.current.error).toBe("invalid amount: abc")
    expect(result.current.traces[0]?.status).toBe(400)
  })

  it("keeps only the latest 20 traces", () => {
    const traces: Trace[] = Array.from({ length: 20 }, (_, index) => ({
      method: "GET",
      url: `/api/expenses?n=${index}`,
      status: 200,
      durationMs: 1,
      responseBody: [],
      handler: "list_expenses",
      concepts: ["borrowing"],
    }))
    const next = pushTraces(traces, [
      {
        method: "POST",
        url: "/api/expenses",
        status: 201,
        durationMs: 2,
        responseBody: expense,
        handler: "create_expense",
        concepts: ["ownership"],
      },
    ])
    expect(next).toHaveLength(20)
    expect(next[0]?.handler).toBe("create_expense")
    expect(next.some((trace) => trace.url === "/api/expenses?n=19")).toBe(false)
  })
})
