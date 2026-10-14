export type AmountResult = { ok: true; cents: number } | { ok: false; error: string }

/** Same rules as `Money::parse` in `crates/core/src/money.rs`. */
export function parseAmount(input: string): AmountResult {
  const text = input.trim()
  if (text.length === 0 || text.startsWith("+") || text.startsWith("-")) {
    return { ok: false, error: `invalid amount: ${text}` }
  }

  const dot = text.indexOf(".")
  let whole = text
  let frac = ""
  if (dot !== -1) {
    whole = text.slice(0, dot)
    frac = text.slice(dot + 1)
    if (frac.length === 0 || frac.includes(".")) {
      return { ok: false, error: `invalid amount: ${text}` }
    }
  }

  if (!/^\d+$/.test(whole) || !/^\d*$/.test(frac) || frac.length > 2) {
    return { ok: false, error: `invalid amount: ${text}` }
  }

  const major = Number(whole)
  const minor = frac.length === 0 ? 0 : frac.length === 1 ? Number(frac) * 10 : Number(frac)
  if (!Number.isSafeInteger(major) || !Number.isSafeInteger(minor)) {
    return { ok: false, error: `invalid amount: ${text}` }
  }
  const cents = major * 100 + minor
  if (!Number.isSafeInteger(cents)) {
    return { ok: false, error: `invalid amount: ${text}` }
  }
  return { ok: true, cents }
}

export function validateExpenseAmount(input: string): AmountResult {
  const parsed = parseAmount(input)
  if (!parsed.ok) {
    return parsed
  }
  if (parsed.cents === 0) {
    return { ok: false, error: "amount must be greater than zero" }
  }
  return parsed
}

export function formatMoney(cents: number): string {
  const negative = cents < 0
  const absolute = Math.abs(cents)
  const major = Math.floor(absolute / 100)
  const minor = absolute % 100
  const digits = String(major)
  let grouped = ""
  for (let index = 0; index < digits.length; index += 1) {
    if (index > 0 && (digits.length - index) % 3 === 0) {
      grouped += ","
    }
    grouped += digits[index] ?? ""
  }
  return `${negative ? "-" : ""}${grouped}.${String(minor).padStart(2, "0")}`
}
