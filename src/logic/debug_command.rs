use calx_ecs::Entity;

use GameContext;
use ecs;
use point::{Point, RectangleIter, POINT_ZERO};
use prefab;
use renderer;
use state;
use world::traits::*;
use world::{self, World};

use super::command::*;

const TEST_WORLD_ID: u32 = 10000000;

pub(super) fn cmd_debug_menu(context: &mut GameContext) -> CommandResult<()> {
    menu!(context,
          "Item test" => debug_item_test(context),
          "List entities" => debug_list_entities(context),
          "Goto world" => debug_goto_world(context),
          "Debug prefab" => debug_prefab(context),
          "Reload shaders" => debug_reload_shaders(),
          "Restart game" => debug_restart_game(context)
    )
}

fn debug_prefab(context: &mut GameContext) -> CommandResult<()> {
    let prefabs = prefab::get_prefab_names();
    let selected = menu_choice_indexed(context, prefabs)?;

    // Whip up a new testing world and port us there
    debug_regen_prefab(context, &selected);
    Ok(())
}

fn debug_list_entities(context: &mut GameContext) -> CommandResult<()> {
    let mut mes = String::new();
    {
        let world = &context.state.world;
        for e in world.entities() {
            let name = world.ecs().names.get(*e).map_or("(unknown)".to_string(), |n| n.name.clone());
            let pos = match world.position(*e) {
                Some(pos) => pos.to_string(),
                None      => "(unknown)".to_string(),
            };
            mes.push_str(&format!("[name: {}, pos: {}] ", name, pos));
        }
    }
    mes!(context.state.world, "{}", a=mes);
    Ok(())
}

fn get_debug_world(prefab: &str) -> World {
    World::new()
        .with_prefab(prefab)
        .with_randomized_seed()
        .with_id(TEST_WORLD_ID)
        .build()
}

fn debug_regen_prefab(context: &mut GameContext, prefab_name: &str) {
    goto_new_world(context, get_debug_world(prefab_name));
}

fn debug_goto_world(context: &mut GameContext) -> CommandResult<()> {
    let input = player_input(context, "Which id?").ok_or(CommandError::Cancel)?;

    let id = input.parse::<u32>()
        .map_err(|_| CommandError::Invalid("That's not a valid id."))?;

    let new_world = world::serial::load_world(id)
        .map_err(|_| CommandError::Invalid("That world doesn't exist."))?;

    goto_new_world(context, new_world);
    Ok(())
}

fn debug_item_test(context: &mut GameContext) -> CommandResult<()> {
    goto_new_world(context, get_debug_world("blank"));

    for pos in RectangleIter::new(Point::new(0, 0), Point::new(10, 10)) {
        if context.state.world.pos_loaded(&pos) {
            context.state.world.create(ecs::prefab::item("cola", "cola"), pos);
        }
    }

    Ok(())
}

fn goto_new_world(context: &mut GameContext, mut new_world: World) {
    let world = &mut context.state.world;

    let start_pos = match new_world.find_stairs_in() {
        Some(pos) => pos,
        None      => POINT_ZERO,
    };

    world.move_to_map(new_world, start_pos).unwrap();
}

fn debug_restart_game(context: &mut GameContext) -> CommandResult<()> {
    state::restart_game(context);
    Ok(())
}

fn debug_reload_shaders() -> CommandResult<()> {
    renderer::with_mut(|rc| rc.reload_shaders());
    Ok(())
}
