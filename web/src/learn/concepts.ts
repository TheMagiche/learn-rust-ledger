import { usedInByConcept } from "./usedIn"

export type Concept = {
  id: string
  title: string
  summary: string
  explanation: string
  example: string
  bookLink: string
  exampleLink: string
  usedIn: string[]
}

const definitions = [
  {
    id: "cargo",
    title: "Cargo",
    summary: "Cargo builds, tests, and runs the project.",
    explanation:
      "Cargo reads Cargo.toml, downloads crates, and produces the spend binary. `cargo run -p spend -- serve --dev` means: build the spend package and pass everything after `--` to the program.",
    example: "cargo test --workspace\ncargo run -p spend -- add 12.50 food --note lunch",
    bookLink: "https://doc.rust-lang.org/book/ch01-03-hello-cargo.html",
    exampleLink: "https://doc.rust-lang.org/cargo/guide/",
  },
  {
    id: "crates-workspace",
    title: "Crates and workspaces",
    summary: "A crate is one library or binary. A workspace builds several together.",
    explanation:
      "spend-core has no files or HTTP. spend-store knows about JSON. spend-api knows about axum. The spend binary is the only crate with a main function. Splitting them means you can test the ledger without starting a server.",
    example: "[workspace]\nmembers = [\"crates/core\", \"crates/store\", \"crates/api\", \"crates/cli\"]",
    bookLink: "https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/cargo.html",
  },
  {
    id: "variables-mutability",
    title: "Variables and mutability",
    summary: "Bindings cannot change unless you write mut.",
    explanation:
      "Rust defaults to immutable. `let id = self.next_id` cannot be assigned again. `let mut note` can, because a blank note is replaced with None. The compiler turns accidental writes into errors.",
    example: "let id = 1;\nlet mut note = Some(\"  \".to_string());\nnote = None;",
    bookLink: "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/variable_bindings/mut.html",
  },
  {
    id: "shadowing",
    title: "Shadowing",
    summary: "A new let can reuse a name and change the type.",
    explanation:
      "In the CLI, the parameter amount is text the user typed. The next line binds amount again as Money. The right-hand side still sees the text. This is shadowing, not mutation: the new name is a different value.",
    example: "let amount = Money::parse(&amount)?;",
    bookLink: "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html#shadowing",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/variable_bindings/scope.html",
  },
  {
    id: "ownership",
    title: "Ownership",
    summary: "Every value has one owner. When the owner goes away, the value is dropped.",
    explanation:
      "NewExpense is moved into Ledger::add, which builds an Expense and stores it in the vector. After that move, the caller cannot use the draft again. The ledger is the owner until remove gives the expense back.",
    example: "ledger.add(draft)?;",
    bookLink: "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/scope/move.html",
  },
  {
    id: "move",
    title: "Move",
    summary: "Assignment of a non-Copy value transfers ownership.",
    explanation:
      "Vec::remove returns the Expense by value. That is a move out of the vector. A reference would point at a slot that no longer holds the expense. String and Expense are moved; Money is Copy, so assigning it copies the cents.",
    example: "let removed = ledger.remove(id)?;",
    bookLink: "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#ways-variables-and-data-interact-move",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/scope/move.html",
  },
  {
    id: "borrowing",
    title: "Borrowing",
    summary: "A reference lets you read a value without taking ownership.",
    explanation:
      "list takes &self and returns Vec<&Expense>. The caller can look at each row and cannot free it or change the ledger through that reference. Many shared borrows can exist at once. The borrow ends when the reference goes out of scope.",
    example: "pub fn get(&self, id: u32) -> Option<&Expense>",
    bookLink: "https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/scope/borrow.html",
  },
  {
    id: "mut-borrow",
    title: "Mutable borrow",
    summary: "&mut is exclusive access to change a value.",
    explanation:
      "add and remove take &mut self because they change the vector. Only one mutable borrow can be alive. That is why the HTTP handlers lock the mutex before calling them: two requests cannot both hold &mut Ledger.",
    example: "pub fn add(&mut self, draft: NewExpense) -> Result<&Expense, SpendError>",
    bookLink: "https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#mutable-references",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/scope/borrow/mut.html",
  },
  {
    id: "slices",
    title: "Slices",
    summary: "A slice is a view of a contiguous sequence, like part of a string or array.",
    explanation:
      "Date checks look at text.as_bytes(), which is a &[u8]. bytes[4] is the first dash in YYYY-MM-DD, and bytes[..4] is the year, without copying the string. A &str is itself a slice of UTF-8 bytes.",
    example: "let bytes = text.as_bytes();\nbytes.len() == 10 && bytes[4] == b'-'",
    bookLink: "https://doc.rust-lang.org/book/ch04-03-slices.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/primitives/array.html",
  },
  {
    id: "strings",
    title: "Strings",
    summary: "&str borrows text. String owns it.",
    explanation:
      "trim() returns a &str that points into the existing text. to_string() allocates a String the ledger can keep after the request body is dropped. The JSON file stores that owned String.",
    example: "note = Some(value.trim().to_string());",
    bookLink: "https://doc.rust-lang.org/book/ch08-02-strings.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/std/str.html",
  },
  {
    id: "structs",
    title: "Structs",
    summary: "A struct groups named fields into one type.",
    explanation:
      "Expense is a struct: id, amount, category, note, and date. NewExpense is the same idea before the ledger assigns an id. Field names are part of the type, so a date cannot silently land in the note.",
    example: "pub struct Expense {\n    pub id: u32,\n    pub amount: Money,\n    pub category: Category,\n}",
    bookLink: "https://doc.rust-lang.org/book/ch05-01-defining-structs.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/custom_types/structs.html",
  },
  {
    id: "enums",
    title: "Enums",
    summary: "An enum is a closed set of variants.",
    explanation:
      "Category is Food, Transport, Rent, Utilities, Fun, Health, or Other. A string could be misspelled. An enum cannot. serde writes the variant as a lowercase word such as \"food\".",
    example: "pub enum Category { Food, Transport, Rent, Utilities, Fun, Health, Other }",
    bookLink: "https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/custom_types/enum.html",
  },
  {
    id: "match",
    title: "match",
    summary: "match compares a value against patterns and must be exhaustive.",
    explanation:
      "FromStr for Category matches the lowercase word and uses _ for everything else, which becomes UnknownCategory. The compiler checks that every variant is handled when you match on the enum itself.",
    example: "match text.trim().to_ascii_lowercase().as_str() {\n    \"food\" => Ok(Category::Food),\n    _ => Err(SpendError::UnknownCategory(text.to_string())),\n}",
    bookLink: "https://doc.rust-lang.org/book/ch06-02-match.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/flow_control/match.html",
  },
  {
    id: "option",
    title: "Option",
    summary: "Option is Some(value) or None. It replaces null.",
    explanation:
      "A missing note is None, not an empty string and not null. get returns Option<&Expense>: None means that id is not in the ledger. ok_or turns None into a Result error the handler can return as 404.",
    example: "let expense = ledger.get(id).ok_or(SpendError::NotFound(id))?;",
    bookLink: "https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#the-option-enum-and-its-advantages-over-null-values",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/std/option.html",
  },
  {
    id: "result",
    title: "Result",
    summary: "Result is Ok(value) or Err(error) for failures you can recover from.",
    explanation:
      "Money::parse returns Result<Money, SpendError>. Bad input is Err, not a panic. main matches on the Result, prints error: to stderr, and exits 1. Library code does not call unwrap for mistakes a person can make.",
    example: "match Money::parse(\"12.50\") {\n    Ok(money) => println!(\"{money}\"),\n    Err(err) => eprintln!(\"error: {err}\"),\n}",
    bookLink: "https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/error/result.html",
  },
  {
    id: "question-mark",
    title: "The ? operator",
    summary: "? returns early when a Result or Option is the error case.",
    explanation:
      "In a handler that returns Result<_, ApiError>, ? on a SpendError uses From to convert it and returns that error as the function's result. The rest of the function only runs on success. It is the short form of a match that returns the Err.",
    example: "let amount = Money::parse(&body.amount)?;",
    bookLink: "https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/error/result/enter_question_mark.html",
  },
  {
    id: "custom-errors",
    title: "Custom errors",
    summary: "An error enum names each failure in plain language.",
    explanation:
      "SpendError has variants such as ZeroAmount, BadAmount, UnknownCategory, and NotFound. thiserror writes the Display text, so the API can put err.to_string() into {\"error\": \"...\"} without a separate message table.",
    example: "#[error(\"expense {0} not found\")]\nNotFound(u32),",
    bookLink: "https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#choosing-what-to-return-from-a-function",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/define_error_type.html",
  },
  {
    id: "from-into",
    title: "From and Into",
    summary: "From converts one type into another. Into is the mirror.",
    explanation:
      "impl From<SpendError> for ApiError lets ? do the conversion. You write the conversion once. Into is implemented automatically for the other direction. Not every pair of types has From — that is the E0277 error.",
    example: "impl From<SpendError> for ApiError {\n    fn from(value: SpendError) -> Self {\n        ApiError::Spend(value)\n    }\n}",
    bookLink: "https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/conversion/from_into.html",
  },
  {
    id: "traits",
    title: "Traits",
    summary: "A trait is a shared set of methods.",
    explanation:
      "Display, FromStr, and Store are traits. Category implements Display and FromStr so it can be printed and parsed. JsonFileStore and MemoryStore both implement Store, so the API can save without knowing which one it has.",
    example: "pub trait Store: Send + Sync {\n    fn load(&self) -> Result<Ledger, StoreError>;\n    fn save(&self, ledger: &Ledger) -> Result<(), StoreError>;\n}",
    bookLink: "https://doc.rust-lang.org/book/ch10-02-traits.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/trait.html",
  },
  {
    id: "derive",
    title: "Derive",
    summary: "derive asks the compiler to write a trait implementation.",
    explanation:
      "#[derive(Debug, Clone, Serialize)] generates boring, correct code. Debug is for logs, Clone copies the value, Serialize writes JSON. You still write Display and FromStr by hand when the generated version would be wrong.",
    example: "#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\npub struct Money(pub u64);",
    bookLink: "https://doc.rust-lang.org/book/ch05-02-example-structs.html#adding-useful-functionality-with-derived-traits",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/trait/derive.html",
  },
  {
    id: "display",
    title: "Display",
    summary: "Display is the human-readable form used by {} and to_string().",
    explanation:
      "Money's Display prints grouped decimals such as 1,234.50. Category's Display prints food. The API sends that string as display next to the integer amount so the browser does not reimplement rounding.",
    example: "impl fmt::Display for Money {\n    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\n        write!(f, \"{}.{:02}\", with_commas(self.0 / 100), self.0 % 100)\n    }\n}",
    bookLink: "https://doc.rust-lang.org/book/ch05-02-example-structs.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/hello/print/print_display.html",
  },
  {
    id: "fromstr",
    title: "FromStr",
    summary: "FromStr parses text into a typed value.",
    explanation:
      "\"FOOD\".parse::<Category>() works because Category implements FromStr. The error type is SpendError. The CLI and the HTTP path both call it, so a bad category is the same error in both places.",
    example: "impl FromStr for Category {\n    type Err = SpendError;\n    fn from_str(text: &str) -> Result<Self, Self::Err> { /* ... */ }\n}",
    bookLink: "https://doc.rust-lang.org/book/ch10-02-traits.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/conversion/string.html",
  },
  {
    id: "newtype",
    title: "Newtype",
    summary: "A tuple struct with one field makes a distinct type.",
    explanation:
      "Money(pub u64) is not a u64. A function that wants money will not accept an expense id. The wrapper costs nothing at runtime and gives the amount its own methods, such as parse and Display.",
    example: "pub struct Money(pub u64);\nlet lunch = Money::parse(\"12.50\")?;",
    bookLink: "https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/generics/new_types.html",
  },
  {
    id: "impl-blocks",
    title: "impl blocks",
    summary: "impl adds methods and associated functions to a type.",
    explanation:
      "Money::parse is an associated function: you call it on the type. cents is a method: you call it on a value. Ledger's methods live in impl Ledger. If the method is not in an impl block, the compiler reports E0599.",
    example: "impl Money {\n    pub fn parse(input: &str) -> Result<Money, SpendError> { /* ... */ }\n    pub fn cents(self) -> u64 { self.0 }\n}",
    bookLink: "https://doc.rust-lang.org/book/ch05-03-method-syntax.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/fn/methods.html",
  },
  {
    id: "iterators",
    title: "Iterators",
    summary: "Iterators walk a collection without an index variable.",
    explanation:
      "list is iter, filter, and collect. total maps each expense to its Money and sums. The vector itself is not copied; the iterator yields references until collect builds the result you asked for.",
    example: "self.expenses.iter().filter(|expense| filter.matches(expense)).collect()",
    bookLink: "https://doc.rust-lang.org/book/ch13-02-iterators.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/trait/iter.html",
  },
  {
    id: "closures",
    title: "Closures",
    summary: "A closure is a small function that can use nearby variables.",
    explanation:
      "filter(|expense| filter.matches(expense)) passes a closure to the iterator. It captures filter from the surrounding function. You do not have to name a separate fn for a one-off test.",
    example: ".filter(|expense| filter.matches(expense))",
    bookLink: "https://doc.rust-lang.org/book/ch13-01-closures.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/fn/closures.html",
  },
  {
    id: "vec",
    title: "Vec",
    summary: "Vec<T> is an owned, growable list.",
    explanation:
      "The ledger stores expenses in a Vec. push adds one. remove(index) takes one out and shifts the later rows so ids stay in order. The length lives with the allocation; you do not track it yourself.",
    example: "self.expenses.push(expense);\nlet removed = self.expenses.remove(index);",
    bookLink: "https://doc.rust-lang.org/book/ch08-01-vectors.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/std/vec.html",
  },
  {
    id: "hashmap",
    title: "HashMap",
    summary: "HashMap looks up a value by key, without keeping a sort order.",
    explanation:
      "Monthly budgets are HashMap<Category, Money>. set_budget inserts one category. Lookup is by the category, not by position. The report does not rely on the map's order; by_category uses a BTreeMap when order matters.",
    example: "self.budgets.insert(category, amount);",
    bookLink: "https://doc.rust-lang.org/book/ch08-03-hash-maps.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/std/hash.html",
  },
  {
    id: "btreemap",
    title: "BTreeMap",
    summary: "BTreeMap is a map that keeps its keys sorted.",
    explanation:
      "by_category returns BTreeMap<Category, Money>. Category implements Ord, so food always prints before transport. A HashMap would make the chart jump around between requests.",
    example: "let mut totals = BTreeMap::new();\ntotals.entry(expense.category).or_insert(Money(0));",
    bookLink: "https://doc.rust-lang.org/book/ch08-03-hash-maps.html",
    exampleLink: "https://doc.rust-lang.org/std/collections/struct.BTreeMap.html",
  },
  {
    id: "generics",
    title: "Generics",
    summary: "A generic parameter stands for many concrete types.",
    explanation:
      "Sum for Money is implemented for any iterator of Money: fn sum<I>(iter: I) where I: Iterator<Item = Money>. A vector, a map's values, or a filter chain can all be summed without a separate function for each.",
    example: "fn sum<I>(iter: I) -> Self\nwhere\n    I: Iterator<Item = Money>,",
    bookLink: "https://doc.rust-lang.org/book/ch10-01-syntax.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/generics.html",
  },
  {
    id: "trait-objects",
    title: "Trait objects",
    summary: "dyn Trait means some type that implements the trait, chosen at runtime.",
    explanation:
      "Box<dyn Store + Send + Sync> holds either a JsonFileStore or a MemoryStore. The API calls load and save through the trait. The concrete type is erased, which is why tests can use memory and the CLI can use a file.",
    example: "pub store: Box<dyn Store + Send + Sync>",
    bookLink: "https://doc.rust-lang.org/book/ch17-02-trait-objects.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/trait/dyn.html",
  },
  {
    id: "arc",
    title: "Arc",
    summary: "Arc shares ownership across tasks with a reference count.",
    explanation:
      "Axum clones the state into every handler. Cloning an Arc only increments a counter; it does not clone the ledger. The last clone to drop frees the state. Arc is for sharing, not for mutation by itself.",
    example: "pub type SharedState = Arc<AppState>;",
    bookLink: "https://doc.rust-lang.org/book/ch16-03-shared-state.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/std/arc.html",
  },
  {
    id: "mutex",
    title: "Mutex",
    summary: "A mutex lets one task at a time change shared data.",
    explanation:
      "The ledger sits in tokio::sync::Mutex. lock().await waits until this task can take the guard, which acts like &mut Ledger. tokio's mutex does not poison: a panic in another task does not permanently lock it. Hold the guard only while you change the ledger, then save.",
    example: "let mut ledger = state.ledger.lock().await;\nledger.add(draft)?;",
    bookLink: "https://doc.rust-lang.org/book/ch16-03-shared-state.html#atomic-reference-counting-with-refcellt",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/std/arc.html",
  },
  {
    id: "async-await",
    title: "async and await",
    summary: "async functions return a future. await pauses until it is ready.",
    explanation:
      "Handlers are async fn. They do not run until axum polls them. await on the mutex lock pauses this request without blocking every other request. The question mark still works inside an async function that returns Result.",
    example: "pub async fn health() -> Json<HealthBody> {\n    Json(HealthBody { ok: true })\n}",
    bookLink: "https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/async.html",
  },
  {
    id: "tokio",
    title: "Tokio",
    summary: "Tokio is the async runtime that polls futures and speaks to the operating system.",
    explanation:
      "The CLI builds a multi-thread tokio runtime only for serve. axum::serve accepts connections on that runtime. ctrl_c().await waits for Ctrl-C, then the server stops accepting new connections and finishes the ones already running.",
    example: "axum::serve(listener, app)\n    .with_graceful_shutdown(shutdown_signal())\n    .await?;",
    bookLink: "https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html",
    exampleLink: "https://tokio.rs/tokio/tutorial",
  },
  {
    id: "serde",
    title: "Serde",
    summary: "Serde turns Rust values into JSON and back.",
    explanation:
      "Expense derives Serialize and Deserialize. Money is a newtype, so the file stores a bare integer of cents. The HTTP response is a different struct that adds a display string. Request amounts stay strings so JSON numbers never become floats.",
    example: "#[derive(Serialize, Deserialize)]\npub struct Expense { pub amount: Money, /* ... */ }",
    bookLink: "https://doc.rust-lang.org/book/ch05-02-example-structs.html",
    exampleLink: "https://serde.rs/",
  },
  {
    id: "modules-visibility",
    title: "Modules and visibility",
    summary: "pub decides what other modules and crates can use.",
    explanation:
      "lib.rs lists pub mod money and the rest. Items inside a module are private until marked pub. The ledger's vector is private, so other crates add expenses through methods instead of pushing on the list themselves.",
    example: "pub mod money;\n// expenses: Vec<Expense>  // private field",
    bookLink: "https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/mod.html",
  },
  {
    id: "testing",
    title: "Testing",
    summary: "#[test] functions run under cargo test and stay out of the release binary.",
    explanation:
      "#[cfg(test)] mod tests compiles only for cargo test. A table of inputs next to expected Results is a good fit for Money::parse. Integration tests in tests/ talk to the public API or the compiled binary the way a user would.",
    example: "#[test]\nfn parse_table() {\n    assert_eq!(Money::parse(\"12.50\").map(|m| m.cents()), Ok(1250));\n}",
    bookLink: "https://doc.rust-lang.org/book/ch11-01-writing-tests.html",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/testing.html",
  },
  {
    id: "doc-tests",
    title: "Documentation tests",
    summary: "Code samples in /// comments are compiled and run by cargo test.",
    explanation:
      "Money::parse has a # Examples block. cargo test compiles that sample as a separate crate, so a stale snippet fails the build. That is why the example uses the public path spend_core::Money.",
    example: "/// ```\n/// use spend_core::Money;\n/// let money = Money::parse(\"12.50\").expect(\"valid\");\n/// assert_eq!(money.cents(), 1250);\n/// ```",
    bookLink: "https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#documentation-comments",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/meta/doc.html",
  },
  {
    id: "integer-money",
    title: "Integer money",
    summary: "Store currency as whole minor units, never as floats.",
    explanation:
      "0.1 + 0.2 is not 0.3 in binary floating point. Money is a u64 of cents. \"12.50\" parses to 1250 by looking at digits, with no f64 step. The CLI treats 450 as 450.00. The API sends the amount as a string for the same reason.",
    example: "Money::parse(\"12.50\") // Ok(Money(1250))\nMoney::parse(\"1.234\") // Err — more than two decimal places",
    bookLink: "https://doc.rust-lang.org/book/ch03-02-data-types.html#integer-types",
    exampleLink: "https://doc.rust-lang.org/rust-by-example/primitives/literals.html",
  },
] as const

export const concepts: Concept[] = definitions.map((concept) => ({
  ...concept,
  usedIn: usedInByConcept[concept.id] ?? [],
}))

export function conceptById(id: string): Concept | undefined {
  return concepts.find((concept) => concept.id === id)
}
