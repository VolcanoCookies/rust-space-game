use bevy::prelude::{
    BuildChildren, Commands, EventReader, GlobalTransform, Plugin, Query, Transform,
};

use crate::events::generic::{BindPositionEvent, UnbindPositionEvent};

pub struct BindingPlugin;

impl Plugin for BindingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<BindPositionEvent>()
            .add_event::<UnbindPositionEvent>()
            .add_system(on_bind_position)
            .add_system(on_unbind_position);
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

fn on_bind_position(
    mut commands: Commands,
    mut events: EventReader<BindPositionEvent>,
    mut query: Query<(&mut Transform, &GlobalTransform)>,
) {
    for event in events.iter() {
        if event.parent == event.child {
            continue;
        }

        let global_transform = if let Ok(parent_global_transform) =
            query.get_component::<GlobalTransform>(event.parent)
        {
            parent_global_transform.clone()
        } else {
            continue;
        };

        if let Ok((mut child_transform, child_global_transform)) = query.get_mut(event.child) {
            child_transform.translation =
                child_global_transform.translation() - global_transform.translation();
            child_transform.translation = global_transform
                .affine()
                .inverse()
                .transform_point3(child_global_transform.translation());

            commands.entity(event.parent).add_child(event.child);
        }
    }
}

fn on_unbind_position(
    mut commands: Commands,
    mut events: EventReader<UnbindPositionEvent>,
    mut query: Query<(&mut Transform, &GlobalTransform)>,
) {
    for event in events.iter() {
        if event.parent == event.child {
            continue;
        }

        if let Ok((mut child_transform, child_global_transform)) = query.get_mut(event.child) {
            *child_transform = child_global_transform.compute_transform();
            commands
                .entity(event.parent)
                .remove_children(&[event.child]);
        }
    }
}
