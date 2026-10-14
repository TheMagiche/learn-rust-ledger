import { describe, expect, it } from "vitest"

import { formatMoney, parseAmount, validateExpenseAmount } from "./money"

describe("parseAmount", () => {
  const cases: Array<[string, number | null]> = [
    ["0", 0],
    ["0.00", 0],
    ["12", 1200],
    ["12.5", 1250],
    ["12.50", 1250],
    ["450", 45000],
    ["1234.50", 123450],
    ["  8.05 ", 805],
    ["-1", null],
    ["1.234", null],
    ["abc", null],
    ["", null],
    [".50", null],
    ["12.", null],
    ["+4", null],
    ["1,234.50", null],
  ]

  it.each(cases)("parses %j", (input, expected) => {
    const parsed = parseAmount(input)
    if (expected === null) {
      expect(parsed.ok).toBe(false)
    } else {
      expect(parsed).toEqual({ ok: true, cents: expected })
    }
  })

  it("rejects zero when recording an expense", () => {
    expect(validateExpenseAmount("0")).toEqual({
      ok: false,
      error: "amount must be greater than zero",
    })
    expect(validateExpenseAmount("abc")).toEqual({ ok: false, error: "invalid amount: abc" })
  })

  it("formats grouped decimals", () => {
    expect(formatMoney(0)).toBe("0.00")
    expect(formatMoney(1250)).toBe("12.50")
    expect(formatMoney(123450)).toBe("1,234.50")
  })
})
