pub struct AuthenticatedUser {} //TODO

#[derive(Debug, Clone)]
pub enum AccessRules {
    DenyByDefault { allowlist: Vec<String> }, // TODO: implement authorization
    AllowByDefault { denylist: Vec<String> }, // TODO: implement authorization
}

impl AccessRules {
    pub fn public() -> Self {
        Self::AllowByDefault { denylist: vec![] }
    }
    pub fn denylist(denylist: Vec<String>) -> Self {
        Self::AllowByDefault { denylist }
    }
    pub fn allowlist(allowlist: Vec<String>) -> Self {
        Self::DenyByDefault { allowlist }
    }
    pub fn private() -> Self {
        Self::DenyByDefault { allowlist: vec![] }
    }
    pub fn auth_required(&self) -> bool {
        todo!()
    }
    pub fn does_user_have_access(&self, _auth: AuthenticatedUser) -> bool {
        todo!()
    }
}
