//! Tenant-scoped database access.
//!
//! Every query against a tenant table runs inside a `TenantTx`, which is a
//! transaction with:
//!
//! 1. `SET LOCAL ROLE opensetlist_app` — a non-superuser role, because
//!    superusers bypass Row Level Security unconditionally. Dropping to the app
//!    role is what makes the RLS policies actually bite.
//! 2. `set_config('app.current_tenant_id', $1, true)` — the workspace scope the
//!    RLS policies read.
//! 3. `set_config('app.current_user_id', $1, true)` — lets a user read their
//!    own membership rows (for `/auth/me`).
//!
//! The `is_local = true` flag (and `SET LOCAL ROLE`) makes all of this
//! transaction-scoped: it resets on commit/rollback, so a pooled connection can
//! never leak another tenant's context. A connection-level `SET` could.

use std::ops::{Deref, DerefMut};

use sqlx::{PgConnection, Postgres, Transaction};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

pub struct TenantTx<'a> {
    tx: Transaction<'a, Postgres>,
}

impl Deref for TenantTx<'_> {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl DerefMut for TenantTx<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tx
    }
}

impl TenantTx<'_> {
    pub async fn commit(self) -> Result<(), AppError> {
        self.tx.commit().await?;
        Ok(())
    }
}

impl AppState {
    /// Begin a transaction scoped to a tenant (workspace) and/or a user.
    pub async fn scoped(
        &self,
        tenant_id: Option<Uuid>,
        user_id: Option<Uuid>,
    ) -> Result<TenantTx<'_>, AppError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("set local role opensetlist_app")
            .execute(&mut *tx)
            .await?;

        if let Some(tenant) = tenant_id {
            sqlx::query("select set_config('app.current_tenant_id', $1, true)")
                .bind(tenant.to_string())
                .execute(&mut *tx)
                .await?;
        }
        if let Some(user) = user_id {
            sqlx::query("select set_config('app.current_user_id', $1, true)")
                .bind(user.to_string())
                .execute(&mut *tx)
                .await?;
        }

        Ok(TenantTx { tx })
    }
}
