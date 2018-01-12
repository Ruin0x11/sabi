use GameContext;
use ecs::globals;
use ecs::prefab;
use graphics::cell::{CellFeature, StairDest, StairDir, StairKind};
use infinigen::ChunkedWorld;
use point::{Point, POINT_ZERO};
use state::GameState;
use world::traits::*;
use world::{self, World, Bounds};
use rand::{self, Rng};
use util::rand_util;

use super::command::*;

pub(super) fn cmd_use_stairs(context: &mut GameContext, dir: StairDir) -> CommandResult<()> {
    let pos = player_pos(context)?;
    let state = &mut context.state;
    let next = find_stair_dest(state, pos, dir)?;

    let (true_next, dest) = load_stair_dest(state, pos, next)?;
    state.world.move_to_map(true_next, dest).unwrap();

    for pos in rand_util::random_positions(&state.world, 50, &mut rand::thread_rng()).iter() {
        let item = prefab::random_item();
        state.world.spawn(item, *pos);
    }

    debug!(state.world.logger, "map id: {:?}", state.world.map_id());
    Ok(())
}

fn find_stair_dest(state: &GameState, pos: Point, dir: StairDir) -> CommandResult<StairDest> {
    let stair_cell =
        state.world
             .cell_const(&pos)
             .ok_or(CommandError::Bug("World was not loaded at stair pos!"))?;

    match stair_cell.feature {
        Some(CellFeature::Stairs(stair_dir, dest)) => {
            if stair_dir != dir {
                return Err(CommandError::Cancel);
            }

            if let StairDest::Ungenerated(StairKind::Unconnected) = dest {
                return Err(CommandError::Invalid("This stair doesn't lead anywhere..."));
            }

            debug!(state.world.logger, "STAIR at {}: {:?}", pos, dest);

            Ok(dest)
        },
        _ => Err(CommandError::Cancel),
    }
}

fn load_stair_dest(state: &mut GameState,
                   stair_pos: Point,
                   next: StairDest)
                   -> CommandResult<(World, Point)> {
    match next {
        StairDest::Generated(map_id, dest) => {
            let world = &mut state.world;
            debug!(world.logger, "Found stair leading to: {:?}", map_id);
            let world =
                world::serial::load_world(map_id)
                    .map_err(|_| CommandError::Bug("Failed to load already generated world!"))?;
            Ok((world, dest))
        },
        StairDest::Ungenerated(stair_kind) => {
            debug!(state.world.logger, "Failed to load map, generating...");

            let res = {
                generate_stair_dest(state, stair_pos, stair_kind)
            };
            debug!(state.world.logger, "new stairs: {:?}", state.world.cell_const(&stair_pos));
            res
        },
    }
}

fn generate_dungeon_floor(state: &mut GameState, dungeon_id: u64) -> CommandResult<World> {
    let dungeon = state.globals
                       .dungeons
                       .get_mut(&dungeon_id)
                       .expect("Invalid dungeon!");
    let mut new_floor =
        dungeon.data
               .generate_next_floor(&state.world)
               .ok_or(CommandError::Bug("Failed to generate stair!"))?;

    integrate_world_with_dungeon(&mut new_floor, dungeon, dungeon_id)?;

    debug!(state.world.logger, "Dungeon status: {:?}", dungeon);

    Ok(new_floor)
}

fn generate_dungeon_branch(state: &mut GameState,
                           dungeon_id: u64,
                           branch: usize)
                           -> CommandResult<World> {
    let dungeon = state.globals
                       .dungeons
                       .get_mut(&dungeon_id)
                       .expect("Invalid dungeon!");
    let mut new_floor =
        dungeon.data
               .generate_branch(&state.world, branch)
               .ok_or(CommandError::Bug("Failed to generate stair!"))?;

    integrate_world_with_dungeon(&mut new_floor, dungeon, dungeon_id)?;

    debug!(state.world.logger, "Dungeon status: {:?}", dungeon);

    Ok(new_floor)
}

/// Connects the given world to a dungeon as a dungeon floor.
fn integrate_world_with_dungeon(new_floor: &mut World,
                                dungeon: &globals::Dungeon,
                                dungeon_id: u64)
                                -> CommandResult<()> {
    let floor_id = new_floor.flags().map_id;
    assert!(dungeon.data.has_floor(floor_id));

    // If this is a leaf node, there is no way farther down.
    if dungeon.data.is_leaf(floor_id) {
        return Ok(());
    }

    // If we are at a branch point, wire the stairs down to point to each one of the branches.
    // Otherwise, wire them to continue the current dungeon section.
    let is_branch_point = dungeon.data.is_branch_point(floor_id);
    let mut branches = if is_branch_point {
        dungeon.data
               .branches(floor_id)
               .ok_or(CommandError::Bug("No dungeon branches available!"))?
               .iter()
               .collect()
    } else {
        Vec::new()
    };

    // Now, connect the stairs to the next floor
    for (pos, marker) in new_floor.terrain().markers.clone().iter() {
        if marker == "stairs_out" {
            debug!(new_floor.logger, "Connecting stairs to dungeon {:?}", dungeon_id);
            let mut stairs_mut = new_floor.cell_mut(&pos).unwrap();

            if let Some(CellFeature::Stairs(_, StairDest::Ungenerated(ref mut kind))) =
                stairs_mut.feature
            {
                let stair_kind = if is_branch_point {
                    let branch =
                        branches.pop()
                                .expect("More stairs on floor than branches in section!");
                    StairKind::DungeonBranch(dungeon_id, *branch)
                } else {
                    StairKind::Dungeon(dungeon_id)
                };
                *kind = stair_kind
            }
        }
    }

    // assert!(branches.is_empty(), "Not all branches were given stairs in the prefab!");

    Ok(())
}

fn generate_stair_dest(state: &mut GameState,
                       stair_pos: Point,
                       stair_kind: StairKind)
                       -> CommandResult<(World, Point)> {
    let mut new_world = match stair_kind {
        StairKind::Dungeon(dungeon_id) => generate_dungeon_floor(state, dungeon_id)?,
        StairKind::DungeonBranch(dungeon_id, branch) => {
            generate_dungeon_branch(state, dungeon_id, branch)?
        },
        StairKind::Unconnected => {
            return Err(CommandError::Bug("Stair was left in an unconnected state!"));
        },
    };

    let prev_id = state.world.flags().map_id;
    let dest_id = new_world.flags().map_id;

    let stairs_mut = state.world.cell_mut(&stair_pos).unwrap();

    if let Some(CellFeature::Stairs(stair_dir, ref mut dest @ StairDest::Ungenerated(..))) =
        stairs_mut.feature
    {
        let new_stair_pos =
            new_world.find_stairs_in()
                     .ok_or(CommandError::Bug("Generated world has no stairs!"))?;

        *dest = StairDest::Generated(dest_id, new_stair_pos);

        new_world.place_stairs(stair_dir.reverse(), new_stair_pos, prev_id, stair_pos);

        Ok((new_world, new_stair_pos))
    } else {
        Err(CommandError::Bug("Stairs should have already been found by now..."))
    }
}
