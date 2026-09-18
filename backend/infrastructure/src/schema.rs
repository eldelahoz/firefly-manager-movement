// @generated automatically by Diesel CLI.

diesel::table! {
    app_settings (id) {
        id -> Integer,
        #[max_length = 255]
        ff3_url -> Varchar,
        ff3_token -> Text,
        #[max_length = 255]
        default_creation_tag -> Nullable<Varchar>,
        updated_at -> Datetime,
        enable_origin_trace -> Bool,
        #[max_length = 16]
        default_date_format -> Nullable<Varchar>,
    }
}

diesel::table! {
    batches (id) {
        id -> Bigint,
        #[max_length = 255]
        filename -> Varchar,
        #[max_length = 255]
        file_hash -> Varchar,
        #[max_length = 50]
        format -> Varchar,
        #[max_length = 3]
        currency -> Char,
        account_id -> Nullable<Bigint>,
        #[max_length = 20]
        status -> Varchar,
        created_at -> Datetime,
        headers_json -> Json,
        #[max_length = 8]
        detected_delimiter -> Varchar,
    }
}

diesel::table! {
    creations (id) {
        id -> Bigint,
        movement_id -> Bigint,
        ff3_id -> Bigint,
        created_at -> Datetime,
    }
}

diesel::table! {
    csv_mappings (id) {
        id -> Bigint,
        #[max_length = 255]
        header_signature -> Varchar,
        mapping_json -> Json,
        created_at -> Datetime,
    }
}

diesel::table! {
    deleted_audit (id) {
        id -> Bigint,
        ff3_id -> Bigint,
        snapshot_json -> Json,
        deleted_at -> Datetime,
    }
}

diesel::table! {
    ff3_accounts (id) {
        id -> Bigint,
        ff3_id -> Integer,
        #[max_length = 255]
        name -> Varchar,
        #[sql_name = "type"]
        #[max_length = 50]
        type_ -> Varchar,
        #[max_length = 3]
        currency -> Char,
        #[max_length = 64]
        iban -> Nullable<Varchar>,
        #[max_length = 255]
        account_number -> Nullable<Varchar>,
        synced_at -> Datetime,
    }
}

diesel::table! {
    ff3_transactions (id) {
        id -> Bigint,
        reconciliation_id -> Bigint,
        ff3_id -> Bigint,
        date -> Date,
        amount_cents -> Bigint,
        description -> Text,
        is_duplicate -> Bool,
        created_at -> Datetime,
    }
}

diesel::table! {
    matches (id) {
        id -> Bigint,
        reconciliation_id -> Bigint,
        movement_id -> Bigint,
        ff3_transaction_id -> Bigint,
        matched_at -> Datetime,
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
        #[max_length = 20]
        status -> Varchar,
        created_at -> Datetime,
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
        ran_at -> Datetime,
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
    creations,
    csv_mappings,
    deleted_audit,
    ff3_accounts,
    ff3_transactions,
    matches,
    movements,
    reconciliations,
);
