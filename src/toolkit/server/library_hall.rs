use super::access_rules::AccessRules;
use function_name::named;
use scoretracker::config::library_tab::{InternalLibraryAccessPath, InternalLibraryConnections, MirrorStatus};
use scoretracker::data::library::stpl_url::LibraryDomain;
use scoretracker::{log_fn_name, warn};
use std::ops::Deref;
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug, Clone)]
pub enum LibraryMirrorLocation {
    /// This library is on the same system as the server, and the files of the library can be directly accessed by the server process.
    Internal(InternalLibraryAccessPath),

    /// This library is on a different server, and direct filesystem access is not available.
    //External { address: IpAddr, port: u16 },
    External { url: String },
}

impl LibraryMirrorLocation {
    pub fn unique_key(&self) -> Cow<'_, str> {
        match self {
            Self::Internal(internal) => internal.unique_key(),
            Self::External { url } => Cow::Borrowed(url),
        }
    }
}

/// One of the places to access the library at.
#[derive(Debug, Clone)]
pub struct LibraryMirror {
    /// Internal or external.
    pub location: LibraryMirrorLocation,

    /// Whether this mirror is currently available.
    pub status: MirrorStatus,
}

impl LibraryMirror {
    pub fn unique_key(&self) -> Cow<'_, str> {
        self.location.unique_key()
    }
}

/// Library hall of mirrors. A list of mirrors that can be used to access the library.
#[derive(Debug, Clone)]
pub struct LibraryHall {
    /// Mirrors of the library. Mirrors can be online or offline, all of them should be stored here.
    pub mirrors: Vec<LibraryMirror>,

    /// Domain-wide permissions, for all of the mirrors.
    pub access_rules: AccessRules,
}

#[derive(Debug, Clone)]
pub struct LibraryConnections {
    connections: HashMap<LibraryDomain, LibraryHall>,
}

impl LibraryConnections {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    #[named]
    pub fn add_mirror(&mut self, domain: LibraryDomain, mirror: LibraryMirror) {
        log_fn_name!(auto);
        if let Some(existing) = self.connections.get_mut(&domain) {
            if existing.mirrors.iter().any(|x| x.unique_key() == mirror.unique_key()) {
                warn!("detected duplicate error, not adding: {mirror:?}");
            } else {
                existing.mirrors.push(mirror);
            }
        }
    }

    pub fn add_internal_mirrors(&mut self, internal_connections: InternalLibraryConnections) {
        for (domain, internal_hall) in internal_connections.connections() {
            for internal_mirror in internal_hall.mirrors() {
                self.add_mirror(
                    domain.clone(),
                    LibraryMirror {
                        location: LibraryMirrorLocation::Internal(internal_mirror.access_path),
                        status: internal_mirror.status,
                    },
                )
            }
        }
    }
}

impl Deref for LibraryConnections {
    type Target = HashMap<LibraryDomain, LibraryHall>;
    fn deref(&self) -> &Self::Target {
        &self.connections
    }
}
