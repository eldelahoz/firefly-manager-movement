//! Transaction type from the counterparty (RF-08 v0.10), following Firefly III's
//! `account_to_transaction` rules. The batch account is always an own asset.

use std::fmt;

/// Sense of a statement movement: negative amount = money out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Out,
    In,
}

impl Direction {
    pub fn from_cents(amount_cents: i64) -> Self {
        if amount_cents < 0 {
            Direction::Out
        } else {
            Direction::In
        }
    }
}

/// An existing Firefly III account, as known by the account cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountRef<'a> {
    pub ff3_id: i64,
    /// Firefly III short type: `asset`, `expense`, `revenue`, `liabilities`.
    pub account_type: &'a str,
    pub currency: &'a str,
}

/// The counterparty the user chose for a movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counterparty<'a> {
    /// Nothing chosen: Firefly III books it against its Cash account.
    Cash,
    /// A new expense (money out) or revenue (money in) account, created by name.
    NewName,
    Existing(AccountRef<'a>),
    /// Both an id and a new name were sent.
    IdAndName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionKind {
    Withdrawal,
    Deposit,
    Transfer,
}

impl TransactionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionKind::Withdrawal => "withdrawal",
            TransactionKind::Deposit => "deposit",
            TransactionKind::Transfer => "transfer",
        }
    }
}

/// Which side of the Firefly III transaction the batch account is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchSide {
    Source,
    Destination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CounterpartyError {
    SameAccount,
    ExpenseOnMoneyIn,
    RevenueOnMoneyOut,
    CurrencyMismatch { batch: String, counterparty: String },
    IdAndName,
    UnsupportedType(String),
}

impl fmt::Display for CounterpartyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SameAccount => write!(f, "counterparty is the batch account itself"),
            Self::ExpenseOnMoneyIn => {
                write!(f, "an expense account cannot be the source of money in")
            }
            Self::RevenueOnMoneyOut => write!(
                f,
                "a revenue account cannot be the destination of money out"
            ),
            Self::CurrencyMismatch {
                batch,
                counterparty,
            } => write!(
                f,
                "transfer across currencies is not supported ({batch} → {counterparty})"
            ),
            Self::IdAndName => write!(
                f,
                "send either counterparty_ff3_id or counterparty_name, not both"
            ),
            Self::UnsupportedType(t) => write!(f, "unsupported counterparty account type: {t}"),
        }
    }
}

impl std::error::Error for CounterpartyError {}

/// Derives the Firefly III transaction type for a movement on `batch_account`.
pub fn transaction_kind(
    direction: Direction,
    batch_account: AccountRef<'_>,
    counterparty: Counterparty<'_>,
) -> Result<(TransactionKind, BatchSide), CounterpartyError> {
    use TransactionKind::{Deposit, Transfer, Withdrawal};
    let side = match direction {
        Direction::Out => BatchSide::Source,
        Direction::In => BatchSide::Destination,
    };
    let by_direction = match direction {
        Direction::Out => Withdrawal,
        Direction::In => Deposit,
    };
    let kind = match counterparty {
        Counterparty::IdAndName => return Err(CounterpartyError::IdAndName),
        Counterparty::Cash | Counterparty::NewName => by_direction,
        Counterparty::Existing(cp) if cp.ff3_id == batch_account.ff3_id => {
            return Err(CounterpartyError::SameAccount)
        }
        Counterparty::Existing(cp) => match (cp.account_type, direction) {
            ("expense", Direction::Out) | ("revenue", Direction::In) => by_direction,
            ("expense", Direction::In) => return Err(CounterpartyError::ExpenseOnMoneyIn),
            ("revenue", Direction::Out) => return Err(CounterpartyError::RevenueOnMoneyOut),
            // Firefly III reads liabilities as `liabilities`; `liability` is its filter alias.
            ("liabilities" | "liability", _) => by_direction,
            ("asset", _) if cp.currency != batch_account.currency => {
                return Err(CounterpartyError::CurrencyMismatch {
                    batch: batch_account.currency.to_string(),
                    counterparty: cp.currency.to_string(),
                })
            }
            ("asset", _) => Transfer,
            (other, _) => return Err(CounterpartyError::UnsupportedType(other.to_string())),
        },
    };
    Ok((kind, side))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BATCH: AccountRef<'static> = AccountRef {
        ff3_id: 1,
        account_type: "asset",
        currency: "COP",
    };

    fn account(ff3_id: i64, account_type: &'static str) -> Counterparty<'static> {
        Counterparty::Existing(AccountRef {
            ff3_id,
            account_type,
            currency: "COP",
        })
    }

    fn kind(
        d: Direction,
        c: Counterparty<'_>,
    ) -> Result<(TransactionKind, BatchSide), CounterpartyError> {
        transaction_kind(d, BATCH, c)
    }

    use BatchSide::{Destination, Source};
    use Direction::{In, Out};
    use TransactionKind::{Deposit, Transfer, Withdrawal};

    #[test]
    fn direction_from_sign() {
        assert_eq!(Direction::from_cents(-1), Out);
        assert_eq!(Direction::from_cents(1), In);
    }

    #[test]
    fn expense_on_money_out_is_withdrawal() {
        assert_eq!(kind(Out, account(5, "expense")), Ok((Withdrawal, Source)));
    }

    #[test]
    fn expense_on_money_in_is_rejected() {
        assert_eq!(
            kind(In, account(5, "expense")),
            Err(CounterpartyError::ExpenseOnMoneyIn)
        );
    }

    #[test]
    fn revenue_on_money_in_is_deposit() {
        assert_eq!(kind(In, account(6, "revenue")), Ok((Deposit, Destination)));
    }

    #[test]
    fn revenue_on_money_out_is_rejected() {
        assert_eq!(
            kind(Out, account(6, "revenue")),
            Err(CounterpartyError::RevenueOnMoneyOut)
        );
    }

    #[test]
    fn liability_on_money_out_is_withdrawal() {
        assert_eq!(
            kind(Out, account(7, "liabilities")),
            Ok((Withdrawal, Source))
        );
    }

    #[test]
    fn liability_on_money_in_is_deposit() {
        assert_eq!(
            kind(In, account(7, "liabilities")),
            Ok((Deposit, Destination))
        );
    }

    #[test]
    fn own_asset_on_money_out_is_transfer() {
        assert_eq!(kind(Out, account(4, "asset")), Ok((Transfer, Source)));
    }

    #[test]
    fn own_asset_on_money_in_is_transfer() {
        assert_eq!(kind(In, account(4, "asset")), Ok((Transfer, Destination)));
    }

    #[test]
    fn own_asset_in_another_currency_is_rejected() {
        let usd = Counterparty::Existing(AccountRef {
            ff3_id: 4,
            account_type: "asset",
            currency: "USD",
        });
        assert_eq!(
            kind(Out, usd),
            Err(CounterpartyError::CurrencyMismatch {
                batch: "COP".into(),
                counterparty: "USD".into()
            })
        );
    }

    #[test]
    fn batch_account_itself_is_rejected() {
        assert_eq!(
            kind(Out, account(1, "asset")),
            Err(CounterpartyError::SameAccount)
        );
    }

    #[test]
    fn new_name_on_money_out_is_withdrawal() {
        assert_eq!(kind(Out, Counterparty::NewName), Ok((Withdrawal, Source)));
    }

    #[test]
    fn new_name_on_money_in_is_deposit() {
        assert_eq!(kind(In, Counterparty::NewName), Ok((Deposit, Destination)));
    }

    #[test]
    fn cash_on_money_out_is_withdrawal() {
        assert_eq!(kind(Out, Counterparty::Cash), Ok((Withdrawal, Source)));
    }

    #[test]
    fn cash_on_money_in_is_deposit() {
        assert_eq!(kind(In, Counterparty::Cash), Ok((Deposit, Destination)));
    }

    #[test]
    fn id_and_name_is_rejected() {
        assert_eq!(
            kind(Out, Counterparty::IdAndName),
            Err(CounterpartyError::IdAndName)
        );
    }

    #[test]
    fn other_account_types_are_rejected() {
        assert_eq!(
            kind(Out, account(9, "initial-balance")),
            Err(CounterpartyError::UnsupportedType("initial-balance".into()))
        );
    }
}
