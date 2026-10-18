use scoretracker::{data::library::stpl_url::LibraryDomain, db::Database};
use smol::lock::Mutex;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::server::{
    config::ServerConfig,
    domain_resolved::{DomainResolveError, DomainResolved},
    library_hall::LibraryConnections,
};

pub struct ServerGlobals {
    pub server_config: Arc<ServerConfig>,
    pub connected_libraries: Arc<RwLock<LibraryConnections>>,
    pub db: Arc<Mutex<Database>>,
}

impl ServerGlobals {
    pub fn resolve_domain(&self, domain: &LibraryDomain, auth_info: Option<Uuid>) -> Result<DomainResolved, DomainResolveError> {
        let lock = self.connected_libraries.read().unwrap();
        let Some(hall) = lock.get(domain) else {
            return Err(DomainResolveError::NotKnown);
        };

        // Additional guards for private domains
        if hall.private {
            // Reject requests without any authentication
            let Some(auth_user_uuid) = auth_info else {
                return Err(DomainResolveError::Unauthorized);
            };

            // Reject users who are not the owner
            if auth_user_uuid != hall.owner_uuid {
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
