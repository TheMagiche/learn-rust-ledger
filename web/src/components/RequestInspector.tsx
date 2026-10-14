import { useState } from "react"

import { toCurl, toSpendCommand } from "../lib/commands"
import type { Trace } from "../types"

type RequestInspectorProps = {
  trace: Trace
}

export function RequestInspector({ trace }: RequestInspectorProps) {
  const curl = toCurl(trace)
  const command = toSpendCommand(trace)
  const statusColor = trace.status >= 500 ? "text-danger" : trace.status >= 400 ? "text-copper" : "text-pine"

  return (
    <div className="grid gap-3 text-sm">
      <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        <span className="rounded bg-ink px-2 py-0.5 font-mono text-xs text-paper dark:bg-night-ink dark:text-night">{trace.method}</span>
        <span className="font-mono break-all">{trace.url}</span>
        <span className={`font-semibold ${statusColor}`}>{trace.status}</span>
        <span className="text-muted dark:text-night-muted">{trace.durationMs} ms</span>
      </div>
      <BodyBlock label="Request body" value={trace.requestBody} />
      <BodyBlock label="Response body" value={trace.responseBody} />
      <CopyBlock label="curl" text={curl} />
      <CopyBlock label="spend" text={command} />
    </div>
  )
}

function BodyBlock({ label, value }: { label: string; value: unknown }) {
  const text = value === undefined || value === null ? "—" : JSON.stringify(value, null, 2)
  return (
    <div>
      <p className="text-xs font-semibold uppercase tracking-wide text-muted dark:text-night-muted">{label}</p>
      <pre className="mt-1 overflow-x-auto rounded-md bg-paper px-3 py-2 font-mono text-xs dark:bg-night">{text}</pre>
    </div>
  )
}

function CopyBlock({ label, text }: { label: string; text: string }) {
  const [copied, setCopied] = useState(false)

  async function copy() {
    try {
      await navigator.clipboard.writeText(text)
      setCopied(true)
    } catch {
      setCopied(false)
    }
  }

  return (
    <div>
      <div className="flex items-center justify-between gap-2">
        <p className="text-xs font-semibold uppercase tracking-wide text-muted dark:text-night-muted">{label}</p>
        <button type="button" className="text-xs font-semibold text-copper" onClick={() => void copy()}>
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="mt-1 overflow-x-auto rounded-md bg-paper px-3 py-2 font-mono text-xs dark:bg-night">{text}</pre>
    </div>
  )
}
