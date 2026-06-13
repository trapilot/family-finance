pub mod detail;
pub mod form;
pub mod holdings;
pub mod list;

use dioxus::prelude::*;
use crate::models::Wallet;
use list::WalletList;
use detail::WalletDetail;
use form::WalletForm;

#[derive(Clone, PartialEq, Debug)]
pub enum WalletView {
    List,
    Add,
    Edit(Wallet),
    Detail(Wallet),
}

#[component]
pub fn Wallets() -> Element {
    let view = use_signal(|| WalletView::List);
    let current_view = view.read().clone();

    match current_view {
        WalletView::List         => rsx! { WalletList { view } },
        WalletView::Add          => rsx! { WalletForm { view, editing: None } },
        WalletView::Edit(wallet) => rsx! { WalletForm { view, editing: Some(wallet) } },
        WalletView::Detail(w)    => rsx! { WalletDetail { view, wallet: w } },
    }
    
    // match view.read().clone() {
    //     WalletView::List         => rsx! { WalletList { view } },
    //     WalletView::Add          => rsx! { WalletForm { view, editing: None } },
    //     WalletView::Edit(wallet) => rsx! { WalletForm { view, editing: Some(wallet) } },
    //     WalletView::Detail(w)    => rsx! { WalletDetail { view, wallet: w } },
    // }
}
