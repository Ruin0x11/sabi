use std::fmt;

use infinigen::ChunkedWorld;

use point::{Point, SquareIter};
use uuid::Uuid;
use world::{Bounds, World};
use world::traits::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Quest {
    pub assigner: Uuid,
    pub kind: QuestKind,
    pub location: QuestLocation,
    pub status: QuestStatus,
}

impl Quest {
    pub fn process(&mut self, world: &World) {
        if !self.in_progress() {
            return;
        }

        if self.is_finished(world) {
            mes!(world, "Finished quest!");
            self.status = QuestStatus::Finished;
        }
    }

    pub fn in_progress(&self) -> bool {
        self.status == QuestStatus::InProgress
    }

    pub fn is_finished(&self, world: &World) -> bool {
        match self.kind {
            QuestKind::Kill(_uuid) => false,
            QuestKind::Goto(radius) => {
                // check if in overworld
                if *world.terrain().bounds() != Bounds::Unbounded {
                    return false;
                }

                let player = match world.player() {
                    Some(p) => p,
                    None => return false,
                };

                let player_pos = world.position(player).unwrap();
                SquareIter::new(self.location.overworld_pos(), radius).any(|pos| pos == player_pos)
            },
        }
    }
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum QuestStatus {
    Finished,
    InProgress,
    NotYetStarted,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum QuestKind {
    Kill(Uuid),
    Goto(i32),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum QuestLocation {
    Overworld(Point),
}

impl QuestLocation {
    pub fn overworld_pos(&self) -> Point {
        match *self {
            QuestLocation::Overworld(pos) => pos,
        }
    }
}

impl fmt::Display for Quest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "You want the best, and you know you've got it. {} {}",
               self.kind,
               self.location)
    }
}

impl fmt::Display for QuestKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuestKind::Kill(uuid) => {
                write!(
                    f,
                    "There's this vile thing known as {} that's bothering me. \
Please kill it.",
                    uuid
                )
            },
            QuestKind::Goto(radius) => {
                write!(
                    f,
                    "I want someone to scout out the surrounding area. \
Get within {} places of the target area.",
                    radius
                )
            },
        }
    }
}

impl fmt::Display for QuestLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuestLocation::Overworld(pos) => write!(f, "The designated location is {}.", pos),
        }
    }
}

pub fn quests(npc: Uuid) -> Vec<Quest> {
    let mut quests = Vec::new();

    for _ in 0..20 {
        let data = Quest {
            assigner: npc,
            kind: QuestKind::Goto(8),
            location: QuestLocation::Overworld(Point::new(30, 40)),
            status: QuestStatus::InProgress,
        };
        quests.push(data);
    }

    quests
}
