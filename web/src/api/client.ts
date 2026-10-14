import { errorMessage } from "../lib/guards"
import type { Trace } from "../types"

export class ApiRequestError extends Error {
  trace: Trace

  constructor(message: string, trace: Trace) {
    super(message)
    this.name = "ApiRequestError"
    this.trace = trace
  }
}

export function expensesQuery(filters: { category: string; month: string; search: string }): string {
  const params = new URLSearchParams()
  if (filters.category) {
    params.set("category", filters.category)
  }
  if (filters.month) {
    params.set("month", filters.month)
  }
  if (filters.search.trim()) {
    params.set("search", filters.search.trim())
  }
  const query = params.toString()
  return query ? `/api/expenses?${query}` : "/api/expenses"
}

export function summaryQuery(month: string): string {
  return month ? `/api/summary?month=${encodeURIComponent(month)}` : "/api/summary"
}

export async function apiRequest<T>(options: {
  method: string
  url: string
  body?: unknown
  signal?: AbortSignal
}): Promise<{ data: T; trace: Trace }> {
  const headers: Record<string, string> = {}
  let payload: string | undefined
  if (options.body !== undefined) {
    headers["content-type"] = "application/json"
    payload = JSON.stringify(options.body)
  }

  const started = performance.now()
  let response: Response
  try {
    response = await fetch(options.url, {
      method: options.method,
      headers,
      body: payload,
      signal: options.signal,
    })
  } catch (error) {
    if (isAbort(error)) {
      throw error
    }
    const trace = buildTrace(options, 0, started, null, null, null)
    const message = error instanceof Error ? error.message : "network error"
    throw new ApiRequestError(message, trace)
  }

  const raw = await response.text()
  let responseBody: unknown = null
  if (raw.length > 0) {
    try {
      responseBody = JSON.parse(raw) as unknown
    } catch {
      responseBody = raw
    }
  }

  const trace = buildTrace(
    options,
    response.status,
    started,
    responseBody,
    response.headers.get("x-rust-handler"),
    response.headers.get("x-rust-concepts"),
  )

  if (!response.ok) {
    throw new ApiRequestError(errorMessage(responseBody, `request failed (${response.status})`), trace)
  }

  return { data: responseBody as T, trace }
}

function buildTrace(
  options: { method: string; url: string; body?: unknown },
  status: number,
  started: number,
  responseBody: unknown,
  handler: string | null,
  conceptsHeader: string | null,
): Trace {
  return {
    method: options.method,
    url: options.url,
    requestBody: options.body,
    status,
    durationMs: Math.max(0, Math.round(performance.now() - started)),
    responseBody,
    handler,
    concepts: conceptsHeader
      ? conceptsHeader
          .split(",")
          .map((id) => id.trim())
          .filter((id) => id.length > 0)
      : [],
  }
}

export function isAbort(error: unknown): boolean {
  return error instanceof DOMException && error.name === "AbortError"
}
