use std::sync::{Arc, RwLock};

use scoretracker::data::library::stpl_url::LibraryDomain;

use super::{
    access_rules::AuthenticatedUser,
    config::ServerConfig,
    domain_resolved::{DomainResolveError, DomainResolved},
    library_hall::LibraryConnections,
};

pub struct ServerGlobals {
    pub server_config: Arc<ServerConfig>,
    pub connected_libraries: Arc<RwLock<LibraryConnections>>,
}

impl ServerGlobals {
    pub fn resolve_domain(
        &self,
        domain: &LibraryDomain,
        auth_opt: Option<AuthenticatedUser>,
    ) -> Result<DomainResolved, DomainResolveError> {
        let lock = self.connected_libraries.read().unwrap();
        let Some(hall) = lock.get(domain) else {
            return Err(DomainResolveError::NotKnown);
        };

        if hall.access_rules.auth_required() {
            let Some(auth) = auth_opt else {
                return Err(DomainResolveError::Unauthorized);
            };
            if !hall.access_rules.does_user_have_access(auth) {
                return Err(DomainResolveError::Forbidden);
            }
        }

        todo!("find best mirror and return it.. alternatively, return all mirrors and let the caller deal with it")
        // match &hall {
        //     AnyLibraryConnection::External { url } => Ok(DomainResolved::External { url: url.clone() }),
        //     AnyLibraryConnection::Internal { .. } => todo!(), // DomainResolveResult::Internal, // TODO: get libraryaccessapi url if it exists; do not return/expose local paths if possible
        // }
    }
}
