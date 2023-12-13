// @generated automatically by Diesel CLI.

diesel::table! {
    actors (id_actor) {
        id_actor -> Int8,
        id_movie -> Int8,
        #[max_length = 100]
        name -> Varchar,
    }
}

diesel::table! {
    categories (id_category) {
        id_category -> Int4,
        #[max_length = 100]
        description -> Varchar,
    }
}

diesel::table! {
    category_movies (id_category_movie) {
        id_category_movie -> Int8,
        id_movie -> Int8,
        id_category -> Int4,
    }
}

diesel::table! {
    connections (id_connection) {
        id_connection -> Uuid,
        id_user -> Uuid,
        connect_at -> Nullable<Timestamp>,
        ended_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    directors (id_director) {
        id_director -> Int8,
        id_movie -> Int8,
        #[max_length = 100]
        name -> Varchar,
    }
}

diesel::table! {
    movies (id_movie) {
        id_movie -> Int8,
        #[max_length = 100]
        name -> Varchar,
        description -> Text,
        duration -> Int4,
        release_date -> Nullable<Timestamp>,
        image_link -> Text,
    }
}

diesel::table! {
    reviews (id_review) {
        id_review -> Int8,
        id_movie -> Int8,
        id_user -> Uuid,
        score -> Int4,
        comment -> Nullable<Text>,
    }
}

diesel::table! {
    rl_role (id_role) {
        id_role -> Int4,
        #[max_length = 50]
        description -> Varchar,
    }
}

diesel::table! {
    rl_users (id_user) {
        id_user -> Uuid,
        #[max_length = 200]
        email -> Varchar,
        #[max_length = 100]
        nickname -> Varchar,
        password -> Text,
        id_role -> Int4,
    }
}

diesel::table! {
    user_favorites (id_user_favorite) {
        id_user_favorite -> Int8,
        id_user -> Uuid,
        id_movie -> Int8,
    }
}

diesel::joinable!(actors -> movies (id_movie));
diesel::joinable!(category_movies -> categories (id_category));
diesel::joinable!(category_movies -> movies (id_movie));
diesel::joinable!(directors -> movies (id_movie));
diesel::joinable!(reviews -> movies (id_movie));
diesel::joinable!(reviews -> rl_users (id_user));
diesel::joinable!(rl_users -> rl_role (id_role));
diesel::joinable!(user_favorites -> movies (id_movie));
diesel::joinable!(user_favorites -> rl_users (id_user));

diesel::allow_tables_to_appear_in_same_query!(
    actors,
    categories,
    category_movies,
    connections,
    directors,
    movies,
    reviews,
    rl_role,
    rl_users,
    user_favorites,
);
