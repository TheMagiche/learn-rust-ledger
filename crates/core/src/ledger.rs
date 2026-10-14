//! The in-memory book of expenses and monthly budgets.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::category::Category;
use crate::error::SpendError;
use crate::expense::{month_key, Expense, NewExpense};
use crate::money::Money;

/// What `list` and `total` keep. Empty fields do not filter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    pub category: Option<Category>,
    pub month: Option<String>,
    pub search: Option<String>,
}

impl Filter {
    fn matches(&self, expense: &Expense) -> bool {
        if let Some(category) = self.category {
            if expense.category != category {
                return false;
            }
        }
        if let Some(month) = &self.month {
            if month_key(expense.date) != *month {
                return false;
            }
        }
        if let Some(search) = &self.search {
            let needle = search.trim().to_lowercase();
            if !needle.is_empty() {
                let in_note = expense
                    .note
                    .as_ref()
                    .is_some_and(|note| note.to_lowercase().contains(&needle));
                let in_category = expense.category.to_string().contains(&needle);
                if !in_note && !in_category {
                    return false;
                }
            }
        }
        true
    }
}

/// One category compared with its monthly cap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetStatus {
    pub category: Category,
    pub budget: Money,
    pub spent: Money,
    pub over: bool,
}

/// LEARN(vec): expenses live in a `Vec`, an owned growable list.
/// The fields are private so every change goes through a method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ledger {
    expenses: Vec<Expense>,
    #[serde(default = "default_next_id")]
    next_id: u32,
    /// LEARN(hashmap): budgets are a `HashMap` because we look up one category at a time.
    budgets: HashMap<Category, Money>,
}

fn default_next_id() -> u32 {
    1
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            expenses: Vec::new(),
            next_id: 1,
            budgets: HashMap::new(),
        }
    }
}

impl Ledger {
    /// LEARN(mut-borrow): `&mut self` means this method may change the ledger, and
    /// no other borrow of it can exist until the method returns.
    /// LEARN(ownership): `draft` is owned by `add`. We move its fields into a new `Expense`.
    pub fn add(&mut self, draft: NewExpense) -> Result<&Expense, SpendError> {
        if draft.amount.cents() == 0 {
            return Err(SpendError::ZeroAmount);
        }

        // LEARN(variables-mutability): `note` is `mut` so we can replace a blank string with `None`.
        // Bindings are immutable unless we write `mut`.
        let mut note = draft.note;
        if note.as_ref().is_some_and(|value| value.trim().is_empty()) {
            note = None;
        } else if let Some(value) = note {
            // LEARN(strings): `trim()` borrows; `to_string()` allocates an owned `String` we can store.
            note = Some(value.trim().to_string());
        }

        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.expenses.push(Expense {
            id,
            amount: draft.amount,
            category: draft.category,
            note,
            date: draft.date,
        });

        self.expenses.last().ok_or(SpendError::NotFound(id))
    }

    /// LEARN(move): `Vec::remove` hands back an owned `Expense`. The ledger no longer has it,
    /// so the value has moved to the caller. A shared reference (`&Expense`) would be wrong here:
    /// it would point into a list that no longer stores that item.
    pub fn remove(&mut self, id: u32) -> Result<Expense, SpendError> {
        let position = self.expenses.iter().position(|expense| expense.id == id);
        match position {
            Some(index) => Ok(self.expenses.remove(index)),
            None => Err(SpendError::NotFound(id)),
        }
    }

    /// LEARN(option): `Some` is the expense, `None` means that id is not in the book.
    /// LEARN(borrowing): `&self` and `&Expense` are shared borrows. The caller can read them
    /// and cannot change the ledger through them.
    pub fn get(&self, id: u32) -> Option<&Expense> {
        self.expenses.iter().find(|expense| expense.id == id)
    }

    /// LEARN(iterators): `iter`, `filter`, and `collect` walk the list without a manual index.
    /// LEARN(closures): `|expense| self.filter.matches(expense)` is a closure passed to `filter`.
    pub fn list(&self, filter: &Filter) -> Vec<&Expense> {
        self.expenses
            .iter()
            .filter(|expense| filter.matches(expense))
            .collect()
    }

    pub fn total(&self, filter: &Filter) -> Money {
        self.list(filter)
            .into_iter()
            .map(|expense| expense.amount)
            .sum()
    }

    /// LEARN(btreemap): a `BTreeMap` keeps category keys sorted, so reports stay in a stable order.
    pub fn by_category(&self, month: Option<&str>) -> BTreeMap<Category, Money> {
        let mut totals = BTreeMap::new();
        for expense in &self.expenses {
            if month.is_some_and(|month| month_key(expense.date) != month) {
                continue;
            }
            let entry = totals.entry(expense.category).or_insert(Money(0));
            *entry = *entry + expense.amount;
        }
        totals
    }

    pub fn set_budget(&mut self, category: Category, amount: Money) {
        self.budgets.insert(category, amount);
    }

    pub fn budget_status(&self, month: &str) -> Vec<BudgetStatus> {
        let spent = self.by_category(Some(month));
        let mut categories: Vec<Category> = self.budgets.keys().copied().collect();
        for category in spent.keys() {
            if !categories.contains(category) {
                categories.push(*category);
            }
        }
        categories.sort();
        categories
            .into_iter()
            .map(|category| {
                let budget = self.budgets.get(&category).copied().unwrap_or(Money(0));
                let spent_amount = spent.get(&category).copied().unwrap_or(Money(0));
                let has_budget = self.budgets.contains_key(&category);
                BudgetStatus {
                    category,
                    budget,
                    spent: spent_amount,
                    over: has_budget && spent_amount > budget,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::category::Category;

    fn date(text: &str) -> NaiveDate {
        NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("test date")
    }

    fn sample(ledger: &mut Ledger, amount: u64, category: Category, note: &str, day: &str) -> u32 {
        let expense = ledger
            .add(NewExpense {
                amount: Money(amount),
                category,
                note: Some(note.to_string()),
                date: date(day),
            })
            .expect("add");
        expense.id
    }

    #[test]
    fn add_assigns_ids_and_rejects_zero() {
        let mut ledger = Ledger::default();
        let err = ledger
            .add(NewExpense {
                amount: Money(0),
                category: Category::Food,
                note: None,
                date: date("2026-10-14"),
            })
            .expect_err("zero");
        assert_eq!(err, SpendError::ZeroAmount);

        let first = sample(&mut ledger, 1_250, Category::Food, "lunch", "2026-10-14");
        let second = sample(&mut ledger, 400, Category::Transport, "bus", "2026-10-15");
        assert_eq!(first, 1);
        assert_eq!(second, 2);
        assert_eq!(ledger.get(1).expect("first").note.as_deref(), Some("lunch"));
    }

    #[test]
    fn blank_note_becomes_none() {
        let mut ledger = Ledger::default();
        ledger
            .add(NewExpense {
                amount: Money(100),
                category: Category::Fun,
                note: Some("   ".to_string()),
                date: date("2026-10-01"),
            })
            .expect("add");
        assert_eq!(ledger.get(1).expect("row").note, None);
    }

    #[test]
    fn remove_returns_owned_expense() {
        let mut ledger = Ledger::default();
        sample(&mut ledger, 100, Category::Rent, "october", "2026-10-01");
        let removed = ledger.remove(1).expect("removed");
        assert_eq!(removed.category, Category::Rent);
        assert_eq!(ledger.remove(1), Err(SpendError::NotFound(1)));
        assert!(ledger.get(1).is_none());
    }

    #[test]
    fn filters_and_total() {
        let mut ledger = Ledger::default();
        sample(&mut ledger, 1_000, Category::Food, "lunch", "2026-10-14");
        sample(&mut ledger, 2_000, Category::Food, "dinner", "2026-09-01");
        sample(&mut ledger, 500, Category::Transport, "bus", "2026-10-02");

        let october_food = Filter {
            category: Some(Category::Food),
            month: Some("2026-10".to_string()),
            search: None,
        };
        assert_eq!(ledger.list(&october_food).len(), 1);
        assert_eq!(ledger.total(&october_food), Money(1_000));

        let search = Filter {
            category: None,
            month: None,
            search: Some("BUS".to_string()),
        };
        assert_eq!(ledger.list(&search).len(), 1);
        assert_eq!(ledger.total(&Filter::default()), Money(3_500));
    }

    #[test]
    fn budgets_flag_overspend() {
        let mut ledger = Ledger::default();
        sample(
            &mut ledger,
            2_000,
            Category::Food,
            "groceries",
            "2026-10-03",
        );
        ledger.set_budget(Category::Food, Money(1_500));
        ledger.set_budget(Category::Rent, Money(50_000));

        let status = ledger.budget_status("2026-10");
        let food = status
            .iter()
            .find(|row| row.category == Category::Food)
            .expect("food");
        assert!(food.over);
        assert_eq!(food.spent, Money(2_000));
        let rent = status
            .iter()
            .find(|row| row.category == Category::Rent)
            .expect("rent");
        assert!(!rent.over);
        assert_eq!(
            ledger.by_category(Some("2026-10"))[&Category::Food],
            Money(2_000)
        );
    }
}
