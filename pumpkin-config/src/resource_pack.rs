use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct ResourcePackConfig {
    pub enabled: bool,
    /// The path to the resource pack.
    pub resource_pack_url: String,
    /// The SHA1 hash (40) of the resource pack.
    pub resource_pack_sha1: String,
    /// Custom prompt Text component, Leave blank for none
    pub prompt_message: String,
    /// Will force the Player to accept the resource pack
    pub force: bool,
}

impl ResourcePackConfig {
    pub fn validate(&self) {
        assert_eq!(
            !self.resource_pack_url.is_empty(),
            !self.resource_pack_sha1.is_empty(),
            "Resource Pack path or Sha1 hash is missing"
        );
        assert!(
            self.resource_pack_sha1.len() <= 40,
            "Resource pack sha1 hash is too long (max. 40)"
        )
    }
}
