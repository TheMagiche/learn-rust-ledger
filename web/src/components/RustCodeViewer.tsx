import { useEffect, useState, type ReactNode } from "react"

import { highlightRust, type ThemedToken } from "../learn/highlight"
import { rustSources } from "../learn/rustSources"
import { HANDLER_CALLS } from "../lib/handlers"
import { useTheme } from "../theme"

type RustCodeViewerProps = {
  handler: string | null
  onConcept: (id: string) => void
}

const LEARN_PATTERN = /LEARN\(([a-z0-9-]+)\)/

export function RustCodeViewer({ handler, onConcept }: RustCodeViewerProps) {
  const names = handler ? [handler, ...(HANDLER_CALLS[handler] ?? [])] : []
  const blocks = names.flatMap((name) => {
    const source = rustSources[name]
    return source ? [{ name, ...source }] : []
  })

  if (!handler) {
    return <p className="text-sm text-muted dark:text-night-muted">Make a request to see the handler that served it.</p>
  }

  if (blocks.length === 0) {
    return (
      <p className="text-sm text-muted dark:text-night-muted">
        No extracted source for <span className="font-mono">{handler}</span>. Run <span className="font-mono">make gen</span>.
      </p>
    )
  }

  return (
    <div className="grid gap-4">
      {blocks.map((block) => (
        <SourceBlock key={block.name} name={block.name} file={block.file} startLine={block.startLine} code={block.code} onConcept={onConcept} />
      ))}
    </div>
  )
}

function SourceBlock({
  name,
  file,
  startLine,
  code,
  onConcept,
}: {
  name: string
  file: string
  startLine: number
  code: string
  onConcept: (id: string) => void
}) {
  const { theme } = useTheme()
  const [tokens, setTokens] = useState<ThemedToken[][] | null>(null)
  const [failed, setFailed] = useState(false)
  const lines = code.split("\n")

  useEffect(() => {
    let cancelled = false
    setFailed(false)
    void highlightRust(code, theme)
      .then((next) => {
        if (!cancelled) {
          setTokens(next)
        }
      })
      .catch(() => {
        if (!cancelled) {
          setFailed(true)
        }
      })
    return () => {
      cancelled = true
    }
  }, [code, theme])

  return (
    <div>
      <p className="font-mono text-xs text-muted dark:text-night-muted">
        {name} · {file}:{startLine}
      </p>
      <div className="mt-1 overflow-x-auto rounded-md bg-[#f6f8fa] text-xs leading-5 dark:bg-[#0d1117]">
        <pre className="min-w-max p-3 font-mono">
          {(tokens ?? lines.map((line) => [{ content: line, color: undefined, offset: 0 }])).map((line, index) => {
            const text = lines[index] ?? ""
            const learn = LEARN_PATTERN.exec(text)
            const lineNumber = startLine + index
            const content = (
              <code>
                {line.map((token, tokenIndex) => (
                  <span key={tokenIndex} style={{ color: token.color }}>
                    {token.content}
                  </span>
                ))}
              </code>
            )
            return learn ? (
              <button
                key={lineNumber}
                type="button"
                className="block w-full bg-copper/15 text-left hover:bg-copper/25"
                onClick={() => onConcept(learn[1] ?? "")}
              >
                <LineBody number={lineNumber}>{content}</LineBody>
              </button>
            ) : (
              <div key={lineNumber}>
                <LineBody number={lineNumber}>{content}</LineBody>
              </div>
            )
          })}
        </pre>
        {failed ? <p className="px-3 pb-2 text-xs text-danger">Highlighter failed. Showing plain source.</p> : null}
      </div>
    </div>
  )
}

function LineBody({ number, children }: { number: number; children: ReactNode }) {
  return (
    <span className="grid grid-cols-[3rem_1fr] gap-3">
      <span className="text-right text-muted/80 select-none dark:text-night-muted">{number}</span>
      <span>{children}</span>
    </span>
  )
}
