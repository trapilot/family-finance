pub mod detail;
pub mod form;
pub mod list;
pub mod family_form;
pub mod family_detail;

use dioxus::prelude::*;
use crate::models::{Family, Member};
use list::MemberList;
use detail::MemberDetail;
use form::MemberForm;
use family_form::FamilyForm;
use family_detail::FamilyDetail;

#[derive(Clone, PartialEq, Debug)]
pub enum MemberView {
    List,
    Add,
    Edit(Member),
    Detail(Member),
    AddFamily,
    EditFamily(Family),
    FamilyDetail(Family),
}

#[component]
pub fn Members() -> Element {
    let view = use_signal(|| MemberView::List);

    let current_view = view.read().clone();
    match current_view {
        MemberView::List          => rsx! { MemberList { view } },
        MemberView::Add           => rsx! { MemberForm { view, editing: None } },
        MemberView::Edit(member)  => rsx! { MemberForm { view, editing: Some(member) } },
        MemberView::Detail(member) => rsx! { MemberDetail { view, member } },
        MemberView::AddFamily     => rsx! { FamilyForm { view, editing: None } },
        MemberView::EditFamily(family) => rsx! { FamilyForm { view, editing: Some(family) } },
        MemberView::FamilyDetail(family) => rsx! { FamilyDetail { view, family } },
    }
}
