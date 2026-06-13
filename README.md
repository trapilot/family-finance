# Family Finance

A local-first personal and family finance management app built with Rust, Dioxus, and SQLite.

The application is designed for users who want full control over their financial data without relying on cloud services. All data is stored locally and can be backed up using a portable `.ffbackup` file.

---

## Features

### Cash Flow Management

* Income tracking
* Expense tracking
* Wallet-to-wallet transfers
* Multi-wallet support
* Transaction history and filtering
* Category management
* Budget monitoring

### Investment Management

* Stock and crypto holdings
* Buy and sell transactions
* Weighted average purchase price calculation
* Realized profit & loss tracking
* Unrealized profit & loss tracking
* Manual market price updates

### Multi-Currency Support

Currencies are intentionally separated and **never automatically converted**.

Supported currencies:

| Currency | Unit                |
| -------- | ------------------- |
| VND      | Vietnamese Dong     |
| USD      | US Dollar           |
| GOLD     | Chỉ (1 chỉ = 3.75g) |

Cross-currency transfers require manual input of the received amount.

Example:

```text
8,500,000 VND → 0.5 GOLD
```

---

## Technology Stack

```toml
dioxus        = { version = "0.7", features = ["mobile"] }
rusqlite      = { version = "0.31", features = ["bundled"] }
serde         = { version = "1", features = ["derive"] }
serde_json    = "1"
uuid          = { version = "1", features = ["v4"] }
chrono        = { version = "0.4", features = ["serde"] }
rust_decimal  = { version = "1", features = ["serde-float"] }
once_cell     = "1"
```

### Frontend

* Dioxus 0.7
* Mobile renderer (WebView-based)
* Cross-platform UI

### Backend

* Rust
* SQLite
* Local-first architecture
* Offline operation

---

## Project Goals

* Single-user finance management
* Family finance organization
* Offline-first experience
* Simple and transparent financial records
* Easy backup and restore
* No cloud dependency

---

## Transaction Types

### Income

```text
wallet.balance += amount
```

Income categories:

* Salary
* Bonus
* Side Income
* Rental
* Investment
* Other

### Expense

```text
wallet.balance -= amount
```

### Transfer

```text
from.balance -= amount
to.balance += to_amount
```

Same currency:

```text
amount == to_amount
```

Cross currency:

```text
amount != to_amount
```

Example:

```text
10,000,000 VND → 0.6 GOLD
```

### Investment Buy

```text
wallet.balance -= quantity × price
holding.quantity += quantity
```

Average purchase price is recalculated using weighted average.

### Investment Sell

```text
holding.quantity -= quantity
wallet.balance += quantity × price
```

Realized P&L:

```text
(price - avg_buy_price) × quantity
```

---

## Wallet Types

```text
cash
bank
e_wallet
investment
```

---

## Asset Types

```text
stock
crypto
```

---

## Application Structure

```text
Dashboard
├── Net Worth by Currency
├── Monthly Income vs Expense
├── Savings & Savings Rate
├── Overspending Alerts
└── Recent Transactions

Transactions
├── Transaction List
├── Filters
├── Add Transaction
├── Edit Transaction
└── Delete Transaction

Wallets
├── Wallet Overview
├── Currency Groups
├── Investment Wallets
├── Wallet Details
└── Holdings & P&L

Reports
├── Expense by Category
├── Income vs Expense Timeline
└── Investment Performance

Settings
├── Categories
├── Export Backup
├── Import Backup
└── Family Members (Phase 2)
```

---

## Project Structure

```text
family-finance/
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── db/
│   │   ├── mod.rs
│   │   └── migrations.rs
│   ├── models/
│   │   ├── wallet.rs
│   │   ├── transaction.rs
│   │   ├── category.rs
│   │   ├── holding.rs
│   │   └── member.rs
│   ├── repository/
│   │   ├── wallet_repo.rs
│   │   ├── transaction_repo.rs
│   │   ├── category_repo.rs
│   │   └── holding_repo.rs
│   ├── services/
│   │   ├── transaction_service.rs
│   │   ├── holding_service.rs
│   │   └── report_service.rs
│   └── views/
│       ├── dashboard.rs
│       ├── transactions/
│       ├── wallets/
│       ├── reports/
│       └── settings/
├── Cargo.toml
└── Dioxus.toml
```

---

## Backup Format

The application supports exporting and importing a portable backup file with the `.ffbackup` extension.

Example:

```json
{
  "app_version": "1.0.0",
  "exported_at": 1234567890,
  "wallets": [],
  "holdings": [],
  "categories": [],
  "transactions": [],
  "members": [],
  "settings": {}
}
```

---

## Roadmap

### Phase 1

* Wallet management
* Transaction management
* Category management
* Budget tracking
* Investment tracking
* Reports and analytics
* Backup and restore

### Phase 2

* Family member profiles
* Household information
* Shared family finance records

---

## Design Principles

* Local-first
* Offline-first
* Privacy-focused
* No mandatory accounts
* No cloud dependency
* Simple data model
* Portable backups

---

## Current Status

🚧 In Development

The project is currently focused on completing the core finance and investment management features before expanding into family profile management.

---

## License

MIT License

```
```
