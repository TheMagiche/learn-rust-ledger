export type Lesson = {
  id: string
  title: string
  goal: string
  click: string
  file: string
  exercise: string
  hint: string
}

export const lessons: Lesson[] = [
  {
    id: "cargo",
    title: "Cargo and your first request",
    goal: "See a request leave the browser, hit axum, and come back with the name of the Rust function that handled it.",
    click: "Stay on App. The page loads expenses and a summary. Open the Rust panel and pick the newest trace.",
    file: "crates/cli/src/main.rs and crates/api/src/routes.rs",
    exercise: "In another terminal run `spend --help`, then `curl -sS http://127.0.0.1:8080/api/health`. Compare the JSON with the health handler.",
    hint: "`make dev` starts the API and Vite. The `--` in `cargo run -p spend -- serve --dev` separates Cargo's flags from the program's flags.",
  },
  {
    id: "structs",
    title: "Structs and enums",
    goal: "Add an expense and see a struct get built from the JSON body.",
    click: "Submit the form with a category. In the panel, open create_expense and the Category enum.",
    file: "crates/core/src/expense.rs and crates/core/src/category.rs",
    exercise: "Type a category the select does not offer by calling curl with \"spaceship\". Read the 400 body.",
    hint: "serde rename_all = \"lowercase\" is why the JSON says \"food\" and the Rust variant is Category::Food.",
  },
  {
    id: "money",
    title: "Money without floats",
    goal: "Watch a decimal string become integer cents.",
    click: "Enter 12.50 and save. Then try 1.234 and 0 in the form before you submit.",
    file: "crates/core/src/money.rs",
    exercise: "Add a row to the parse table in money.rs for \"1.2\" (120 cents) and run `cargo test -p spend-core parse_table`.",
    hint: "One decimal digit means tenths, so it is multiplied by 10 to become cents. A trailing dot is rejected.",
  },
  {
    id: "ownership",
    title: "Ownership",
    goal: "Delete a row and see the ledger give the Expense back by value.",
    click: "Delete an expense and confirm. Read delete_expense, then open Ledger::remove.",
    file: "crates/core/src/ledger.rs — remove",
    exercise: "Say out loud why remove returns Expense and not &Expense. Check the 204 and the following list request.",
    hint: "After Vec::remove, that slot no longer holds the value. A reference would dangle. The move hands the value to the caller.",
  },
  {
    id: "borrowing",
    title: "Borrowing",
    goal: "Filter the list with a shared borrow, then compare it with the mutable borrow used to add.",
    click: "Change the month, category, or search. Open list_expenses and Ledger::list. Then look at Ledger::add.",
    file: "crates/core/src/ledger.rs — list and add",
    exercise: "Explain why list can run while another reader holds a reference, and why add cannot.",
    hint: "&self is shared. &mut self is exclusive. The mutex is how the server provides that exclusive access.",
  },
  {
    id: "errors",
    title: "Errors and ?",
    goal: "Trigger a 400 and a 404 and follow the conversion from SpendError to JSON.",
    click: "Delete an id that is already gone, or submit amount abc. Open the trace and the error type.",
    file: "crates/core/src/error.rs and crates/api/src/error.rs",
    exercise: "Find the From implementation and the ? that uses it in create_expense.",
    hint: "NotFound becomes HTTP 404. The other SpendError variants become 400. The body is always {\"error\": \"...\"}.",
  },
  {
    id: "iterators",
    title: "Iterators and closures",
    goal: "See filter, map, and sum produce the month total and the category chart.",
    click: "Open a summary trace. Read Ledger::list, Ledger::total, and Ledger::by_category.",
    file: "crates/core/src/ledger.rs",
    exercise: "The search box only looks at the note and the category name. Find the closure that does that.",
    hint: "The closure is the |expense| passed to filter. total then maps each borrowed expense to its Copy Money and sums.",
  },
  {
    id: "traits",
    title: "Traits",
    goal: "See Display, FromStr, and Store as three contracts.",
    click: "Add an expense and look at how the category is parsed. Then open the store trait.",
    file: "crates/core/src/category.rs and crates/store/src/lib.rs",
    exercise: "Sketch a CsvStore that implements Store. It only needs load and save. You do not have to finish the parser.",
    hint: "MemoryStore exists so tests can use the same trait without touching disk. That is the payoff of the trait object in AppState.",
  },
  {
    id: "shared-state",
    title: "Shared state",
    goal: "Name what Arc and Mutex each do when two requests arrive.",
    click: "Save two expenses in a row. Open crates/api/src/state.rs from the concept cards.",
    file: "crates/api/src/state.rs",
    exercise: "Write one sentence for Arc and one for Mutex. Check them against the LEARN comments.",
    hint: "Arc shares the pointer. The mutex makes the ledger change exclusive. Cloning the Arc does not clone the expenses.",
  },
  {
    id: "testing",
    title: "Testing",
    goal: "Extend the money parse table and run it.",
    click: "Open the parse_table test. It already includes \"0\", \"-1\", \"1.234\", and \"abc\".",
    file: "crates/core/src/money.rs — the tests module",
    exercise: "Add \"12.\" as an error and \"0.5\" as 50 cents. Run `cargo test -p spend-core parse_table`. Then run `cargo test --doc -p spend-core` for the doc test.",
    hint: "#[cfg(test)] keeps the module out of the shipped binary. The doc test is the example on Money::parse.",
  },
]

export const LESSON_STORAGE_KEY = "spend.lessons.progress"
