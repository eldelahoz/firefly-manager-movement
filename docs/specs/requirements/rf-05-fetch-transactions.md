# RF-05 - Target Account Transaction Fetching

| Field | Value |
| --- | --- |
| Priority | P0 |
| Use cases | CU-3 |
| Depends on | RF-04 (target account), RF-19 (FF3 connection) |
| Related | RF-06 (matching engine), RF-14 (browsing), NFR-09 |

## Description

The system MUST fetch the target account's transactions from the Firefly III API over the reconciliation **date window** - the statement's date range extended by the configurable search window (default ±7 days, PRD section 4). Fetching MUST use **exhaustive pagination**: every page of the FF3 response is consumed until the dataset is complete. Fetched transactions are cached per reconciliation run.

## Preconditions

1. A batch with movements exists and a target asset account is selected (RF-04).
2. The FF3 API is reachable.

## Steps

1. **Given** a reconciliation is launched on a batch,
2. **When** the engine needs the FF3 side of the comparison,
3. **Then** the system retrieves the account's transactions from Firefly III for the window `[min_date − window, max_date + window]`, consuming every page of results until the dataset is complete.

## Expected result

- The complete set of the account's transactions over the window is available to the matching engine (RF-06) and persisted with the reconciliation run.

## Exceptions

- **FF3 API error or timeout**: the reconciliation is aborted with a clear error and a retry option (NFR-09); no partial results are persisted as a run.
- **Very long windows**: handled by exhaustive pagination by design; the window remains bounded by the statement dates ± window (risk table, PRD section 9).

## Acceptance criteria

- [ ] Transactions are retrieved over the computed window
- [ ] Pagination is exhaustive
- [ ] The same account data is available for browsing by period (RF-14)
- [ ] The end-to-end flow batch → reconcile → transactions fetched works

## Notes

- How the fetch is transported and paginated against the FF3 API is specified in the TRD (section 3).
- Fetched transactions are cached per reconciliation run (TRD section 2).
