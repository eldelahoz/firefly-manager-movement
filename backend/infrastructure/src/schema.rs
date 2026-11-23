// @generated automatically by Diesel CLI.

diesel::table! {
    app_settings (id) {
        id -> Integer,
        ff3_url -> Text,
        ff3_token -> Text,
        default_creation_tag -> Nullable<Text>,
        updated_at -> Timestamp,
        enable_origin_trace -> Bool,
        default_date_format -> Nullable<Text>,
    }
}

diesel::table! {
    batches (id) {
        id -> Bigint,
        filename -> Text,
        file_hash -> Text,
        format -> Text,
        currency -> Text,
        account_id -> Nullable<Bigint>,
        status -> Text,
        created_at -> Timestamp,
        headers_json -> Json,
        detected_delimiter -> Text,
    }
}

diesel::table! {
    counterparty_memory (id) {
        id -> Bigint,
        account_ff3_id -> Bigint,
        direction -> Text,
        desc_key -> Text,
        counterparty_ff3_id -> Nullable<Bigint>,
        counterparty_name -> Nullable<Text>,
        is_cash -> Bool,
        uses -> Bigint,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    creations (id) {
        id -> Bigint,
        movement_id -> Bigint,
        ff3_id -> Bigint,
        created_at -> Timestamp,
    }
}

diesel::table! {
    csv_mappings (id) {
        id -> Bigint,
        header_signature -> Text,
        mapping_json -> Json,
        created_at -> Timestamp,
    }
}

diesel::table! {
    deleted_audit (id) {
        id -> Bigint,
        ff3_id -> Bigint,
        snapshot_json -> Json,
        deleted_at -> Timestamp,
    }
}

diesel::table! {
    ff3_accounts (id) {
        id -> Bigint,
        ff3_id -> Integer,
        name -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        currency -> Text,
        iban -> Nullable<Text>,
        account_number -> Nullable<Text>,
        synced_at -> Timestamp,
    }
}

diesel::table! {
    ff3_transactions (id) {
        id -> Bigint,
        reconciliation_id -> Bigint,
        ff3_id -> Bigint,
        ff3_journal_id -> Bigint,
        date -> Date,
        amount_cents -> Bigint,
        description -> Text,
        is_duplicate -> Bool,
        created_at -> Timestamp,
        counterparty_name -> Nullable<Text>,
        counterparty_type -> Nullable<Text>,
    }
}

diesel::table! {
    matches (id) {
        id -> Bigint,
        reconciliation_id -> Bigint,
        movement_id -> Bigint,
        ff3_transaction_id -> Bigint,
        matched_at -> Timestamp,
    }
}

diesel::table! {
    movements (id) {
        id -> Bigint,
        batch_id -> Bigint,
        line_no -> Integer,
        date -> Date,
        amount_cents -> Bigint,
        description -> Text,
        raw_json -> Json,
        status -> Text,
        created_at -> Timestamp,
        history_ff3_id -> Nullable<Bigint>,
        history_name -> Nullable<Text>,
        history_cash -> Nullable<Bool>,
    }
}

diesel::table! {
    reconciliations (id) {
        id -> Bigint,
        batch_id -> Bigint,
        tolerance_days -> Integer,
        window_start -> Date,
        window_end -> Date,
        matched_count -> Integer,
        missing_count -> Integer,
        extra_count -> Integer,
        ran_at -> Timestamp,
    }
}

diesel::joinable!(batches -> ff3_accounts (account_id));
diesel::joinable!(creations -> movements (movement_id));
diesel::joinable!(ff3_transactions -> reconciliations (reconciliation_id));
diesel::joinable!(matches -> ff3_transactions (ff3_transaction_id));
diesel::joinable!(matches -> movements (movement_id));
diesel::joinable!(matches -> reconciliations (reconciliation_id));
diesel::joinable!(movements -> batches (batch_id));
diesel::joinable!(reconciliations -> batches (batch_id));

diesel::allow_tables_to_appear_in_same_query!(
    app_settings,
    batches,
    counterparty_memory,
    creations,
    csv_mappings,
    deleted_audit,
    ff3_accounts,
    ff3_transactions,
    matches,
    movements,
    reconciliations,
);
