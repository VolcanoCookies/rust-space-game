use bevy::{
    ecs::{event::Event, schedule::ShouldRun, system::Resource},
    prelude::{Commands, Res, SystemLabel},
};
use message::ClientId;
use network_id::NetworkIdMap;
use serde::{de::DeserializeOwned, Serialize};
use unique_type_id::UniqueTypeId;

pub mod client;
pub mod message;
pub mod network_id;
pub mod server;

pub trait NetworkEvent: Serialize + DeserializeOwned + UniqueTypeId<u16> + Event {
    // If return false, drop event
    fn network_to_entity(
        &mut self,
        commands: &mut Commands,
        network_id_map: &mut NetworkIdMap,
    ) -> bool;

    // If return false, drop event
    fn entity_to_network(&mut self, network_id_map: &mut NetworkIdMap) -> bool;

    // NO-OP on clientbound events.
    fn set_client_id(&mut self, client_id: ClientId);
}

pub trait Clientbound {}

pub trait Serverbound {}

pub enum Direction {
    Clientbound,
    Serverbound,
    Bidirectional,
}

pub trait NetworkEventDirection {
    const DIRECTION: Direction;
}

fn has_resource<T: Resource>(resource: Option<Res<T>>) -> ShouldRun {
    match resource.is_some() {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

#[derive(SystemLabel)]
enum Labels {
    ReceiveUntyped,
    AfterReceiveTyped,
    BeforeSendTyped,
}
