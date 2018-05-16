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
    series (id) {
        id -> Bigint,
        uuid -> Varchar,
        title -> Varchar,
        slug -> Varchar,
        description -> Text,
        published -> Bool,
        archived -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Bigint,
        username -> Varchar,
        password -> Varchar,
        verified -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
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
    users_stripe_subscriptions (id) {
        id -> Bigint,
        user_id -> Bigint,
        uuid -> Varchar,
        application_fee_percent -> Nullable<Float>,
        cancel_at_period_end -> Bool,
        canceled_at -> Nullable<Bigint>,
        created_at -> Nullable<Bigint>,
        current_period_start -> Bigint,
        current_period_end -> Bigint,
        customer -> Varchar,
        ended_at -> Nullable<Bigint>,
        livemode -> Bool,
        quantity -> Bigint,
        start -> Bigint,
        status -> Varchar,
        tax_percent -> Nullable<Float>,
        trial_start -> Nullable<Bigint>,
        trial_end -> Nullable<Bigint>,
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
        series -> Nullable<Bigint>,
        episode_number -> Nullable<Integer>,
        archived -> Bool,
        vimeo_id -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    groups,
    series,
    users,
    users_group,
    users_sessions,
    users_stripe_card,
    users_stripe_customer,
    users_stripe_subscriptions,
    users_stripe_token,
    users_verify_email,
    users_views,
    videos,
);
