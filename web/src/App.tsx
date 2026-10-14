import { useEffect, useState } from "react"

import { CompilerErrors } from "./pages/CompilerErrors"
import { Glossary } from "./pages/Glossary"
import { Home } from "./pages/Home"
import { Lessons } from "./pages/Lessons"
import { ThemeProvider, useTheme } from "./theme"

type Page = "app" | "lessons" | "glossary" | "errors"

const NAV: Array<{ id: Page; href: string; label: string }> = [
  { id: "app", href: "#/", label: "App" },
  { id: "lessons", href: "#/lessons", label: "Lessons" },
  { id: "glossary", href: "#/glossary", label: "Glossary" },
  { id: "errors", href: "#/errors", label: "Compiler Errors" },
]

function pageFromHash(): Page {
  const hash = window.location.hash.replace(/^#\/?/, "")
  if (hash.startsWith("lessons")) {
    return "lessons"
  }
  if (hash.startsWith("glossary")) {
    return "glossary"
  }
  if (hash.startsWith("errors")) {
    return "errors"
  }
  return "app"
}

export function App() {
  return (
    <ThemeProvider>
      <Shell />
    </ThemeProvider>
  )
}

function Shell() {
  const [page, setPage] = useState<Page>(pageFromHash)
  const { theme, toggle } = useTheme()

  useEffect(() => {
    const onHash = () => setPage(pageFromHash())
    window.addEventListener("hashchange", onHash)
    return () => window.removeEventListener("hashchange", onHash)
  }, [])

  return (
    <div className="min-h-screen">
      <header className="border-b border-line bg-card/80 backdrop-blur dark:border-night-line dark:bg-night-card/80">
        <div className="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-3 px-4 py-3">
          <a href="#/" className="font-serif text-3xl tracking-tight">
            spend
          </a>
          <nav className="flex flex-wrap items-center gap-1 text-sm font-semibold" aria-label="Primary">
            {NAV.map((item) => (
              <a
                key={item.id}
                href={item.href}
                aria-current={page === item.id ? "page" : undefined}
                className={`rounded-md px-3 py-1.5 ${
                  page === item.id ? "bg-ink text-paper dark:bg-night-ink dark:text-night" : "text-muted hover:text-ink dark:text-night-muted dark:hover:text-night-ink"
                }`}
              >
                {item.label}
              </a>
            ))}
            <button
              type="button"
              onClick={toggle}
              aria-pressed={theme === "dark"}
              className="rounded-md border border-line px-3 py-1.5 dark:border-night-line"
            >
              {theme === "dark" ? "Light" : "Dark"}
            </button>
          </nav>
        </div>
      </header>
      {import.meta.env.VITE_PAGES === "true" ? (
        <p className="border-b border-line bg-copper-soft px-4 py-3 text-sm text-ink dark:border-night-line dark:bg-night-soft dark:text-night-ink">
          <span className="mx-auto block max-w-6xl">
            This is the static UI. Lessons, the glossary, and compiler errors run in the browser. Recording expenses
            needs the Rust API from <code className="font-mono">make dev</code>.
          </span>
        </p>
      ) : null}
      <main className="mx-auto max-w-6xl px-4 py-6">
        {page === "app" ? <Home /> : null}
        {page === "lessons" ? <Lessons /> : null}
        {page === "glossary" ? <Glossary /> : null}
        {page === "errors" ? <CompilerErrors /> : null}
      </main>
    </div>
  )
}
