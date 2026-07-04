pub mod category;
pub mod family;
pub mod holding;
pub mod member;
pub mod transaction;
pub mod wallet;

pub use category::{Category, NewCategory};
pub use family::{Family, NewFamily};
pub use holding::{AssetType, Holding };
pub use member::{Member, NewMember};
pub use transaction::{IncomeType, NewTransaction, Transaction, TransactionType};
pub use wallet::{Currency, NewWallet, Wallet, WalletType};

use rust_decimal::Decimal;

pub(crate) fn f64_to_decimal(v: f64) -> Decimal {
    Decimal::try_from(v).unwrap_or(Decimal::ZERO)
}

pub(crate) fn decimal_to_f64(d: Decimal) -> f64 {
    use rust_decimal::prelude::ToPrimitive;
    d.to_f64().unwrap_or(0.0)
}