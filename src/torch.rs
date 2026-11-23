//! A floor torch's item placing the wall torch against a wall, as the original's
//! `StandingAndWallBlockItem` does: vanilla's `StandingAndWallBlockItem.getPlacementState` and
//! `WallTorchBlock.getStateForPlacement` over the placer's nearest looking directions.

use mod_api::Direction;

/// `Direction.orderedByNearest`: the six directions, nearest to where the placer looks first.
pub fn ordered_by_nearest(yaw: f32, pitch: f32) -> [Direction; 6] {
    let pitch = pitch.to_radians();
    let yaw = -yaw.to_radians();
    let (sin_pitch, cos_pitch) = (pitch.sin(), pitch.cos());
    let (sin_yaw, cos_yaw) = (yaw.sin(), yaw.cos());
    let x_pos = sin_yaw > 0.0;
    let y_pos = -sin_pitch > 0.0;
    let z_pos = cos_yaw > 0.0;
    let x_yaw = if x_pos { sin_yaw } else { -sin_yaw };
    let y_mag = if y_pos { -sin_pitch } else { sin_pitch };
    let z_yaw = if z_pos { cos_yaw } else { -cos_yaw };
    let x_mag = x_yaw * cos_pitch;
    let z_mag = z_yaw * cos_pitch;
    let axis_x = if x_pos {
        Direction::East
    } else {
        Direction::West
    };
    let axis_y = if y_pos {
        Direction::Up
    } else {
        Direction::Down
    };
    let axis_z = if z_pos {
        Direction::South
    } else {
        Direction::North
    };
    let (a, b, c) = if x_yaw > z_yaw {
        if y_mag > x_mag {
            (axis_y, axis_x, axis_z)
        } else if z_mag > y_mag {
            (axis_x, axis_z, axis_y)
        } else {
            (axis_x, axis_y, axis_z)
        }
    } else if y_mag > z_mag {
        (axis_y, axis_z, axis_x)
    } else if x_mag > y_mag {
        (axis_z, axis_x, axis_y)
    } else {
        (axis_z, axis_y, axis_x)
    };
    [a, b, c, c.opposite(), b.opposite(), a.opposite()]
}

/// `BlockPlaceContext.getNearestLookingDirections`: unless the clicked block is replaced, the
/// direction into the clicked face comes first.
pub fn nearest_looking(
    yaw: f32,
    pitch: f32,
    face: Direction,
    replace_clicked: bool,
) -> [Direction; 6] {
    let mut dirs = ordered_by_nearest(yaw, pitch);
    if !replace_clicked {
        let into = face.opposite();
        if let Some(i) = dirs.iter().position(|d| *d == into) {
            dirs[..=i].rotate_right(1);
        }
    }
    dirs
}

/// Where the item puts the torch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Floor,
    /// A wall torch facing away from its wall.
    Wall(Direction),
}

/// `StandingAndWallBlockItem.getPlacementState` with `attachmentDirection` down: the first
/// looking direction that gives a torch that can stand (`floor` for the floor form, `wall`
/// for a wall form facing a direction); none when no form can.
pub fn placement(
    looking: [Direction; 6],
    floor: impl Fn() -> bool,
    wall: impl Fn(Direction) -> bool,
) -> Option<Placement> {
    // WallTorchBlock.getStateForPlacement: the first horizontal looking direction whose wall
    // holds the torch.
    let wall_form = looking
        .iter()
        .filter(|d| !matches!(d, Direction::Up | Direction::Down))
        .map(|d| d.opposite())
        .find(|facing| wall(*facing));
    looking
        .iter()
        .filter(|d| **d != Direction::Up)
        .find_map(|d| match d {
            Direction::Down => floor().then_some(Placement::Floor),
            _ => wall_form.map(Placement::Wall),
        })
}

pub fn facing_name(d: Direction) -> &'static str {
    match d {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looking_down_puts_down_first_and_the_clicked_face_leads() {
        assert_eq!(ordered_by_nearest(0.0, 90.0)[0], Direction::Down);
        assert_eq!(ordered_by_nearest(0.0, 0.0)[0], Direction::South);
        // Looking south but clicking a wall's east face: west (into it) comes first.
        assert_eq!(
            nearest_looking(0.0, 0.0, Direction::East, false)[0],
            Direction::West
        );
    }

    #[test]
    fn a_click_on_a_wall_places_the_wall_torch_facing_out() {
        let looking = nearest_looking(0.0, 0.0, Direction::North, false);
        assert_eq!(
            placement(looking, || true, |f| f == Direction::North),
            Some(Placement::Wall(Direction::North))
        );
        // Looking down at the floor with a wall in reach: the floor form.
        let looking = nearest_looking(0.0, 80.0, Direction::Up, false);
        assert_eq!(
            placement(looking, || true, |_| true),
            Some(Placement::Floor)
        );
        // Nothing holds it.
        assert_eq!(placement(looking, || false, |_| false), None);
    }
}
