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
})
