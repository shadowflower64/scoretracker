use serde::{Deserialize, Serialize};

/// Capabilities of this worker
///
/// Note: These are still not clearly defined, this is all still TODO.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Capabilities {
    /// Whether or not this worker prefers to only work on small tasks, because it is not very powerful.
    pub light_tasks_only: bool,

    /// Whether or not this worker has access to a graphics card that can be used for processing tasks.
    pub gpu: bool,

    /// Whether or not this worker is a *dedicated* worker.
    ///
    /// Dedicated workers live on machines that are dedicated to be scoretracker hive workers. Launching a task using this worker will not disturb anyone.
    pub dedicated: bool,

    /// Whether or not this worker is a *lazy* worker.
    ///
    /// Lazy workers will do tasks only when the owner is away from the computer or only at night so as not to disturb the machine owner.
    pub lazy: bool,
}

/// Message sent from the worker process to the scoretracker server (serverbound).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerboundMessage {
    TestingHello { message: String },
    TestingGotcha { message: String },
    Capabilities(Capabilities),
}

/// Message sent from the scoretracker server to the worker process (clientbound).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientboundMessage {
    RequestCapabilities,
    TestingLater { message: String },
    TestingIHeard { message: String },
    TestingAutomated { message: String },
}
