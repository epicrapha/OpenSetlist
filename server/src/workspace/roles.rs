//! Workspace roles and ordering.

use serde::{Deserialize, Serialize};

/// Role within a workspace. Ordered: owner > admin > leader > member > viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "membership_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Owner,
    Admin,
    Leader,
    Member,
    Viewer,
}

impl Role {
    fn rank(self) -> u8 {
        match self {
            Role::Owner => 4,
            Role::Admin => 3,
            Role::Leader => 2,
            Role::Member => 1,
            Role::Viewer => 0,
        }
    }

    /// True when this role is at least as privileged as `minimum`.
    pub fn at_least(self, minimum: Role) -> bool {
        self.rank() >= minimum.rank()
    }

    pub fn parse(value: &str) -> Option<Role> {
        match value.to_ascii_lowercase().as_str() {
            "owner" => Some(Role::Owner),
            "admin" => Some(Role::Admin),
            "leader" => Some(Role::Leader),
            "member" => Some(Role::Member),
            "viewer" => Some(Role::Viewer),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Role::Owner => "owner",
            Role::Admin => "admin",
            Role::Leader => "leader",
            Role::Member => "member",
            Role::Viewer => "viewer",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Role;

    #[test]
    fn owner_outranks_member_and_viewer_does_not() {
        assert!(Role::Owner.at_least(Role::Admin));
        assert!(Role::Admin.at_least(Role::Admin));
        assert!(!Role::Member.at_least(Role::Admin));
        assert!(!Role::Viewer.at_least(Role::Member));
        assert!(Role::Member.at_least(Role::Viewer));
    }

    #[test]
    fn parse_roundtrips_and_rejects_unknown() {
        for role in [
            Role::Owner,
            Role::Admin,
            Role::Leader,
            Role::Member,
            Role::Viewer,
        ] {
            assert_eq!(Role::parse(role.as_str()), Some(role));
        }
        assert_eq!(Role::parse("superuser"), None);
    }
}
