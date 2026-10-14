import { CompilerErrorCard } from "../components/CompilerErrorCard"
import { compilerErrors } from "../learn/compilerErrors"

export function CompilerErrors() {
  return (
    <div className="mx-auto grid max-w-3xl gap-4">
      <header>
        <h2 className="font-serif text-4xl">Compiler errors</h2>
        <p className="mt-2 text-muted dark:text-night-muted">
          Each snippet is compiled with rustc. The message below is the one the compiler prints, and the test in{" "}
          <span className="font-mono">crates/core/tests/compiler_errors.rs</span> checks the error code.
        </p>
      </header>
      {compilerErrors.map((example) => (
        <CompilerErrorCard key={example.id} example={example} />
      ))}
    </div>
  )
}
