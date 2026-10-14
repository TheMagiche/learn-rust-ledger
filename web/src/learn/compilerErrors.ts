import raw from "./compilerErrors.json"

export type CompilerErrorExample = {
  id: string
  code: string
  title: string
  concepts: string[]
  broken: string
  rustc: string
  explanation: string
  fix: string
}

export const compilerErrors: CompilerErrorExample[] = raw

export function compilerErrorByConcepts(conceptIds: string[]): CompilerErrorExample | undefined {
  const borrowIds = new Set(["ownership", "move", "borrowing", "mut-borrow"])
  const wanted = conceptIds.filter((id) => borrowIds.has(id))
  if (wanted.length === 0) {
    return undefined
  }
  return compilerErrors.find((example) => example.concepts.some((id) => wanted.includes(id)))
}
