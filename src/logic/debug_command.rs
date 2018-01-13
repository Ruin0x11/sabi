use rand::{self, Rng};

use GameContext;
use ai::{self, Ai, AiKind};
use debug;
use ecs;
use ecs::traits::*;
use logic::entity::*;
use point::{Point, RectangleIter, POINT_ZERO};
use prefab::{self, PrefabArgs};
use renderer;
use state;
use world::traits::*;
use world::{self, World};

use super::command::*;

const TEST_WORLD_ID: u32 = 10000000;

pub(super) fn cmd_debug_menu(context: &mut GameContext) -> CommandResult<()> {
    menu!(context,
          "Reload AI"      => debug_reload_ai(context),
          "omniscience"    => debug_omniscience(context),
          "AI"             => debug_ai_menu(context),
          "follow entity"  => debug_follow_entity(context),
          "Item test"      => debug_item_test(context),
          "List entities"  => debug_list_entities(context),
          "Place enemies"  => debug_place_enemies(context),
          "Goto world"     => debug_goto_world(context),
          "Debug prefab"   => debug_prefab(context),
          "Deploy prefab"  => debug_deploy_prefab(context),
          "Reload shaders" => debug_reload_shaders(),
          "Restart game"   => debug_restart_game(context)
    )
}

fn debug_omniscience(context: &mut GameContext) -> CommandResult<()> {
    let player = context.state.world.player().unwrap();
    let props = context.state.world.ecs_mut().props.get_mut_or_err(player);
    props.props.set::<bool>("omniscient", true);
    Ok(())
}

fn debug_reload_ai(context: &mut GameContext) -> CommandResult<()> {
    ai::reload_planner();
    mes!(context.state.world, "Reloaded AI.");
    Ok(())
}

fn debug_follow_entity(context: &mut GameContext) -> CommandResult<()> {
    mes!(context.state.world, "Which?");
    let pos = select_tile(context, |pos, world| {
        debug::follow_entity(world.mob_at(pos));
        debug::update(world);
    })?;
    debug::follow_entity(context.state.world.mob_at(pos));
    Ok(())
}

fn debug_ai_menu(context: &mut GameContext) -> CommandResult<()> {
    menu!(context,
          "scav"           => debug_scav(context),
          "guard"          => debug_guard(context),
          "loadout"        => debug_loadout(context),
          "pet"            => debug_pet(context),
          "throw"          => debug_throw(context)
    )
}

fn make_pet(context: &mut GameContext, pos: Point) {
    let world = &mut context.state.world;

    let e = world.create(ecs::load_mob("putit").unwrap().c(Ai::new(AiKind::Follow)), pos)
                 .unwrap();
    let uuid = e.uuid(world).unwrap();
    world.flags_mut().globals.party.add_member(uuid);
}

fn debug_pet(context: &mut GameContext) -> CommandResult<()> {
    for pos in select_rect(context)? {
        make_pet(context, pos);
    }

    Ok(())
}

fn debug_loadout(context: &mut GameContext) -> CommandResult<()> {
    goto_new_world(context,
                   get_debug_world("blank", Some(prefab_args! { width: 100, height: 100, }))
                       .unwrap());

    context.state
           .world
           .create(ecs::load_mob("putit").unwrap(), Point::new(1, 1));

    Ok(())
}

use ecs::traits::ComponentQuery;

// TODO: Make into prefab
fn debug_scav(context: &mut GameContext) -> CommandResult<()> {
    goto_new_world(context,
                   get_debug_world("blank", Some(prefab_args! { width: 100, height: 100, }))
                       .unwrap());

    for pos in RectangleIter::new(Point::new(0, 0), Point::new(100, 100)) {
        if rand::thread_rng().next_f32() < 0.1 {
            context.state
                   .world
                   .create(ecs::prefab::item("cola", "cola"), pos);
        }
    }

    context.state
           .world
           .create(ecs::prefab::mob("putit", 100, "putit").c(Ai::new(AiKind::Scavenge)),
                   Point::new(1, 1));
    context.state
           .world
           .create(ecs::prefab::mob("putit", 100, "putit").c(Ai::new(AiKind::Scavenge)),
                   Point::new(1, 2));
    context.state
           .world
           .create(ecs::prefab::mob("putit", 100, "putit").c(Ai::new(AiKind::Scavenge)),
                   Point::new(2, 1));
    context.state
           .world
           .create(ecs::prefab::mob("putit", 100, "putit").c(Ai::new(AiKind::Scavenge)),
                   Point::new(2, 2));

    Ok(())
}

fn debug_guard(context: &mut GameContext) -> CommandResult<()> {
    goto_new_world(context,
                   get_debug_world("blank", Some(prefab_args! { width: 10, height: 30, }))
                       .unwrap());

    context.state
           .world
           .create(ecs::prefab::mob("putit", 100, "putit").c(Ai::new(AiKind::SeekTarget)),
                   Point::new(5, 20));
    context.state
           .world
           .create(ecs::prefab::mob("guard", 1000, "npc").c(Ai::new(AiKind::Guard)),
                   Point::new(1, 1));

    Ok(())
}


fn debug_throw(context: &mut GameContext) -> CommandResult<()> {
    goto_new_world(context, get_debug_world("blank", None).unwrap());

    context.state
           .world
           .create(ecs::prefab::item("watermelon", "watermelon"), Point::new(5, 5));

    context.state
           .world
           .create(ecs::prefab::mob("putit", 1000, "putit"), Point::new(9, 9));

    Ok(())
}

fn debug_item_test(context: &mut GameContext) -> CommandResult<()> {
    goto_new_world(context, get_debug_world("blank", None).unwrap());

    for pos in RectangleIter::new(Point::new(0, 0), Point::new(3, 3)) {
        context.state
               .world
               .create(ecs::prefab::item("cola", "cola"), pos);
    }

    context.state
           .world
           .create(ecs::prefab::mob("putit", 100, "putit"), Point::new(5, 5));

    Ok(())
}

fn debug_prefab(context: &mut GameContext) -> CommandResult<()> {
    let selected = choose_prefab(context)?;

    debug_regen_prefab(context, &selected)
}

fn choose_prefab(context: &mut GameContext) -> CommandResult<String> {
    let prefabs = prefab::get_prefab_names();
    menu_choice_indexed(context, prefabs)
}

fn debug_deploy_prefab(context: &mut GameContext) -> CommandResult<()> {
    mes!(context.state.world, "Which one to deploy?");
    let selected_name = choose_prefab(context)?;

    let prefab = prefab::create(&selected_name, &None)
        .map_err(|e| CommandError::Debug(format!("Failed to make prefab: {}", e)))?;

    mes!(context.state.world, "Where to deploy?");
    let pos = select_tile(context, |_, _| ())?;

    context.state.world.deploy_prefab(prefab, pos);
    context.state.world.add_marker_overlays();
    Ok(())
}

fn debug_list_entities(context: &mut GameContext) -> CommandResult<()> {
    let mut mes = String::new();
    {
        let world = &context.state.world;
        for e in world.entities() {
            let name = world.ecs()
                            .names
                            .get(*e)
                            .map_or("(unknown)".to_string(), |n| n.name.clone());
            let pos = match world.position(*e) {
                Some(pos) => pos.to_string(),
                None => "(unknown)".to_string(),
            };
            mes.push_str(&format!("[name: {}, pos: {}] ", name, pos));
        }
    }
    mes!(context.state.world, "{}", mes);
    Ok(())
}

fn select_rect(context: &mut GameContext) -> CommandResult<RectangleIter> {
    mes!(context.state.world, "First corner?");
    let upper_left = select_tile(context, |_, _| ())?;

    mes!(context.state.world, "Second corner?");
    let lower_right = select_tile(context, |_, _| ())?;

    if lower_right.x < upper_left.x || lower_right.y < upper_left.y {
        return Err(CommandError::Cancel);
    }

    let size = lower_right - upper_left;
    Ok(RectangleIter::new(upper_left, size))
}

fn debug_place_enemies(context: &mut GameContext) -> CommandResult<()> {
    for pos in select_rect(context)? {
        context.state
               .world
               .create(ecs::prefab::mob("putit", 50, "putit"), pos);
    }

    Ok(())
}

fn get_debug_world(prefab: &str, args: Option<PrefabArgs>) -> Result<World, String> {
    match args {
        Some(a) => {
            World::new()
                .with_prefab(prefab)
                .with_randomized_seed()
                .with_id(TEST_WORLD_ID)
                .with_prefab_args(a)
                .build()
        },
        None => {
            World::new()
                .with_prefab(prefab)
                .with_randomized_seed()
                .with_id(TEST_WORLD_ID)
                .build()
        },
    }
}

fn debug_regen_prefab(context: &mut GameContext, prefab_name: &str) -> CommandResult<()> {
    let world = get_debug_world(prefab_name, None)
        .map_err(|e| CommandError::Debug(format!("Failed to make world: {}", e)))?;
    goto_new_world(context, world);
    Ok(())
}

fn debug_goto_world(context: &mut GameContext) -> CommandResult<()> {
    let input = player_input(context, "Which id?")
        .ok_or(CommandError::Cancel)?;

    let id = input.parse::<u32>()
                  .map_err(|_| CommandError::Invalid("That's not a valid id."))?;

    let new_world = world::serial::load_world(id)
        .map_err(|_| CommandError::Invalid("That world doesn't exist."))?;

    goto_new_world(context, new_world);
    Ok(())
}

fn goto_new_world(context: &mut GameContext, mut new_world: World) {
    let world = &mut context.state.world;

    let start_pos = match new_world.find_stairs_in() {
        Some(pos) => pos,
        None => POINT_ZERO,
    };

    world.move_to_map(new_world, start_pos).unwrap();
    world.add_marker_overlays();
}

fn debug_restart_game(context: &mut GameContext) -> CommandResult<()> {
    state::restart_game(context);
    Ok(())
}

fn debug_reload_shaders() -> CommandResult<()> {
    renderer::with_mut(|rc| rc.reload_shaders());
    Ok(())
}
