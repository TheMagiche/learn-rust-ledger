import { existsSync, readdirSync, readFileSync, statSync } from "node:fs"
import { join } from "node:path"
import { describe, expect, it } from "vitest"

import { concepts } from "./concepts"

const REQUIRED = [
  "cargo",
  "crates-workspace",
  "variables-mutability",
  "shadowing",
  "ownership",
  "move",
  "borrowing",
  "mut-borrow",
  "slices",
  "strings",
  "structs",
  "enums",
  "match",
  "option",
  "result",
  "question-mark",
  "custom-errors",
  "from-into",
  "traits",
  "derive",
  "display",
  "fromstr",
  "newtype",
  "impl-blocks",
  "iterators",
  "closures",
  "vec",
  "hashmap",
  "btreemap",
  "generics",
  "trait-objects",
  "arc",
  "mutex",
  "async-await",
  "tokio",
  "serde",
  "modules-visibility",
  "testing",
  "doc-tests",
  "integer-money",
]

function cratesDir(): string {
  const candidates = [join(process.cwd(), "crates"), join(process.cwd(), "../crates")]
  const found = candidates.find((candidate) => existsSync(candidate))
  if (!found) {
    throw new Error(`cannot find crates from ${process.cwd()}`)
  }
  return found
}

function readRust(dir: string): string {
  const chunks: string[] = []
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry)
    if (entry === "target") {
      continue
    }
    if (statSync(path).isDirectory()) {
      chunks.push(readRust(path))
    } else if (entry.endsWith(".rs")) {
      chunks.push(readFileSync(path, "utf8"))
    }
  }
  return chunks.join("\n")
}

describe("concept ids", () => {
  it("includes every required id", () => {
    const ids = new Set(concepts.map((concept) => concept.id))
    for (const id of REQUIRED) {
      expect(ids.has(id), id).toBe(true)
    }
  })

  it("covers every LEARN comment and teaching header", () => {
    const rust = readRust(cratesDir())
    const ids = new Set(concepts.map((concept) => concept.id))
    const used = new Set<string>()
    for (const match of rust.matchAll(/LEARN\(([a-z0-9-]+)\)/g)) {
      if (match[1]) {
        used.add(match[1])
      }
    }
    for (const match of rust.matchAll(/concepts:\s*"([a-z0-9,-]+)"/g)) {
      for (const id of (match[1] ?? "").split(",")) {
        if (id) {
          used.add(id)
        }
      }
    }
    expect(used.size).toBeGreaterThan(10)
    for (const id of used) {
      expect(ids.has(id), id).toBe(true)
    }
  })
})
