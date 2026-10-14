//! Text tables for `spend report`. The numbers come from [`crate::Ledger`].

use std::collections::BTreeMap;

use crate::category::Category;
use crate::ledger::BudgetStatus;
use crate::money::Money;

pub fn format_report(
    month: &str,
    totals: &BTreeMap<Category, Money>,
    statuses: &[BudgetStatus],
    total: Money,
) -> String {
    let mut lines = vec![month.to_string(), String::new()];
    if totals.is_empty() {
        lines.push("no expenses".to_string());
    } else {
        lines.push(format!("{:<12} {:>12}", "category", "spent"));
        for (category, amount) in totals {
            lines.push(format!("{category:<12} {:>12}", amount.to_string()));
        }
        lines.push(format!("{:<12} {:>12}", "total", total.to_string()));
    }

    lines.push(String::new());
    lines.push("budgets".to_string());
    if statuses.is_empty() {
        lines.push("no budgets".to_string());
    } else {
        for status in statuses {
            let flag = if status.over { "over" } else { "ok" };
            lines.push(format!(
                "{:<12} budget {:>10}  spent {:>10}  {flag}",
                status.category, status.budget, status.spent
            ));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::Ledger;
    use crate::money::Money;
    use crate::NewExpense;

    #[test]
    fn report_includes_category_and_budget() {
        let mut ledger = Ledger::default();
        ledger
            .add(NewExpense {
                amount: Money::parse("12.50").expect("amount"),
                category: Category::Food,
                note: Some("lunch".to_string()),
                date: chrono::NaiveDate::from_ymd_opt(2026, 10, 14).expect("date"),
            })
            .expect("add");
        ledger.set_budget(Category::Food, Money::parse("150.00").expect("budget"));
        let month = "2026-10";
        let text = format_report(
            month,
            &ledger.by_category(Some(month)),
            &ledger.budget_status(month),
            ledger.total(&crate::Filter {
                month: Some(month.to_string()),
                ..crate::Filter::default()
            }),
        );
        assert!(text.contains("food"));
        assert!(text.contains("12.50"));
        assert!(text.contains("150.00"));
        assert!(text.contains("ok"));
    }
}
