use crate::dtos::{
    ApplyMissingInput, ExternalIdHit, MovementApply, NewCreation, NewFf3TransactionCreation,
    ReconciliationRun, RunFf3Transaction, RunMatch,
};
use crate::error::ApplicationError;
use crate::ports::{
    AppSettingsRepository, BatchRepository, CreationRepository, Ff3AccountRepository,
    Ff3TransactionRepository, MatchRepository, MovementRepository, ReconciliationRepository,
};
use ffm_domain::ids::{Ff3TransactionId, MovementId, ReconciliationId};
use ffm_domain::transaction_kind::{
    transaction_kind, AccountRef, BatchSide, Counterparty, Direction, TransactionKind,
};
use ffm_domain::value_objects::{BatchStatus, MovementStatus};
use std::collections::HashMap;

/// Outcome for each movement in the batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyOutcome {
    Created,
    AlreadyCreated,
    Failed,
}

fn not_recorded(ff3_id: i64, e: &ApplicationError) -> String {
    format!("created in Firefly III (id {ff3_id}) but not recorded: {e}; retry will reconcile it")
}

fn edits_not_applied(cust: &MovementApply) -> Option<String> {
    let edited = cust.attachment.is_some()
        || cust.description.is_some()
        || cust.category.is_some()
        || cust.budget.is_some()
        || cust.tags.is_some()
        || cust.notes.is_some();
    edited.then(|| {
        "already in Firefly III; the edits and attachment sent with this apply were not applied"
            .into()
    })
}

/// Per-movement report.
#[derive(Debug, Clone)]
pub struct ApplyMovementReport {
    pub movement_id: MovementId,
    pub outcome: ApplyOutcome,
    pub ff3_id: Option<i64>,
    pub reason: Option<String>,
    /// Type sent to Firefly III; `None` unless created in this run.
    pub transaction_type: Option<TransactionKind>,
}

/// Output of the apply-missing use case.
#[derive(Debug)]
pub struct ApplyOutput {
    pub reports: Vec<ApplyMovementReport>,
    pub created_count: usize,
    pub already_created_count: usize,
    pub failed_count: usize,
    pub reconciliation_id: Option<ReconciliationId>,
}

/// Ports needed for the incremental confirmation run.
pub struct RunPorts<R, M, T> {
    pub reconciliations: R,
    pub matches: M,
    pub ff3_transactions: T,
}

/// Use case: batch-create selected missing transactions in Firefly III.
pub struct ApplyMissing<Firefly, Accounts, Batches, Movements, Creations, R, M, T, Settings> {
    firefly: Firefly,
    accounts: Accounts,
    batches: Batches,
    movements: Movements,
    creations: Creations,
    run_ports: RunPorts<R, M, T>,
    settings: Settings,
}

impl<Firefly, Accounts, Batches, Movements, Creations, R, M, T, Settings>
    ApplyMissing<Firefly, Accounts, Batches, Movements, Creations, R, M, T, Settings>
{
    pub fn new(
        firefly: Firefly,
        accounts: Accounts,
        batches: Batches,
        movements: Movements,
        creations: Creations,
        run_ports: RunPorts<R, M, T>,
        settings: Settings,
    ) -> Self {
        Self {
            firefly,
            accounts,
            batches,
            movements,
            creations,
            run_ports,
            settings,
        }
    }
}

impl<Firefly, Accounts, Batches, Movements, Creations, R, M, T, Settings>
    ApplyMissing<Firefly, Accounts, Batches, Movements, Creations, R, M, T, Settings>
where
    Firefly: crate::ports::FireflyClient,
    Accounts: Ff3AccountRepository,
    Batches: BatchRepository,
    Movements: MovementRepository,
    Creations: CreationRepository,
    R: ReconciliationRepository,
    M: MatchRepository,
    T: Ff3TransactionRepository,
    Settings: AppSettingsRepository,
{
    /// The movement turns `matched` in `save_run`, atomically with its pair.
    async fn record_creation(
        &self,
        movement_id: MovementId,
        ff3_id: i64,
    ) -> Result<(), ApplicationError> {
        self.creations
            .insert(NewCreation {
                movement_id,
                ff3_id,
            })
            .await
            .map(|_| ())
    }

    pub async fn execute(&self, input: ApplyMissingInput) -> Result<ApplyOutput, ApplicationError> {
        // ── Load batch ────────────────────────────────────────────
        let batch = self.batches.get(input.batch_id).await?;
        if batch.status != BatchStatus::Reconciled {
            return Err(ApplicationError::InvalidState(format!(
                "batch {} is {:?}, not reconciled",
                input.batch_id.as_i64(),
                batch.status
            )));
        }
        let account_id = batch
            .account_id
            .ok_or_else(|| ApplicationError::InvalidState("batch has no account".into()))?;
        let account = self
            .accounts
            .by_id(account_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("account not found".into()))?;
        let account_ff3_id = account.ff3_id;

        // ── Load settings for default tag and origin trace ──────
        let settings = self.settings.get().await?;
        let default_tag = settings
            .as_ref()
            .and_then(|s| s.default_creation_tag.clone());
        let enable_origin_trace = settings
            .as_ref()
            .map(|s| s.enable_origin_trace)
            .unwrap_or(true); // Default: enabled

        // ── Load batch movements once ─────────────────────────────
        let batch_movements = self.movements.by_batch(input.batch_id).await?;

        // ── Validate every requested movement up-front ────────────
        let mut needs_firefly = false;
        for cust in &input.movements {
            let m = batch_movements
                .iter()
                .find(|m| m.id == cust.movement_id)
                .ok_or_else(|| {
                    ApplicationError::NotFound(format!(
                        "movement {} not in batch {}",
                        cust.movement_id.as_i64(),
                        input.batch_id.as_i64()
                    ))
                })?;
            let already = self.creations.by_movement(m.id).await?.is_some();
            if !already && m.status != MovementStatus::Missing {
                return Err(ApplicationError::InvalidState(format!(
                    "movement {} is {:?}, not missing",
                    m.id.as_i64(),
                    m.status
                )));
            }
            needs_firefly |= !already;
        }

        // ── Lines of this file already in Firefly III (NFR-05) ────
        // A creation whose `creations` row was lost is found by its
        // external_id; if the search fails, nothing is created.
        let external_prefix = format!(
            "ffm-{}-",
            batch.file_hash.get(..16).unwrap_or(&batch.file_hash)
        );
        let mut existing_by_line: HashMap<i32, ExternalIdHit> = HashMap::new();
        if needs_firefly {
            for hit in self
                .firefly
                .transactions_by_external_id_prefix(account_ff3_id, &external_prefix)
                .await?
            {
                if let Some(line) = hit
                    .external_id
                    .strip_prefix(&external_prefix)
                    .and_then(|l| l.parse().ok())
                {
                    existing_by_line.entry(line).or_insert(hit);
                }
            }
        }

        // ── Previous confirmation run (for cumulative counters) ───
        let previous_run = self
            .run_ports
            .reconciliations
            .latest_by_batch(input.batch_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::InvalidState(format!(
                    "batch {} has no reconciliation run",
                    input.batch_id.as_i64()
                ))
            })?;
        let previous_matches = self
            .run_ports
            .matches
            .by_reconciliation(previous_run.id)
            .await?;
        let prev_tx_rows = self
            .run_ports
            .ff3_transactions
            .by_reconciliation(previous_run.id)
            .await?;
        // Match rows reference the local cache row id (Ff3TransactionId);
        // RunMatch keys on the external ff3_id — remap via the cached rows.
        let local_to_ff3: HashMap<Ff3TransactionId, (i64, i64)> = prev_tx_rows
            .iter()
            .map(|t| (t.id, (t.ff3_id, t.ff3_journal_id)))
            .collect();
        let paired: Vec<(MovementId, i64)> = previous_matches
            .iter()
            .filter_map(|m| {
                local_to_ff3
                    .get(&m.ff3_transaction_id)
                    .map(|&(ff3_id, _)| (m.movement_id, ff3_id))
            })
            .collect();

        let mut reports = Vec::new();
        let mut created_ff3_ids: Vec<i64> = Vec::new();
        let mut created_counterparties: Vec<(Option<String>, Option<String>)> = Vec::new();
        let mut successful_movement_ids: Vec<MovementId> = Vec::new();

        for cust in input.movements {
            let m = batch_movements
                .iter()
                .find(|m| m.id == cust.movement_id)
                .expect("validated above");

            // ── Idempotency skip ──────────────────────────────────
            if let Some(existing) = self.creations.by_movement(m.id).await? {
                reports.push(ApplyMovementReport {
                    movement_id: m.id,
                    outcome: ApplyOutcome::AlreadyCreated,
                    ff3_id: Some(existing.ff3_id),
                    reason: None,
                    transaction_type: None,
                });
                continue;
            }

            // ── Pairs are UNIQUE per run on both sides ────────────
            let recovered = existing_by_line.get(&m.line_no);
            let recovered_id = recovered.map(|h| h.ff3_id);
            let conflict = match paired
                .iter()
                .find(|&&(pm, t)| pm == m.id && Some(t) != recovered_id)
            {
                Some(&(_, t)) => Some(format!(
                    "movement {} already matched to Firefly III tx {t}; re-run reconciliation",
                    m.id.as_i64()
                )),
                None => recovered_id.and_then(|id| {
                    paired
                        .iter()
                        .copied()
                        .chain(successful_movement_ids.iter().copied().zip(created_ff3_ids.iter().copied()))
                        .find(|&(owner, t)| t == id && owner != m.id)
                        .map(|(owner, t)| {
                            format!(
                                "Firefly III tx {t} already matched to movement {}; re-run reconciliation",
                                owner.as_i64()
                            )
                        })
                }),
            };
            if let Some(reason) = conflict {
                reports.push(ApplyMovementReport {
                    movement_id: m.id,
                    outcome: ApplyOutcome::Failed,
                    ff3_id: recovered_id,
                    reason: Some(reason),
                    transaction_type: None,
                });
                continue;
            }

            // ── Created by an earlier run but never recorded ──────
            if let Some(hit) = recovered {
                let (outcome, reason) = match self.record_creation(m.id, hit.ff3_id).await {
                    Ok(()) => {
                        created_ff3_ids.push(hit.ff3_id);
                        created_counterparties
                            .push((hit.counterparty_name.clone(), hit.counterparty_type.clone()));
                        successful_movement_ids.push(m.id);
                        (ApplyOutcome::AlreadyCreated, edits_not_applied(&cust))
                    }
                    Err(e) => (ApplyOutcome::Failed, Some(not_recorded(hit.ff3_id, &e))),
                };
                reports.push(ApplyMovementReport {
                    movement_id: m.id,
                    outcome,
                    ff3_id: Some(hit.ff3_id),
                    reason,
                    transaction_type: None,
                });
                continue;
            }

            // ── Zero-amount guard ─────────────────────────────────
            if m.amount_cents.as_i64() == 0 {
                reports.push(ApplyMovementReport {
                    movement_id: m.id,
                    outcome: ApplyOutcome::Failed,
                    ff3_id: None,
                    reason: Some("amount is zero".into()),
                    transaction_type: None,
                });
                continue;
            }

            // ── Type from sign + counterparty (RF-08 v0.10) ───────
            let new_name = cust
                .counterparty_name
                .as_deref()
                .map(str::trim)
                .filter(|n| !n.is_empty());
            let cached = match (cust.counterparty_ff3_id, new_name) {
                (Some(id), None) => self.accounts.by_ff3_id(id).await?,
                _ => None,
            };
            let counterparty = match (cust.counterparty_ff3_id, new_name, &cached) {
                (Some(_), Some(_), _) => Ok(Counterparty::IdAndName),
                (Some(id), None, None) => Err(format!(
                    "counterparty {id} is not a known account; reload the account list"
                )),
                (Some(_), None, Some(a)) => Ok(Counterparty::Existing(AccountRef {
                    ff3_id: a.ff3_id,
                    account_type: &a.account_type,
                    currency: &a.currency,
                })),
                (None, Some(_), _) => Ok(Counterparty::NewName),
                (None, None, _) => Ok(Counterparty::Cash),
            };
            let batch_ref = AccountRef {
                ff3_id: account_ff3_id,
                account_type: &account.account_type,
                currency: &account.currency,
            };
            let resolved = counterparty.and_then(|c| {
                transaction_kind(Direction::from_cents(m.amount_cents.as_i64()), batch_ref, c)
                    .map_err(|e| e.to_string())
            });
            let (kind, batch_side) = match resolved {
                Ok(r) => r,
                Err(reason) => {
                    reports.push(ApplyMovementReport {
                        movement_id: m.id,
                        outcome: ApplyOutcome::Failed,
                        ff3_id: None,
                        reason: Some(reason),
                        transaction_type: None,
                    });
                    continue;
                }
            };
            let name = new_name.map(str::to_string);
            let (source, destination, source_name, destination_name) = match batch_side {
                BatchSide::Source => (Some(account_ff3_id), cust.counterparty_ff3_id, None, name),
                BatchSide::Destination => {
                    (cust.counterparty_ff3_id, Some(account_ff3_id), name, None)
                }
            };

            // ── Description: override > statement ─────────────────
            let description = cust
                .description
                .as_ref()
                .map(|d| d.trim())
                .filter(|d| !d.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| m.description.clone());

            // ── Tags: user selection (even empty) > default tag ───────
            let tags = cust
                .tags
                .clone()
                .unwrap_or_else(|| default_tag.clone().into_iter().collect());

            // ── Notes: user note + origin trace (RF-11, configurable) ─────
            let origin_note = if enable_origin_trace {
                format!("ffm:{}:{}", batch.filename, input.batch_id.as_i64())
            } else {
                String::new()
            };
            let notes = match (&cust.notes, &origin_note[..]) {
                (Some(user_note), "") => user_note.clone(),
                (None, origin_trace) => origin_trace.to_string(),
                (Some(user_note), origin_trace) => format!("{}\n\n{}", user_note, origin_trace),
            };

            let creation = NewFf3TransactionCreation {
                transaction_type: kind.as_str().to_string(),
                amount: m.amount_cents,
                date: m.date,
                description,
                source_ff3_id: source,
                destination_ff3_id: destination,
                source_name,
                destination_name,
                category: cust.category.clone().filter(|c| !c.trim().is_empty()),
                budget: cust.budget.clone().filter(|b| !b.trim().is_empty()),
                notes: if notes.is_empty() { None } else { Some(notes) },
                tags,
                external_id: Some(format!("{external_prefix}{}", m.line_no)),
            };

            match self.firefly.create_transaction(creation).await {
                Ok(created) => {
                    let ff3_id = created.ff3_id;
                    if let Err(e) = self.record_creation(m.id, ff3_id).await {
                        reports.push(ApplyMovementReport {
                            movement_id: m.id,
                            outcome: ApplyOutcome::Failed,
                            ff3_id: Some(ff3_id),
                            reason: Some(not_recorded(ff3_id, &e)),
                            transaction_type: None,
                        });
                        continue;
                    }
                    // The counterparty is the side opposite to the batch account.
                    created_counterparties.push(match batch_side {
                        BatchSide::Source => (created.destination_name, created.destination_type),
                        BatchSide::Destination => (created.source_name, created.source_type),
                    });
                    created_ff3_ids.push(ff3_id);
                    successful_movement_ids.push(m.id);

                    // ── Attachment (non-blocking on failure) ───────
                    let mut reason = None;
                    if let Some(att) = &cust.attachment {
                        if let Err(e) = self
                            .firefly
                            .attach_to_transaction(ff3_id, &att.filename, &att.content_base64)
                            .await
                        {
                            reason = Some(format!("attachment failed: {e}"));
                        }
                    }

                    reports.push(ApplyMovementReport {
                        movement_id: m.id,
                        outcome: ApplyOutcome::Created,
                        ff3_id: Some(ff3_id),
                        reason,
                        transaction_type: Some(kind),
                    });
                }
                Err(e) => {
                    reports.push(ApplyMovementReport {
                        movement_id: m.id,
                        outcome: ApplyOutcome::Failed,
                        ff3_id: None,
                        reason: Some(format!("FF3 error: {e}")),
                        transaction_type: None,
                    });
                }
            }
        }

        // ── Incremental confirmation run (D7) ─────────────────────
        let recon_id = if !successful_movement_ids.is_empty() {
            let all_cached_txs: Vec<RunFf3Transaction> = prev_tx_rows
                .iter()
                .map(|t| RunFf3Transaction {
                    ff3_id: t.ff3_id,
                    ff3_journal_id: t.ff3_journal_id,
                    date: t.date,
                    amount_cents: t.amount_cents,
                    description: t.description.clone(),
                    is_duplicate: t.is_duplicate,
                    counterparty_name: t.counterparty_name.clone(),
                    counterparty_type: t.counterparty_type.clone(),
                })
                .collect();

            // Build the new tx cache: duplicates + newly created + previous unmatched extras.
            let mut new_cached: Vec<RunFf3Transaction> = all_cached_txs
                .iter()
                .filter(|t| t.is_duplicate)
                .cloned()
                .collect();

            // Add newly created transactions (build from successful movements + created FF3 ids).
            for ((movement, ff3_id), (cp_name, cp_type)) in successful_movement_ids
                .iter()
                .zip(created_ff3_ids.iter())
                .zip(created_counterparties.iter())
            {
                let stmt = batch_movements.iter().find(|s| s.id == *movement).unwrap();
                new_cached.push(RunFf3Transaction {
                    ff3_id: *ff3_id,
                    ff3_journal_id: 0,
                    date: stmt.date,
                    amount_cents: stmt.amount_cents,
                    description: stmt.description.clone(),
                    is_duplicate: false,
                    counterparty_name: cp_name.clone(),
                    counterparty_type: cp_type.clone(),
                });
            }

            // Add all previous non-duplicate, non-newly-created txs (matched + unmatched extras).
            for tx in &all_cached_txs {
                if !tx.is_duplicate && !created_ff3_ids.contains(&tx.ff3_id) {
                    new_cached.push(tx.clone());
                }
            }

            // New matches: each successful movement matched to its created FF3 tx.
            let new_match_dtos: Vec<RunMatch> = successful_movement_ids
                .iter()
                .zip(created_ff3_ids.iter())
                .map(|(mid, &ff3_id)| RunMatch {
                    movement_id: *mid,
                    ff3_id,
                    ff3_journal_id: 0,
                })
                .collect();

            // Carry over the previous run's match pairs. Their FF3 transactions
            // are re-cached below; without the carry-over, get_results derives
            // groups from match rows and would demote every old pair to `extra`.
            let mut all_match_dtos: Vec<RunMatch> = previous_matches
                .iter()
                .filter_map(|m| {
                    local_to_ff3
                        .get(&m.ff3_transaction_id)
                        .map(|&(ff3_id, ff3_journal_id)| RunMatch {
                            movement_id: m.movement_id,
                            ff3_id,
                            ff3_journal_id,
                        })
                })
                .collect();
            // A recovered line may already hold this exact pair in the previous
            // run; one-sided conflicts were rejected per movement above.
            let carried = all_match_dtos.len();
            all_match_dtos.retain(|c| {
                !new_match_dtos
                    .iter()
                    .any(|n| n.movement_id == c.movement_id && n.ff3_id == c.ff3_id)
            });
            let newly_matched = new_match_dtos.len() - (carried - all_match_dtos.len());
            all_match_dtos.extend(new_match_dtos.iter().cloned());
            // Recovered txs the previous run listed as unmatched extras.
            let recovered_extras = created_ff3_ids
                .iter()
                .filter(|&&id| {
                    prev_tx_rows
                        .iter()
                        .any(|t| t.ff3_id == id && !t.is_duplicate)
                        && !paired.iter().any(|&(_, t)| t == id)
                })
                .count();

            let prev_matched = previous_run.matched_count as usize;
            let prev_missing = previous_run.missing_count as usize;
            let prev_extra = previous_run.extra_count as usize;

            let matched = prev_matched + newly_matched;
            let missing = prev_missing.saturating_sub(newly_matched);
            let extra = prev_extra.saturating_sub(recovered_extras);

            let run = ReconciliationRun {
                reconciliation: crate::dtos::NewReconciliation {
                    batch_id: input.batch_id,
                    tolerance_days: previous_run.tolerance_days,
                    window_start: previous_run.window_start,
                    window_end: previous_run.window_end,
                    matched_count: matched as i32,
                    missing_count: missing as i32,
                    extra_count: extra as i32,
                },
                ff3_transactions: new_cached,
                matches: all_match_dtos,
                movement_status_updates: successful_movement_ids
                    .iter()
                    .map(|mid| (*mid, MovementStatus::Matched))
                    .collect(),
            };

            Some(self.run_ports.reconciliations.save_run(run).await?)
        } else {
            None
        };

        let created_count = reports
            .iter()
            .filter(|r| r.outcome == ApplyOutcome::Created)
            .count();
        let already_count = reports
            .iter()
            .filter(|r| r.outcome == ApplyOutcome::AlreadyCreated)
            .count();
        let failed_count = reports
            .iter()
            .filter(|r| r.outcome == ApplyOutcome::Failed)
            .count();

        Ok(ApplyOutput {
            reports,
            created_count,
            already_created_count: already_count,
            failed_count,
            reconciliation_id: recon_id,
        })
    }
}
