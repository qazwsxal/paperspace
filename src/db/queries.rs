use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use sqlx::{
    self, query_builder::QueryBuilder, sqlite::SqliteQueryResult, types::chrono::NaiveDate, Sqlite,
    Transaction,
};

use crate::session::state::Counter;

pub async fn get_session(uuid: &str, mut tx: Transaction<'_, Sqlite>) -> Option<Counter> {
    sqlx::query_as!(Counter, "select `value` from `sessions` where id = ?", uuid)
        .fetch_optional(&mut *tx)
        .await
        .unwrap()
}
pub async fn save_session(uuid: &str, session_state: Counter, mut tx: Transaction<'_, Sqlite>) {
    let _result = sqlx::query!("INSERT INTO sessions(id, value) VALUES(?, ?) ON CONFLICT(id) DO UPDATE SET value=excluded.value", uuid, session_state.value)
        .execute(&mut *tx)
        .await;
    let _ = tx.commit().await;
}
