// @generated automatically by Diesel CLI.

diesel::table! {
    tasks (id) {
        id -> Nullable<Integer>,
        title -> Text,
        done -> Bool,
        username -> Text,
    }
}
