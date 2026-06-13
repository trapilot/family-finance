pub mod form;
pub mod list;

use dioxus::prelude::*;
use crate::models::Transaction;
use list::TransactionList;
use form::TransactionForm;

#[derive(Clone, PartialEq, Debug)]
pub enum TxnView {
    List,
    Add,
    Edit(Transaction),
}

#[component]
pub fn Transactions() -> Element {
    let view = use_signal(|| TxnView::List);

    let current_view = view.read().clone();
    match current_view {
        TxnView::List => rsx! { TransactionList { view } },
        TxnView::Add  => rsx! { TransactionForm { view, editing: None } },
        TxnView::Edit(txn) => rsx! { TransactionForm { view, editing: Some(txn) } },
    }
}
