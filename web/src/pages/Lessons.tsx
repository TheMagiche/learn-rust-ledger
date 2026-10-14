import { useEffect, useState } from "react"

import { LESSON_STORAGE_KEY, lessons } from "../learn/lessons"

function readProgress(): Record<string, boolean> {
  try {
    const raw = localStorage.getItem(LESSON_STORAGE_KEY)
    if (!raw) {
      return {}
    }
    const parsed: unknown = JSON.parse(raw)
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
      return {}
    }
    const progress: Record<string, boolean> = {}
    for (const [key, value] of Object.entries(parsed)) {
      if (value === true) {
        progress[key] = true
      }
    }
    return progress
  } catch {
    return {}
  }
}

export function Lessons() {
  const [done, setDone] = useState<Record<string, boolean>>(readProgress)
  const finished = lessons.filter((lesson) => done[lesson.id]).length

  useEffect(() => {
    localStorage.setItem(LESSON_STORAGE_KEY, JSON.stringify(done))
  }, [done])

  return (
    <div className="mx-auto grid max-w-3xl gap-4">
      <header>
        <h2 className="font-serif text-4xl">Ten passes through the ledger</h2>
        <p className="mt-2 text-muted dark:text-night-muted">
          {finished} of {lessons.length} marked done. Progress stays in this browser.
        </p>
      </header>
      <ol className="grid gap-4">
        {lessons.map((lesson, index) => {
          const complete = done[lesson.id] === true
          return (
            <li key={lesson.id} className="rounded-xl border border-line bg-card p-4 dark:border-night-line dark:bg-night-card">
              <div className="flex items-start justify-between gap-3">
                <h3 className="font-serif text-2xl">
                  {index + 1}. {lesson.title}
                </h3>
                <label className="flex items-center gap-2 text-sm font-semibold">
                  <input
                    type="checkbox"
                    checked={complete}
                    onChange={() => setDone((current) => ({ ...current, [lesson.id]: !complete }))}
                  />
                  Done
                </label>
              </div>
              <p className="mt-2">{lesson.goal}</p>
              <p className="mt-3 text-sm">
                <span className="font-semibold">Click: </span>
                {lesson.click}
              </p>
              <p className="mt-2 text-sm">
                <span className="font-semibold">Open: </span>
                <span className="font-mono">{lesson.file}</span>
              </p>
              <p className="mt-2 text-sm">
                <span className="font-semibold">Exercise: </span>
                {lesson.exercise}
              </p>
              <details className="mt-2 text-sm text-muted dark:text-night-muted">
                <summary className="cursor-pointer font-semibold text-copper">Hint</summary>
                <p className="mt-1">{lesson.hint}</p>
              </details>
            </li>
          )
        })}
      </ol>
    </div>
  )
}
