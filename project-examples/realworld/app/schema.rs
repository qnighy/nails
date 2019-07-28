table! {
    article_tags (id) {
        id -> Int8,
        article_id -> Int8,
        tag_id -> Int8,
    }
}

table! {
    articles (id) {
        id -> Int8,
        slug -> Varchar,
        title -> Varchar,
        description -> Varchar,
        body -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        author_id -> Int8,
    }
}

table! {
    comments (id) {
        id -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        body -> Text,
        article_id -> Int8,
        author_id -> Int8,
    }
}

table! {
    favorited_articles (id) {
        id -> Int8,
        article_id -> Int8,
        user_id -> Int8,
    }
}

table! {
    followings (id) {
        id -> Int8,
        following_id -> Int8,
        follower_id -> Int8,
    }
}

table! {
    tags (id) {
        id -> Int8,
        tag -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int8,
        email -> Varchar,
        token -> Varchar,
        username -> Varchar,
        bio -> Nullable<Text>,
        image -> Nullable<Varchar>,
    }
}

joinable!(article_tags -> articles (article_id));
joinable!(article_tags -> tags (tag_id));
joinable!(articles -> users (author_id));
joinable!(comments -> articles (article_id));
joinable!(comments -> users (author_id));
joinable!(favorited_articles -> articles (article_id));
joinable!(favorited_articles -> users (user_id));

allow_tables_to_appear_in_same_query!(
    article_tags,
    articles,
    comments,
    favorited_articles,
    followings,
    tags,
    users,
);
