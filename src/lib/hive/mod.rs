//! A worker process manager system.
//!
//! The "hive" is a system that manages tasks across multiple worker processes.
//! Currently, all of the workers have to live on the same system, but in the future this may change into a distributed system,
//! where multiple computer systems can contribute to doing queued tasks.
//!
//! The hive system consists of several submodules, which are listed below.
pub mod job;
pub mod jobs;
pub mod queue;
pub mod task;
pub mod worker;
