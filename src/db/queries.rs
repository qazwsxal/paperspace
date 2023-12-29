use sqlx::{
    self, Sqlite,
    Transaction,
};

use crate::session::state::Counter;

// pub(crate) const BIND_LIMIT: usize = 32766; //SQLITE_LIMIT_VARIABLE_NUMBER default value.

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
