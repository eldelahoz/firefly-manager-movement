# RF-14 - Account Transaction Browsing

| Field | Value |
| --- | --- |
| Version | v0.2 (original 2026-08-14 + counterparty amendment) |
| Priority | P0 |
| Use cases | CU-7 |
| Depends on | RF-04 (account list), RF-19 (FF3 connection) |
| Related | RF-05 (same fetch path), RF-15 (spotting duplicates), NFR-09 |

## Description

The system MUST display the transactions of a **selected Firefly III account over a given period** (min date / max date), independently of any import batch. The view serves to inspect an account and to spot duplicates outside a reconciliation (CU-7/CU-8).

The read is performed through the app's backend - so FF3 credentials never reach the browser (NFR-02) - with exhaustive pagination.

## Preconditions

1. The FF3 connection is configured and reachable.
2. The user picked an account and a valid period (start ≤ end).

## Steps

1. **Given** the account browsing widget,
2. **When** the user selects an account and a period (min/max date),
3. **Then** the system displays the account's transactions over the period with their details (date, amount, description, counterparty).

**Counterparty (v0.2)**: each transaction also shows the **counterparty** - the name and the type of the account on the opposite side of the browsed account (`Expense account`, `Revenue account`, `Asset account`, `Cash account`, `Loan`, `Debt`, `Mortgage`), as returned by Firefly III. The type is shown as a badge. When Firefly III does not report it, the cell is empty.

## Expected result

- The transaction list matches exactly what FF3 holds for that account and period.

## Exceptions

- **Invalid period** (start > end): the request is refused with a clear validation error.
- **FF3 unavailable**: clear error message with retry (NFR-09).

## Acceptance criteria

- [ ] The system retrieves transactions bounded by the selected period
- [ ] Pagination is exhaustive over the period
- [ ] The widget renders date pickers, account selector and transaction table
- [ ] (v0.2) Each transaction carries `counterparty_name` and `counterparty_type`, taken from the side opposite to the browsed account (source when the account is the destination, destination when it is the source)
- [ ] (v0.2) The table shows the counterparty name and a type badge; a transaction without counterparty data shows an empty cell

## Notes

- v0.2 amendment (counterparty accounts, phase A, RF-08 v0.10): shows where each transaction went or came from. Read-only; no new request, since Firefly III already returns the names and types.
