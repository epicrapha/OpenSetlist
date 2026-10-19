//! Opaque token generation shared by invite acceptance and setlist sharing.

use uuid::Uuid;

/// A single-use, URL-safe, high-entropy token: 32 lowercase hex chars (128 bits).
pub fn random_token() -> String {
    Uuid::new_v4().simple().to_string()
}
