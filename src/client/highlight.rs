use bevy::math::Vec3;

use bevy::prelude::{Color, GlobalTransform, Query, Res, ResMut, Transform};
use bevy_prototype_debug_lines::DebugLines;

use crate::client::controller::LookingAt;
use crate::client::controller::LookingAt::Block;

static CUBE_LINES: [((f32, f32, f32), (f32, f32, f32)); 12] = [
    ((0.5, 0.5, 0.5), (-0.5, 0.5, 0.5)),
    ((0.5, 0.5, 0.5), (0.5, -0.5, 0.5)),
    ((0.5, 0.5, 0.5), (0.5, 0.5, -0.5)),
    ((-0.5, -0.5, 0.5), (-0.5, 0.5, 0.5)),
    ((-0.5, -0.5, 0.5), (0.5, -0.5, 0.5)),
    ((-0.5, -0.5, 0.5), (-0.5, -0.5, -0.5)),
    ((-0.5, 0.5, -0.5), (-0.5, 0.5, 0.5)),
    ((-0.5, 0.5, -0.5), (-0.5, -0.5, -0.5)),
    ((-0.5, 0.5, -0.5), (0.5, 0.5, -0.5)),
    ((0.5, -0.5, -0.5), (-0.5, -0.5, -0.5)),
    ((0.5, -0.5, -0.5), (0.5, 0.5, -0.5)),
    ((0.5, -0.5, -0.5), (0.5, -0.5, 0.5)),
];
static HOVER_COLOR: Color = Color::rgb(0., 0., 0.);

pub fn highlight_mouse_pressed(
    looking_at: Res<LookingAt>,
    transform_query: Query<(&Transform, &GlobalTransform)>,
    mut lines: ResMut<DebugLines>,
) {
    match *looking_at {
        LookingAt::None => {}
        Block(block_entity, _ship_entity, intersect) => {
            if let Ok((_block_transform, block_global_transform)) =
                transform_query.get(block_entity)
            {
                // Direction line
                let start = block_global_transform.translation() + (intersect.normal / 2.);
                let end = start + (intersect.normal / 2.);
                lines.line_gradient(start, end, 0.0, Color::RED, Color::BLUE);

                // Hovered Block
                draw_rect(
                    &mut lines,
                    block_global_transform.compute_transform(),
                    HOVER_COLOR,
                );

                let mut next_block_transform = block_global_transform.compute_transform();
                next_block_transform.translation += intersect.normal;

                draw_rect(&mut lines, next_block_transform, Color::ORANGE_RED);
            }
        }
    }
}

fn draw_rect(lines: &mut DebugLines, transform: Transform, color: Color) {
    for (a, b) in CUBE_LINES {
        let start = transform * Vec3::from(a);
        let end = transform * Vec3::from(b);
        lines.line_colored(start, end, 0., color);
    }
}
