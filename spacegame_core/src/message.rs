use std::{collections::VecDeque, marker::PhantomData};

use bevy::{
    ecs::system::SystemParam,
    prelude::{Res, ResMut},
};
use serde::{Deserialize, Serialize};

use crate::{Clientbound, NetworkEvent, Serverbound};

pub type ClientId = u64;

pub type ChannelId = u8;

pub type Kind = u16;

/// The destination of a client-bound packet.
pub enum Destination {
    Client(ClientId),
    Except(ClientId),
    Broadcast,
}

/// Each event that can be sent over the network implements this Trait, so that we easily access which
/// channel it should be sent on.
pub trait NetworkEventChannelId {
    const CHANNEL_ID: ChannelId;
}

/// A typed message queue resource.
///
/// For the server side, since we need to keep in mind the specific destination of each message.
///
/// Putting events in this queue will, in the [bevy::prelude::CoreStage::PostUpdate] stage,
/// drop any invalid events then serialize the rest and send it using [bevy_renet::renet::RenetServer].
///
/// This is done before the [bevy_renet::RenetServerPlugin::send_packets_system] system so that we do
/// not have a one frame delay for sending packets.
pub struct ServerMessageOutQueue<T>
where
    T: NetworkEvent,
{
    /// A queue of messages we are to transmit at the end of the frame.
    pub(crate) raw: VecDeque<(Destination, T)>,
    /// The channel these messages will be sent to.
    pub(crate) channel_id: ChannelId,
    /// The kind of these messages.
    ///
    /// We store it here so that we do not have to also have [T] be [unique_type_id::UniqueTypeId<Kind>].
    pub(crate) kind: Kind,
}

impl<T> ServerMessageOutQueue<T>
where
    T: NetworkEvent,
{
    pub fn new(channel_id: ChannelId, kind: Kind) -> Self {
        Self {
            raw: VecDeque::new(),
            channel_id,
            kind,
        }
    }

    pub fn send(&mut self, client_id: &ClientId, message: T) {
        self.raw
            .push_back((Destination::Client(*client_id), message));
    }

    pub fn broadcast(&mut self, message: T) {
        self.raw.push_back((Destination::Broadcast, message));
    }

    pub fn broadcast_except(&mut self, client_id: &ClientId, message: T) {
        self.raw
            .push_back((Destination::Except(*client_id), message));
    }
}

/// A typed message queue resource.
///
/// For the client side, since we do not need to keep track of a destination for each message.
///
/// Putting events in this queue will, in the [bevy::prelude::CoreStage::PostUpdate] stage,
/// drop any invalid events then serialize the rest and send it using [bevy_renet::renet::RenetServer].
///
/// This is done before the [bevy_renet::RenetServerPlugin::send_packets_system] system so that we do
/// not have a one frame delay for sending packets.
pub struct ClientMessageOutQueue<T>
where
    T: NetworkEvent,
{
    /// A queue of messages we are to transmit at the end of the frame.
    pub(crate) raw: VecDeque<T>,
    /// The channel these messages will be sent to.
    pub(crate) channel_id: ChannelId,
    /// The kind of these messages.
    ///
    /// We store it here so that we do not have to also have [T] be [unique_type_id::UniqueTypeId<Kind>].
    pub(crate) kind: Kind,
}

impl<T> ClientMessageOutQueue<T>
where
    T: NetworkEvent,
{
    pub fn new(channel_id: ChannelId, kind: Kind) -> Self {
        Self {
            raw: VecDeque::new(),
            channel_id,
            kind,
        }
    }

    pub fn send(&mut self, message: T) {
        self.raw.push_back(message);
    }
}

pub struct MessageInQueue<T>
where
    T: NetworkEvent,
{
    pub(crate) raw: VecDeque<T>,
}

/// A untyped packed, contains only a unique type id to the type that is contained within data.
///
/// This type is useful so that we can deserialize a packet partially, read the unique type id, and use it to deserialize it into the correct type.
#[derive(Serialize, Deserialize)]
pub struct UntypedPacket {
    /// The kind of the binary data.
    pub(crate) kind: Kind,
    /// The binary data.
    pub(crate) data: Vec<u8>,
}
