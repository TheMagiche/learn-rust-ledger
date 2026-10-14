import { Bar, BarChart, Cell, Pie, PieChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts"

import { CATEGORY_COLORS } from "../lib/handlers"
import { useTheme } from "../theme"
import type { CategoryTotal } from "../types"

type CategoryChartProps = {
  rows: CategoryTotal[]
}

type ChartRow = CategoryTotal & { major: number }

function ChartTip({ active, payload }: { active?: boolean; payload?: Array<{ payload?: ChartRow }> }) {
  if (!active || !payload?.[0]?.payload) {
    return null
  }
  const row = payload[0].payload
  return (
    <div className="rounded-md border border-line bg-card px-2 py-1 text-sm dark:border-night-line dark:bg-night-card">
      <span className="capitalize">{row.category}</span> {row.display}
    </div>
  )
}

export function CategoryChart({ rows }: CategoryChartProps) {
  const { theme } = useTheme()
  const data: ChartRow[] = rows.map((row) => ({ ...row, major: row.amount / 100 }))
  const axis = theme === "dark" ? "#b7ada2" : "#6d645b"

  if (data.length === 0) {
    return (
      <section className="rounded-xl border border-line bg-card p-4 dark:border-night-line dark:bg-night-card">
        <h2 className="font-serif text-2xl">By category</h2>
        <p className="mt-3 text-muted dark:text-night-muted">No spending in this month yet. The pie chart fills in after the first expense.</p>
      </section>
    )
  }

  return (
    <section className="rounded-xl border border-line bg-card p-4 dark:border-night-line dark:bg-night-card">
      <h2 className="font-serif text-2xl">By category</h2>
      <div className="mt-2 grid gap-2 md:grid-cols-2">
        <div className="h-56">
          <ResponsiveContainer width="100%" height="100%">
            <PieChart>
              <Pie data={data} dataKey="major" nameKey="category" innerRadius={42} outerRadius={72} paddingAngle={2}>
                {data.map((row) => (
                  <Cell key={row.category} fill={CATEGORY_COLORS[row.category] ?? "#8a8178"} />
                ))}
              </Pie>
              <Tooltip content={<ChartTip />} />
            </PieChart>
          </ResponsiveContainer>
        </div>
        <div className="h-56">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={data}>
              <XAxis dataKey="category" tick={{ fill: axis, fontSize: 12 }} />
              <YAxis tick={{ fill: axis, fontSize: 12 }} />
              <Tooltip content={<ChartTip />} />
              <Bar dataKey="major" radius={4}>
                {data.map((row) => (
                  <Cell key={row.category} fill={CATEGORY_COLORS[row.category] ?? "#8a8178"} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </section>
  )
}
