table! {
    groups (id) {
        id -> Bigint,
        uuid -> Varchar,
        name -> Varchar,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    newsletter_subscribers (id) {
        id -> Bigint,
        email -> Varchar,
        active -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    request_network_hashes (id) {
        id -> Bigint,
        payment_id -> Bigint,
        hash -> Varchar,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    request_network_payments (id) {
        id -> Bigint,
        uuid -> Varchar,
        user_id -> Bigint,
        serie_id -> Bigint,
        amount_in_eth -> Varchar,
        to_address -> Varchar,
        reason -> Varchar,
        used -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    series (id) {
        id -> Bigint,
        uuid -> Varchar,
        title -> Varchar,
        slug -> Varchar,
        description -> Text,
        price -> Integer,
        published -> Bool,
        archived -> Bool,
        in_development -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Bigint,
        username -> Varchar,
        password -> Varchar,
        email -> Varchar,
        verified -> Bool,
        updated -> Timestamp,
        created -> Timestamp,
    }
}

table! {
    users_group (id) {
        id -> Bigint,
        user_id -> Bigint,
        group_id -> Bigint,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    users_recover_email (id) {
        id -> Bigint,
        user_id -> Bigint,
        token -> Varchar,
        used -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    users_series_access (id) {
        id -> Bigint,
        user_id -> Bigint,
        series_id -> Bigint,
        bought -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    users_sessions (id) {
        id -> Bigint,
        user_id -> Bigint,
        token -> Varchar,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    users_stripe_card (id) {
        id -> Bigint,
        user_id -> Bigint,
        address_city -> Nullable<Varchar>,
        address_country -> Nullable<Varchar>,
        address_line1 -> Nullable<Varchar>,
        address_line1_check -> Nullable<Varchar>,
        address_line2 -> Nullable<Varchar>,
        address_state -> Nullable<Varchar>,
        address_zip -> Nullable<Varchar>,
        address_zip_check -> Nullable<Varchar>,
        brand -> Varchar,
        country -> Varchar,
        cvc_check -> Nullable<Varchar>,
        dynamic_last4 -> Nullable<Varchar>,
        exp_month -> Integer,
        exp_year -> Integer,
        funding -> Nullable<Varchar>,
        card_id -> Nullable<Varchar>,
        last4 -> Varchar,
        metadata -> Nullable<Varchar>,
        name -> Nullable<Varchar>,
        object -> Nullable<Varchar>,
        tokenization_method -> Nullable<Varchar>,
    }
}

table! {
    users_stripe_charge (id) {
        id -> Bigint,
        user_id -> Bigint,
        series_id -> Bigint,
        uuid -> Varchar,
        amount -> Integer,
        amount_refunded -> Integer,
        balance_transaction -> Nullable<Varchar>,
        captured -> Bool,
        created_at_stripe -> Bigint,
        description -> Nullable<Text>,
        destination -> Nullable<Varchar>,
        dispute -> Nullable<Varchar>,
        failure_code -> Nullable<Varchar>,
        failure_message -> Nullable<Varchar>,
        livemode -> Bool,
        on_behalf_of -> Nullable<Varchar>,
        order -> Nullable<Varchar>,
        paid -> Bool,
        refunded -> Bool,
        source_id -> Varchar,
        source_transfer -> Nullable<Varchar>,
        statement_descriptor -> Nullable<Varchar>,
        status -> Varchar,
    }
}

table! {
    users_stripe_customer (id) {
        id -> Bigint,
        user_id -> Bigint,
        uuid -> Varchar,
        account_balance -> Bigint,
        business_vat_id -> Nullable<Varchar>,
        created_at_stripe -> Bigint,
        default_source -> Nullable<Varchar>,
        delinquent -> Bool,
        desc -> Nullable<Text>,
        email -> Nullable<Varchar>,
        livemode -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    users_stripe_token (id) {
        id -> Bigint,
        user_id -> Bigint,
        client_ip -> Varchar,
        created_at_stripe -> Bigint,
        token_id -> Varchar,
        livemode -> Bool,
        object -> Nullable<Varchar>,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
        used -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    users_verify_email (id) {
        id -> Bigint,
        user_id -> Bigint,
        token -> Varchar,
        used -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    users_views (id) {
        id -> Bigint,
        user_id -> Bigint,
        video_id -> Bigint,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    videos (id) {
        id -> Bigint,
        uuid -> Varchar,
        title -> Varchar,
        slug -> Varchar,
        description -> Text,
        published -> Bool,
        membership_only -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
        serie_id -> Bigint,
        episode_number -> Integer,
        archived -> Bool,
        vimeo_id -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    groups,
    newsletter_subscribers,
    request_network_hashes,
    request_network_payments,
    series,
    users,
    users_group,
    users_recover_email,
    users_series_access,
    users_sessions,
    users_stripe_card,
    users_stripe_charge,
    users_stripe_customer,
    users_stripe_token,
    users_verify_email,
    users_views,
    videos,
);
