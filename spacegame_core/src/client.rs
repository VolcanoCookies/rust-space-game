use std::{collections::VecDeque, fmt::Debug};

use bevy::{
    ecs::event::Event,
    prelude::{
        App, Commands, CoreStage, EventReader, EventWriter, ParallelSystemDescriptorCoercion,
        Plugin, Res, ResMut,
    },
    utils::HashMap,
};
use bevy_renet::{renet::RenetClient, RenetClientPlugin};
use unique_type_id::UniqueTypeId;

use crate::{
    has_resource,
    message::{ClientId, ClientMessageOutQueue, Kind, NetworkEventChannelId, UntypedPacket},
    network_id::NetworkIdMap,
    Labels, NetworkEvent, NetworkEventDirection,
};

pub struct ClientNetworkPlugin;

impl ClientNetworkPlugin {}

impl Plugin for ClientNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetClientPlugin)
            .insert_resource(NetworkIdMap::new())
            .insert_resource(MessageInQueues {
                map: HashMap::new(),
            })
            .add_system_to_stage(
                CoreStage::PreUpdate,
                receive_untyped
                    .with_run_criteria(has_resource::<RenetClient>)
                    .label(Labels::ReceiveUntyped),
            );
    }

    fn name(&self) -> &str {
        "spacegame_client_network"
    }
}

pub trait AppClientNetworkTrait {
    fn add_network_event<
        T: NetworkEvent + NetworkEventChannelId + UniqueTypeId<u16> + NetworkEventDirection + Debug,
    >(
        &mut self,
    ) -> &mut Self;
}

impl AppClientNetworkTrait for App {
    fn add_network_event<
        T: NetworkEvent + NetworkEventChannelId + UniqueTypeId<u16> + NetworkEventDirection + Debug,
    >(
        &mut self,
    ) -> &mut Self {
        match T::DIRECTION {
            crate::Direction::Clientbound => {
                let mut queues = self.world.resource_mut::<MessageInQueues>();
                queues.map.insert(T::TYPE_ID.0, VecDeque::new());

                self.add_event::<T>().add_system_to_stage(
                    CoreStage::PreUpdate,
                    after_receive_typed::<T>
                        .after(Labels::ReceiveUntyped)
                        .with_run_criteria(has_resource::<RenetClient>)
                        .label(Labels::AfterReceiveTyped),
                )
            }
            crate::Direction::Serverbound => self
                .add_event::<T>()
                .insert_resource(ClientMessageOutQueue::<T>::new(T::CHANNEL_ID, T::TYPE_ID.0))
                .add_system_to_stage(
                    CoreStage::PostUpdate,
                    before_send_typed::<T>
                        .before(bevy_renet::RenetServerPlugin::send_packets_system)
                        .with_run_criteria(has_resource::<RenetClient>)
                        .label(Labels::BeforeSendTyped),
                ),
            crate::Direction::Bidirectional => {
                let mut queues = self.world.resource_mut::<MessageInQueues>();
                queues.map.insert(T::TYPE_ID.0, VecDeque::new());

                self.add_event::<T>()
                    .insert_resource(ClientMessageOutQueue::<T>::new(T::CHANNEL_ID, T::TYPE_ID.0))
                    .add_system_to_stage(
                        CoreStage::PreUpdate,
                        after_receive_typed::<T>
                            .after(Labels::ReceiveUntyped)
                            .with_run_criteria(has_resource::<RenetClient>)
                            .label(Labels::AfterReceiveTyped),
                    )
                    .add_system_to_stage(
                        CoreStage::PostUpdate,
                        before_send_typed::<T>
                            .before(bevy_renet::RenetServerPlugin::send_packets_system)
                            .with_run_criteria(has_resource::<RenetClient>)
                            .label(Labels::BeforeSendTyped),
                    )
            }
        }
    }
}

/// System that is ran right before sending packets.
///
/// It will also replace entity ids with network ids, and drop any appropriate events.
///
/// Runs in the [bevy::prelude::CoreStage::PostUpdate] stage.
///
/// Will drop any invalid packets, then forward the rest to the [bevy_renet::renet::RenetClient] instance
/// right before [bevy::prelude::CoreStage::PostUpdate] system so that we do not have a one frame delay.
fn before_send_typed<T>(
    mut network_id_map: ResMut<NetworkIdMap>,
    mut queue: ResMut<ClientMessageOutQueue<T>>,
    mut client: ResMut<RenetClient>,
) where
    T: NetworkEvent + NetworkEventChannelId + UniqueTypeId<Kind> + Debug,
{
    while let Some(mut message) = queue.raw.pop_front() {
        if message.entity_to_network(&mut network_id_map) {
            message.set_client_id(client.client_id());
            let packet = UntypedPacket {
                kind: queue.kind,
                data: bincode::serialize(&message).unwrap(),
            };
            client.send_message(queue.channel_id, bincode::serialize(&packet).unwrap());
        }
    }
}

/// A map of unique type ids, to queues for holding their raw data before deserailization in the [after_receive_typed] system.
struct MessageInQueues {
    map: HashMap<u16, VecDeque<Vec<u8>>>,
}

/// System that is ran right after [receive_untyped].
///
/// Takes the map of queues with raw data and serializes each one into the corrent type, outputting it on the event bus.
///
/// This system runs in the [bevy::prelude::CoreStage::PreUpdate] stage so that the events are immediately available in the same
/// frame that they are received.
fn after_receive_typed<T>(
    mut queues: ResMut<MessageInQueues>,
    mut events: EventWriter<T>,
    mut commands: Commands,
    mut network_id_map: ResMut<NetworkIdMap>,
) where
    T: UniqueTypeId<Kind> + Event + NetworkEvent + Debug,
{
    if let Some(queue) = queues.map.get_mut(&T::TYPE_ID.0) {
        while let Some(data) = queue.pop_front() {
            let mut event: T = bincode::deserialize(&data).unwrap();
            if event.network_to_entity(&mut commands, &mut network_id_map) {
                events.send(event);
            }
        }
    }
}

/// This stage receives packets from clients.
/// Each packet is deserialized into a untyped packet, [NetworkPacket][kind]
fn receive_untyped(mut client: ResMut<RenetClient>, mut queues: ResMut<MessageInQueues>) {
    while let Some(data) = client.receive_message(0) {
        let untyped_packet: UntypedPacket = bincode::deserialize(&data).unwrap();
        queues
            .map
            .get_mut(&untyped_packet.kind)
            .unwrap()
            .push_back(untyped_packet.data);
    }
}
