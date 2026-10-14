import { isRecord } from "./guards"
import type { Trace } from "../types"

export function toCurl(trace: Trace): string {
  const absolute = new URL(trace.url, "http://127.0.0.1:8080").toString()
  const lines = [`curl -sS -X ${trace.method} ${shellQuote(absolute)}`]
  if (trace.requestBody !== undefined) {
    lines.push("-H 'content-type: application/json'")
    lines.push(`-d ${shellQuote(JSON.stringify(trace.requestBody))}`)
  }
  return lines.join(" \\\n  ")
}

export function toSpendCommand(trace: Trace): string {
  const url = new URL(trace.url, "http://127.0.0.1:8080")
  const path = url.pathname
  const body = isRecord(trace.requestBody) ? trace.requestBody : null

  if (trace.method === "POST" && path === "/api/expenses" && body) {
    const amount = String(body.amount ?? "")
    const category = String(body.category ?? "")
    const note = typeof body.note === "string" && body.note.length > 0 ? ` --note ${shellQuote(body.note)}` : ""
    const date = typeof body.date === "string" && body.date.length > 0 ? ` --date ${body.date}` : ""
    return `spend add ${amount} ${category}${note}${date}`
  }

  if (trace.method === "GET" && path === "/api/expenses") {
    const flags = [
      flag("category", url.searchParams.get("category")),
      flag("month", url.searchParams.get("month")),
      flag("search", url.searchParams.get("search")),
    ].filter(Boolean)
    return ["spend list", ...flags].join(" ")
  }

  if (trace.method === "DELETE" && path.startsWith("/api/expenses/")) {
    return `spend remove ${path.slice("/api/expenses/".length)}`
  }

  if (trace.method === "GET" && path === "/api/summary") {
    const month = url.searchParams.get("month")
    return month ? `spend report --month ${month}` : "spend report"
  }

  if (trace.method === "PUT" && path.startsWith("/api/budgets/") && body) {
    const category = decodeURIComponent(path.slice("/api/budgets/".length))
    return `spend budget set ${category} ${String(body.amount ?? "")}`
  }

  if (trace.method === "GET" && path === "/api/health") {
    return "# health is API-only — there is no spend health subcommand"
  }

  if (trace.method === "GET" && path.startsWith("/api/expenses/")) {
    return `spend list  # then look for id ${path.slice("/api/expenses/".length)}`
  }

  return "spend --help"
}

function flag(name: string, value: string | null): string {
  if (!value) {
    return ""
  }
  return `--${name} ${shellQuote(value)}`
}

export function shellQuote(value: string): string {
  if (/^[A-Za-z0-9_.:/@=-]+$/.test(value)) {
    return value
  }
  return `'${value.replaceAll("'", "'\\''")}'`
}
