use bevy::prelude::{
    default, BuildChildren, Commands, Entity, EventReader, PbrBundle, Plugin, Query, Res, ResMut,
};
use bevy_renet::renet::RenetClient;

use crate::model::block::{BlockBundle, BlockType};
use crate::model::block_map::{BlockMap, BlockPosition};
use crate::resources::block_registry::{self, BlockRegistry};
use crate::shared::events::ship::AddBlockEvent;
use crate::shared::networking::message::{NetworkMessage, ServerMessage};
use crate::shared::networking::network_id::NetworkIdMap;

pub struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<BlockPlaceEvent>()
            .add_event::<BlockRemoveEvent>()
            .add_system(place)
            .add_system(remove);
    }

    fn name(&self) -> &str {
        "PlacementPlugin"
    }
}

pub struct BlockPlaceEvent {
    pub block_type: BlockType,
    pub block_position: BlockPosition,
    pub ship_entity: Entity,
}

pub struct BlockRemoveEvent {
    pub block_entity: Entity,
    pub ship_entity: Entity,
}

fn place(
    mut commands: Commands,
    //mut client: ResMut<RenetClient>,
    mut network_ids: ResMut<NetworkIdMap>,
    block_registry: Res<BlockRegistry>,
    mut block_place_events: EventReader<BlockPlaceEvent>,
    mut block_map_query: Query<&mut BlockMap>,
) {
    for event in block_place_events.iter() {
        if let Ok(mut block_map) = block_map_query.get_mut(event.ship_entity) {
            if None == block_map.get(&event.block_position) {
                let block_entity = commands
                    .spawn_bundle(BlockBundle {
                        block_type: event.block_type,
                        block_position: event.block_position,
                        pbr_bundle: PbrBundle {
                            transform: event.block_position.into(),
                            material: block_registry.get_material(event.block_type),
                            mesh: block_registry.get_mesh(event.block_type),
                            ..default()
                        },
                        ..default()
                    })
                    .id();
                commands.entity(event.ship_entity).add_child(block_entity);
                block_map.set(event.block_position, event.block_type, block_entity);

                let ship_network_id = network_ids.get(event.ship_entity);

                let message = ServerMessage::AddBlock(AddBlockEvent {
                    ship_network_id,
                    block_position: event.block_position,
                    block_type: event.block_type,
                });

                // send_client(client, &message);
            }
        }
    }
}

fn send_client<T: ?Sized>(client: &mut RenetClient, message: &T)
where
    T: serde::Serialize + NetworkMessage,
{
    client.send_message(message.channel_id(), bincode::serialize(message).unwrap());
}

pub fn remove(
    mut commands: Commands,
    mut block_remove_event: EventReader<BlockRemoveEvent>,
    block_position_query: Query<&BlockPosition>,
    mut block_map_query: Query<&mut BlockMap>,
) {
    for event in block_remove_event.iter() {
        if let Ok(mut block_map) = block_map_query.get_mut(event.ship_entity) {
            if let Ok(block_position) = block_position_query.get(event.block_entity) {
                block_map.remove(block_position);
                commands
                    .entity(event.ship_entity)
                    .remove_children(&[event.block_entity]);
                commands.entity(event.block_entity).despawn();
            }
        }
    }
}
