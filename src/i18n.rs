use dioxus::prelude::*;

// ─── Language ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Lang {
    #[default]
    En,
    Vi,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::En, Lang::Vi];

    /// Code stored in the `settings` table
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Vi => "vi",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "vi" => Lang::Vi,
            _    => Lang::En,
        }
    }

    /// Name shown in the language dropdown (always in its own language)
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Vi => "Tiếng Việt",
        }
    }
}

// ─── I18n handle ──────────────────────────────────────────────────────────────

/// Copyable handle to the current language. Reading it inside a component subscribes
/// that component, so switching language re-renders every screen.
#[derive(Clone, Copy, PartialEq)]
pub struct I18n {
    lang: Signal<Lang>,
}

impl I18n {
    pub fn new(lang: Signal<Lang>) -> Self {
        Self { lang }
    }

    pub fn lang(&self) -> Lang {
        (self.lang)()
    }

    pub fn set_lang(&mut self, lang: Lang) {
        self.lang.set(lang);
    }

    /// Translate UI text. The English text is the key; falls back to it when no translation exists.
    pub fn t(&self, en: &'static str) -> &'static str {
        match self.lang() {
            Lang::En => en,
            Lang::Vi => vi(en).unwrap_or(en),
        }
    }

    /// Translate runtime text (e.g. seeded category names stored in the DB).
    pub fn tr(&self, text: &str) -> String {
        match self.lang() {
            Lang::En => text.to_string(),
            Lang::Vi => vi(text).unwrap_or(text).to_string(),
        }
    }

    /// Translate a stored enum code (e.g. `WalletType` "e_wallet") into a display label.
    pub fn code(&self, code: &str) -> String {
        let en = code_label(code);
        match en {
            Some(en) => self.t(en).to_string(),
            None     => code.to_string(),
        }
    }
}

pub fn use_i18n() -> I18n {
    use_context::<I18n>()
}

/// English labels for the codes produced by the models' `Display` impls
fn code_label(code: &str) -> Option<&'static str> {
    Some(match code {
        // WalletType
        "cash"        => "Cash",
        "bank"        => "Bank",
        "loan"        => "Loan",
        "e_wallet"    => "E-Wallet",
        "investment"  => "Investment",
        // TransactionType
        "income"      => "Income",
        "expense"     => "Expense",
        "transfer"    => "Transfer",
        "invest_buy"  => "Buy",
        "invest_sell" => "Sell",
        // IncomeType
        "salary"      => "Salary",
        "bonus"       => "Bonus",
        "side_income" => "Side Income",
        "rental"      => "Rental",
        "other"       => "Other",
        // AssetType
        "stock"       => "Stock",
        "crypto"      => "Crypto",
        // MemberRole
        "owner"       => "Owner",
        "member"      => "Member",
        // Gender
        "male"        => "Male",
        "female"      => "Female",
        _ => return None,
    })
}

// ─── Vietnamese dictionary ────────────────────────────────────────────────────

fn vi(en: &str) -> Option<&'static str> {
    Some(match en {
        "Loading..."                           => "Đang tải...",
        "Overview"                             => "Tổng quan",
        "Txns"                                 => "Giao dịch",
        "Wallets"                              => "Ví",
        "Family"                               => "Gia đình",
        "Reports"                              => "Báo cáo",
        "Settings"                             => "Cài đặt",
        "Language"                             => "Ngôn ngữ",
        "+ Add"                                => "+ Thêm",
        "Edit"                                 => "Sửa",
        "Delete"                               => "Xoá",
        "Cancel"                               => "Huỷ",
        "Save"                                 => "Lưu",
        "Update"                               => "Cập nhật",
        "Remove"                               => "Gỡ bỏ",
        "Copy"                                 => "Sao chép",
        "✓ Copied"                             => "✓ Đã chép",
        "Continue"                             => "Tiếp tục",
        "Verify"                               => "Xác nhận",
        "Unlock"                               => "Mở khoá",
        "Share"                                => "Chia sẻ",
        "Revert"                               => "Khôi phục",
        "Error"                                => "Lỗi",
        "All"                                  => "Tất cả",
        "Set your PIN"                         => "Tạo mã PIN",
        "You will need this PIN to unlock the app and access sensitive features" => "Bạn sẽ cần mã PIN này để mở khoá ứng dụng và truy cập các tính năng quan trọng",
        "Enter PIN"                            => "Nhập mã PIN",
        "Confirm PIN"                          => "Xác nhận mã PIN",
        "Enter your PIN"                       => "Nhập mã PIN của bạn",
        "Please enter your PIN to continue."   => "Vui lòng nhập mã PIN để tiếp tục.",
        "PIN must be at least 4 digits!"       => "Mã PIN phải có ít nhất 4 chữ số!",
        "PINs do not match!"                   => "Mã PIN không khớp!",
        "Incorrect PIN!"                       => "Sai mã PIN!",
        "Failed to set PIN"                    => "Không thể đặt mã PIN",
        "Cash"                                 => "Tiền mặt",
        "Bank"                                 => "Ngân hàng",
        "Loan"                                 => "Khoản vay",
        "E-Wallet"                             => "Ví điện tử",
        "Investment"                           => "Đầu tư",
        "Income"                               => "Thu nhập",
        "Expense"                              => "Chi tiêu",
        "Transfer"                             => "Chuyển khoản",
        "Buy"                                  => "Mua",
        "Sell"                                 => "Bán",
        "Salary"                               => "Lương",
        "Bonus"                                => "Thưởng",
        "Side Income"                          => "Thu nhập phụ",
        "Rental"                               => "Cho thuê",
        "Other"                                => "Khác",
        "Stock"                                => "Cổ phiếu",
        "Crypto"                               => "Tiền mã hoá",
        "Owner"                                => "Chủ hộ",
        "Member"                               => "Thành viên",
        "Male"                                 => "Nam",
        "Female"                               => "Nữ",
        "This Month"                           => "Tháng này",
        "Savings"                              => "Tiết kiệm",
        "Savings Rate"                         => "Tỷ lệ tiết kiệm",
        "⚠️ Overspending"                      => "⚠️ Chi vượt ngân sách",
        "Recent"                               => "Gần đây",
        "No transactions yet"                  => "Chưa có giao dịch nào",
        "Cash & Bank"                          => "Tiền mặt & Ngân hàng",
        "No wallets yet. Add one!"             => "Chưa có ví nào. Hãy thêm một ví!",
        "Balance"                              => "Số dư",
        "History"                              => "Lịch sử",
        "Holdings"                             => "Danh mục nắm giữ",
        "No transactions for this wallet"      => "Ví này chưa có giao dịch",
        "New Wallet"                           => "Ví mới",
        "Edit Wallet"                          => "Sửa ví",
        "Name"                                 => "Tên",
        "e.g. HSBC"                            => "VD: Vietcombank",
        "Type"                                 => "Loại",
        "Currency"                             => "Tiền tệ",
        "Opening Balance"                      => "Số dư ban đầu",
        "Broker"                               => "Công ty chứng khoán / Sàn",
        "e.g. HSBC, VPS, Binance, OKX"         => "VD: SSI, VPS, Binance, OKX",
        "Icon (emoji, optional)"               => "Biểu tượng (emoji, tuỳ chọn)",
        "Create Wallet"                        => "Tạo ví",
        "Update Wallet"                        => "Cập nhật ví",
        "Name is required"                     => "Vui lòng nhập tên",
        "Invalid balance"                      => "Số dư không hợp lệ",
        "Portfolio"                            => "Danh mục",
        "Cost Basis"                           => "Giá vốn",
        "Market Value"                         => "Giá trị thị trường",
        "Unrealized P&L"                       => "Lãi/lỗ chưa chốt",
        "Realized P&L"                         => "Lãi/lỗ đã chốt",
        "No holdings yet. Add a Buy transaction." => "Chưa có tài sản nào. Hãy thêm giao dịch Mua.",
        "Qty"                                  => "SL",
        "Quantity"                             => "Số lượng",
        "Avg Cost"                             => "Giá TB",
        "Last Price"                           => "Giá gần nhất",
        "Mkt Value"                            => "Giá trị TT",
        "New price"                            => "Giá mới",
        "Update last price"                    => "Cập nhật giá gần nhất",
        "Investment Overview"                  => "Tổng quan đầu tư",
        "No investment holdings yet"           => "Chưa có khoản đầu tư nào",
        "Transactions"                         => "Giao dịch",
        "Search by note…"                      => "Tìm theo ghi chú…",
        "No transactions found"                => "Không tìm thấy giao dịch",
        "Delete transaction?"                  => "Xoá giao dịch?",
        "This will reverse all balance changes." => "Mọi thay đổi số dư sẽ được hoàn lại.",
        "New Transaction"                      => "Giao dịch mới",
        "Edit Transaction"                     => "Sửa giao dịch",
        "From Wallet"                          => "Từ ví",
        "Select wallet"                        => "Chọn ví",
        "Amount"                               => "Số tiền",
        "Income Type"                          => "Loại thu nhập",
        "Select type"                          => "Chọn loại",
        "Category"                             => "Danh mục",
        "Select category"                      => "Chọn danh mục",
        "To Wallet"                            => "Đến ví",
        "Select destination wallet"            => "Chọn ví nhận",
        "To Amount (leave same for same-currency)" => "Số tiền nhận (để nguyên nếu cùng loại tiền)",
        "Received amount"                      => "Số tiền nhận",
        "To Currency"                          => "Loại tiền nhận",
        "Holding"                              => "Mã tài sản",
        "Select symbol"                        => "Chọn mã",
        "Price per unit"                       => "Giá mỗi đơn vị",
        "Date"                                 => "Ngày",
        "Note (optional)"                      => "Ghi chú (tuỳ chọn)",
        "Add a note…"                          => "Thêm ghi chú…",
        "Add Transaction"                      => "Thêm giao dịch",
        "Update Transaction"                   => "Cập nhật giao dịch",
        "Invalid amount"                       => "Số tiền không hợp lệ",
        "Invalid to_amount"                    => "Số tiền nhận không hợp lệ",
        "Invalid quantity"                     => "Số lượng không hợp lệ",
        "Invalid price"                        => "Giá không hợp lệ",
        "Timeline"                             => "Theo tháng",
        "Invest"                               => "Đầu tư",
        "Total Expense"                        => "Tổng chi tiêu",
        "No expenses this month"               => "Tháng này chưa có chi tiêu",
        "{pct}% of total"                      => "{pct}% tổng chi",
        "⚠️ Over budget"                       => "⚠️ Vượt ngân sách",
        "Budget usage: {pct}%"                 => "Đã dùng ngân sách: {pct}%",
        "Not enough data yet"                  => "Chưa đủ dữ liệu",
        "Month"                                => "Tháng",
        "Detail"                               => "Chi tiết",
        "Categories"                           => "Danh mục",
        "Backup"                               => "Sao lưu",
        "Custom"                               => "Tự tạo",
        "System (read-only)"                   => "Hệ thống (chỉ xem)",
        "Delete category?"                     => "Xoá danh mục?",
        "Categories in use by transactions cannot be deleted." => "Không thể xoá danh mục đang được giao dịch sử dụng.",
        "New Category"                         => "Danh mục mới",
        "Edit Category"                        => "Sửa danh mục",
        "Name required"                        => "Vui lòng nhập tên",
        "Invalid budget amount"                => "Ngân sách không hợp lệ",
        "e.g. Food & Drinks"                   => "VD: Ăn uống",
        "Icon"                                 => "Biểu tượng",
        "Color"                                => "Màu",
        "Monthly Budget (optional)"            => "Ngân sách hằng tháng (tuỳ chọn)",
        "Leave empty for unlimited"            => "Để trống nếu không giới hạn",
        "Create Category"                      => "Tạo danh mục",
        "Budget"                               => "Ngân sách",
        "No budget"                            => "Không giới hạn",
        "Version 1.0.0 · SQLite local storage" => "Phiên bản 1.0.0 · Lưu trữ cục bộ SQLite",
        "Export Backup"                        => "Xuất bản sao lưu",
        "Export all wallets, holdings, categories, transactions, members and families as a .bk file." => "Xuất toàn bộ ví, tài sản, danh mục, giao dịch, thành viên và gia đình ra tệp .bk.",
        "📤 Export .bk"                         => "📤 Xuất .bk",
        "Saved Backups"                        => "Bản sao lưu đã lưu",
        "Import from shared file"              => "Nhập từ tệp được chia sẻ",
        "⚠️ Import will REPLACE all existing data. This cannot be undone. Select a shared backup file to import it." => "⚠️ Nhập dữ liệu sẽ THAY THẾ toàn bộ dữ liệu hiện có và không thể hoàn tác. Chọn một tệp sao lưu để nhập.",
        "📥 Import File"                        => "📥 Nhập tệp",
        "❌ Failed to read file"                => "❌ Không đọc được tệp",
        "⚠️ Danger Zone"                       => "⚠️ Vùng nguy hiểm",
        "Permanently delete all data. Cannot be undone." => "Xoá vĩnh viễn toàn bộ dữ liệu. Không thể hoàn tác.",
        "Delete All Data"                      => "Xoá toàn bộ dữ liệu",
        "Exported to"                          => "Đã xuất ra",
        "Save failed"                          => "Lưu thất bại",
        "Export failed"                        => "Xuất thất bại",
        "Imported from"                        => "Đã nhập từ",
        "Imported from file"                   => "Đã nhập từ tệp",
        "Import failed"                        => "Nhập thất bại",
        "Sharing backup"                       => "Đang chia sẻ bản sao lưu",
        "Share failed"                         => "Chia sẻ thất bại",
        "Deleted backup"                       => "Đã xoá bản sao lưu",
        "Delete failed"                        => "Xoá thất bại",
        "Families & Members"                   => "Gia đình & Thành viên",
        "+ Add Family"                         => "+ Thêm gia đình",
        "+ Add Member"                         => "+ Thêm thành viên",
        "Unassigned Members"                   => "Thành viên chưa có gia đình",
        "OWNER"                                => "CHỦ HỘ",
        "👑 OWNER"                              => "👑 CHỦ HỘ",
        "👤 MEMBER"                             => "👤 THÀNH VIÊN",
        "No families yet"                      => "Chưa có gia đình nào",
        "Add a family and members to organize contacts!" => "Thêm gia đình và thành viên để quản lý danh bạ!",
        "Common Address"                       => "Địa chỉ chung",
        "Note"                                 => "Ghi chú",
        "Members"                              => "Thành viên",
        "Copy Address"                         => "Chép địa chỉ",
        "Add Family"                           => "Thêm gia đình",
        "Edit Family"                          => "Sửa gia đình",
        "Family Name"                          => "Tên gia đình",
        "Delete Family"                        => "Xoá gia đình",
        "New Member"                           => "Thành viên mới",
        "Edit Member"                          => "Sửa thành viên",
        "Full name is required"                => "Vui lòng nhập họ tên",
        "Avatar"                               => "Ảnh đại diện",
        "Full Name *"                          => "Họ và tên *",
        "Unassigned"                           => "Chưa có",
        "Role"                                 => "Vai trò",
        "Gender"                               => "Giới tính",
        "Phone"                                => "Số điện thoại",
        "Address"                              => "Địa chỉ",
        "Date of Birth"                        => "Ngày sinh",
        "💡 This is the address you will copy when ordering a ride to this person's house." => "💡 Đây là địa chỉ bạn sẽ sao chép khi đặt xe đến nhà người này.",
        "▼ Identity Document (CCCD/CMND)"      => "▼ Giấy tờ tuỳ thân (CCCD/CMND)",
        "▶ Identity Document (CCCD/CMND)"      => "▶ Giấy tờ tuỳ thân (CCCD/CMND)",
        "ID Number"                            => "Số CCCD/CMND",
        "ID Number (CCCD/CMND)"                => "Số CCCD/CMND",
        "ID Issue"                             => "Thông tin cấp",
        "Issue Date"                           => "Ngày cấp",
        "Issue Place"                          => "Nơi cấp",
        "Personal Info"                        => "Thông tin cá nhân",
        "Add Member"                           => "Thêm thành viên",
        "Update Member"                        => "Cập nhật thành viên",
        "Remove member?"                       => "Gỡ thành viên?",
        "\"{name}\" will be permanently removed from your family directory." => "\"{name}\" sẽ bị xoá vĩnh viễn khỏi danh bạ gia đình.",
        "Food & Drinks"                        => "Ăn uống",
        "Transport"                            => "Đi lại",
        "Shopping"                             => "Mua sắm",
        "Health"                               => "Sức khoẻ",
        "Entertainment"                        => "Giải trí",
        "Education"                            => "Giáo dục",
        "Bills & Utilities"                    => "Hoá đơn & Tiện ích",
        "Personal Care"                        => "Chăm sóc cá nhân",
        "Travel"                               => "Du lịch",
        _ => return None,
    })
}
