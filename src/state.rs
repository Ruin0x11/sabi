use action::*;
use actor::*;
use glyph;
use keys::*;
use point::Point;
use world::World;
use engine::Canvas;
use ::GameContext;

pub struct GameState {
    world: Option<World>,
    player: Option<Actor> // FIXME: Does there always have to be a player?
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            world: None,
            player: None,
        }
    }

    pub fn set_world(&mut self, world: World) {
        self.world = Some(world);
    }

    pub fn set_player(&mut self, player: Actor) {
        self.player = Some(player);
    }
}

// IMPLEMENT
#[cfg(never)]
pub fn process_actors(context: &GameContext) {
    if let Some(ref mut world) = context.state.world {
        for actor in world.actors() {

        }
    }
}

pub enum Command {
    Move(Direction),
    Wait,
    Quit,
}

fn process_world(world: &mut Option<World>, canvas: &mut Canvas) {
    // TEMP: renders the world!
    if let Some(ref mut world) = *world {
        world.with_cells(Point::new(0, 0), Point::new(128, 128),
                         |point, ref cell| {
                             canvas.print_glyph(point.x, point.y, cell.tile.glyph.clone())
                         });
    }
}

fn get_command_for_key(context: &GameContext, key: Key) -> Command {
    if key.code == KeyCode::NoneKey {
        warn!(context.logger, "NoneKey was returned");
    }
    debug!(context.logger, "Key: {:?}", key);
    match key {
        Key { code: KeyCode::Esc,   .. } => Command::Quit,
        Key { code: KeyCode::Left,  .. } => Command::Move(Direction::W),
        Key { code: KeyCode::Right, .. } => Command::Move(Direction::E),
        Key { code: KeyCode::Up,    .. } => Command::Move(Direction::N),
        Key { code: KeyCode::Down,  .. } => Command::Move(Direction::S),
        _                                => Command::Wait
    }
}

pub fn get_commands_from_input(context: &mut GameContext) -> Vec<Command> {

    let mut commands = Vec::new();

    let new_keys = context.canvas.get_input();
    context.keys.extend(new_keys);

    while let Some(key) = context.keys.pop() {
        commands.push(get_command_for_key(context, key));
    }
    commands
}

pub fn process_player_commands(context: &mut GameContext) {
    let commands = get_commands_from_input(context);

    if let Some(first) = commands.iter().next() {
        process_player_command(first,
                               &mut context.state.player,
                               &mut context.canvas,
                               &mut context.state.world);
    }
}

fn process_player_command(command: &Command,
                          player: &mut Option<Actor>,
                          canvas: &mut Canvas,
                          world: &mut Option<World>) {
    if let Some(ref mut player_) = *player {
        if let Some(ref mut world_) = *world {
            match *command {
                // TEMP: Commands can still be run even if there is no player?
                Command::Move(dir) => player_.run_action(Action::Move(dir), world_),
                Command::Quit      => canvas.close_window(),
                _                  => ()
            }
        }
    } else {
        panic!("There should be a player by now...");
    }
}

fn render_player(context: &mut GameContext) {
    if let Some(ref mut player) = context.state.player {
        let pos = player.get_pos();
        // TEMP: renders the player!
        context.canvas.print_glyph(pos.x, pos.y, glyph::Glyph::Player);
        debug!(context.logger, "Prayer pos: {}", pos);
        if let Some(ref world) = context.state.world {
            if let Some(cell) = world.cell(pos)  {
                debug!(context.logger, "Tile: {:?}", cell.tile);
            }
        }
    }
}

fn process_player(context: &mut GameContext) {
    process_player_commands(context);
}

// TEMP: Just to bootstrap things dirtily.
pub fn process(context: &mut GameContext) {
    context.canvas.clear();

    // TEMP: Nothing to do with speed here!
    process_world(&mut context.state.world, &mut context.canvas);
    render_player(context);

    context.canvas.present();

    process_player(context);
}
