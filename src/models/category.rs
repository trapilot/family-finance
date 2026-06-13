use rusqlite::Row;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{decimal_to_f64, f64_to_decimal};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Category {
    pub id:            String,
    pub name:          String,
    pub icon:          Option<String>,
    pub color:         Option<String>,
    pub budget_amount: Option<Decimal>,
    pub parent_id:     Option<String>,
    pub sort_order:    i32,
    pub is_system:     bool,
    pub created_at:    i64,
}

impl Category {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let budget_f64: Option<f64> = row.get("budget_amount")?;
        let is_system_i: i32 = row.get("is_system")?;

        Ok(Category {
            id:            row.get("id")?,
            name:          row.get("name")?,
            icon:          row.get("icon")?,
            color:         row.get("color")?,
            budget_amount: budget_f64.map(f64_to_decimal),
            parent_id:     row.get("parent_id")?,
            sort_order:    row.get("sort_order")?,
            is_system:     is_system_i != 0,
            created_at:    row.get("created_at")?,
        })
    }

    pub fn budget_f64(&self) -> Option<f64> {
        self.budget_amount.map(decimal_to_f64)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewCategory {
    pub name:          String,
    pub icon:          Option<String>,
    pub color:         Option<String>,
    pub budget_amount: Option<Decimal>,
    pub parent_id:     Option<String>,
    pub sort_order:    i32,
}

/// System default categories seeded on first launch
pub fn default_categories() -> Vec<NewCategory> {
    vec![
        NewCategory { name: "Food & Drinks".into(),     icon: Some("🍜".into()), color: Some("#FF6B6B".into()), budget_amount: None, parent_id: None, sort_order: 1  },
        NewCategory { name: "Transport".into(),          icon: Some("🚗".into()), color: Some("#4ECDC4".into()), budget_amount: None, parent_id: None, sort_order: 2  },
        NewCategory { name: "Shopping".into(),           icon: Some("🛍️".into()), color: Some("#45B7D1".into()), budget_amount: None, parent_id: None, sort_order: 3  },
        NewCategory { name: "Health".into(),             icon: Some("💊".into()), color: Some("#96CEB4".into()), budget_amount: None, parent_id: None, sort_order: 4  },
        NewCategory { name: "Entertainment".into(),      icon: Some("🎮".into()), color: Some("#FFEAA7".into()), budget_amount: None, parent_id: None, sort_order: 5  },
        NewCategory { name: "Education".into(),          icon: Some("📚".into()), color: Some("#DDA0DD".into()), budget_amount: None, parent_id: None, sort_order: 6  },
        NewCategory { name: "Bills & Utilities".into(),  icon: Some("🏠".into()), color: Some("#F0E68C".into()), budget_amount: None, parent_id: None, sort_order: 7  },
        NewCategory { name: "Personal Care".into(),      icon: Some("💆".into()), color: Some("#FFB6C1".into()), budget_amount: None, parent_id: None, sort_order: 8  },
        NewCategory { name: "Travel".into(),             icon: Some("✈️".into()), color: Some("#87CEEB".into()), budget_amount: None, parent_id: None, sort_order: 9  },
        NewCategory { name: "Other".into(),              icon: Some("📦".into()), color: Some("#D3D3D3".into()), budget_amount: None, parent_id: None, sort_order: 10 },
    ]
}
