import { afterEach, describe, expect, it, vi } from "vitest"

import { apiRequest } from "./client"

afterEach(() => {
  vi.unstubAllGlobals()
})

describe("apiRequest traces", () => {
  it("reads teaching headers, status, and JSON", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(
        async () =>
          new Response(JSON.stringify({ ok: true }), {
            status: 200,
            headers: {
              "content-type": "application/json",
              "x-rust-handler": "health",
              "x-rust-concepts": "async-await, tokio",
            },
          }),
      ),
    )

    const result = await apiRequest<{ ok: boolean }>({ method: "GET", url: "/api/health" })
    expect(result.data).toEqual({ ok: true })
    expect(result.trace.handler).toBe("health")
    expect(result.trace.concepts).toEqual(["async-await", "tokio"])
    expect(result.trace.status).toBe(200)
    expect(result.trace.method).toBe("GET")
    expect(result.trace.url).toBe("/api/health")
    expect(result.trace.durationMs).toBeGreaterThanOrEqual(0)
  })

  it("throws the JSON error and keeps the trace", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(
        async () =>
          new Response(JSON.stringify({ error: "invalid amount: abc" }), {
            status: 400,
            headers: {
              "x-rust-handler": "create_expense",
              "x-rust-concepts": "result,question-mark",
            },
          }),
      ),
    )

    await expect(
      apiRequest({
        method: "POST",
        url: "/api/expenses",
        body: { amount: "abc", category: "food" },
      }),
    ).rejects.toMatchObject({
      message: "invalid amount: abc",
      trace: {
        status: 400,
        handler: "create_expense",
        concepts: ["result", "question-mark"],
        requestBody: { amount: "abc", category: "food" },
      },
    })
  })

  it("uses a null body for 204", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(null, { status: 204, headers: { "x-rust-handler": "delete_expense" } })),
    )
    const result = await apiRequest({ method: "DELETE", url: "/api/expenses/1" })
    expect(result.data).toBeNull()
    expect(result.trace.responseBody).toBeNull()
    expect(result.trace.handler).toBe("delete_expense")
  })

  it("handles wasm request returning plain objects for expenses and summary", async () => {
    const fs = await import("node:fs")
    const path = await import("node:path")
    const wasmModule = await import("../wasm/spend_wasm")
    const wasmPath = path.resolve(__dirname, "../wasm/spend_wasm_bg.wasm")
    const wasmBytes = fs.readFileSync(wasmPath)
    await wasmModule.default(wasmBytes)

    const { wasmRequest, resetWasmLedger } = await import("./wasm")
    const { parseExpense, parseExpenseList, parseSummary } = await import("../lib/guards")

    await resetWasmLedger()
    const createRes = await wasmRequest({
      method: "POST",
      url: "/api/expenses",
      body: { amount: "12.50", category: "food", note: "lunch", date: "2026-10-14" },
    })
    expect(createRes.status).toBe(201)
    const parsedExpense = parseExpense(createRes.data)
    expect(parsedExpense).not.toBeNull()
    expect(parsedExpense?.amount).toBe(1250)
    expect(parsedExpense?.category).toBe("food")

    const listRes = await wasmRequest({
      method: "GET",
      url: "/api/expenses",
    })
    expect(listRes.status).toBe(200)
    const parsedList = parseExpenseList(listRes.data)
    expect(parsedList).toHaveLength(1)

    const budgetRes = await wasmRequest({
      method: "PUT",
      url: "/api/budgets/food?month=2026-10",
      body: { amount: "150.00" },
    })
    expect(budgetRes.status).toBe(200)

    const summaryRes = await wasmRequest({
      method: "GET",
      url: "/api/summary?month=2026-10",
    })
    expect(summaryRes.status).toBe(200)
    const parsedSummary = parseSummary(summaryRes.data)
    expect(parsedSummary).not.toBeNull()
    expect(parsedSummary?.total.display).toBe("12.50")
    expect(parsedSummary?.budgets[0]?.category).toBe("food")
    expect(parsedSummary?.budgets[0]?.budget.display).toBe("150.00")
  })
})
