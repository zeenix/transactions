use serde::Deserialize;

// I went for one enum using internally-tagged enum representation as that seemed very natural to
// use for the problem at hand but after an hour of searching and trying out different things, I
// found out that internally-tagged enums are not supported by the `csv` crate. :( I've submitted
// a PR to document this limtation.
//
//  For details on why: https://github.com/BurntSushi/rust-csv/issues/211
//  PR: https://github.com/BurntSushi/rust-csv/pull/231

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TransactionType {
    /// A credit to the client account.
    Deposit,
    /// A debit to the client account.
    Withdrawal,
    /// A client's claim that a transaction was erroneous and should be reversed.
    Dispute,
    /// A resolution to a dispute, releasing the associated held funds.
    Resolve,
    /// A client reversing a disputed transaction.
    Chargeback,
}

/// An account transaction.
#[derive(Debug, Deserialize)]
pub(crate) struct Transaction {
    pub(crate) r#type: TransactionType,
    pub(crate) client: u16,
    pub(crate) tx: u32,
    pub(crate) amount: Option<f64>,
    #[serde(skip)]
    pub(crate) disputed: bool,
}
