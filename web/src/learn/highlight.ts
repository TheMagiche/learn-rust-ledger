import { createHighlighter, type Highlighter, type ThemedToken } from "shiki"

let highlighterPromise: Promise<Highlighter> | null = null

function highlighter(): Promise<Highlighter> {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      themes: ["github-light", "github-dark"],
      langs: ["rust"],
    })
  }
  return highlighterPromise
}

export async function highlightRust(code: string, theme: "light" | "dark"): Promise<ThemedToken[][]> {
  const instance = await highlighter()
  const result = instance.codeToTokens(code, {
    lang: "rust",
    theme: theme === "dark" ? "github-dark" : "github-light",
  })
  return result.tokens
}

export type { ThemedToken }
