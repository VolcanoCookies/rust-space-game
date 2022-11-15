pub use bevy::prelude::*;
pub use bevy_renet::renet::*;
pub use bevy_renet::*;
use serde::{Deserialize, Serialize};

pub use shared::*;

pub mod math;

pub mod shared;

pub mod client;
pub mod server;

pub const PROTOCOL_ID: u64 = 1;
