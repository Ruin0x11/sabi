use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::f32;

use data::Walkability;
use world::traits::{Query, WorldQuery};
use point::Point;
use world::EcsWorld;

const CALCULATION_LIMIT: u32 = 50;

#[derive(Debug)]
pub struct Path {
    path: Vec<Point>,
}

impl Path {
    fn neighbors(current: Point,
                 world: &EcsWorld,
                 walkability: Walkability) -> Vec<Point> {
        assert!(world.pos_valid(&current));
        let nearby_points: [Point; 9] = [
            (-1, -1).into(),
            (-1,  0).into(),
            (-1,  1).into(),
            ( 0, -1).into(),
            ( 0,  0).into(),
            ( 0,  1).into(),
            ( 1, -1).into(),
            ( 1,  0).into(),
            ( 1,  1).into(),
        ];

        nearby_points.clone().iter()
            .map(|&d| current + d)
            .filter(|&point|
                    world.pos_valid(&point)
                    && world.can_walk(point, walkability))
            .collect::<Vec<_>>()
    }

    fn search_heuristic(destination: Point, next: Point) -> f32 {
        ((destination.x - next.x).abs() + (destination.y - next.y).abs()) as f32
    }

    fn cost_heuristic(current: Point, next: Point) -> f32 {
        assert!((current.x - next.x).abs() <= 1);
        assert!((current.y - next.y).abs() <= 1);
        1.0
    }

    fn create_path(from: Point, to: Point, came_from: HashMap<Point, Option<Point>>) -> Path {
        let mut current = to;
        let mut path_buffer = vec![current];
        while current != from {
            match came_from.get(&current) {
                Some(&Some(new_current)) => {
                    current = new_current;
                    if current != from {
                        path_buffer.push(current);
                    }
                }
                Some(&None) => panic!(
                    "Every point except for the initial one (`from`) one should be some."),
                None => {
                    path_buffer = vec![];
                    break
                },
            }
        }

        assert_eq!(None, path_buffer.iter().find(|&&p| p == from));

        Path {
            path: path_buffer,
        }
    }

    pub fn find(from: Point, to: Point, world: &EcsWorld, walkability: Walkability) -> Self {
        if from == to {
            return Path { path: vec![] };
        }

        // If we set walkability to monsters blocking and try to pathfind to a
        // monster, it counts the destination as unwalkable. In that case, just
        // stop when right next to the destination.
        let stop_when_neighboring = walkability == Walkability::MonstersBlocking
            && world.mob_at(to).is_some();

        if !stop_when_neighboring && !world.can_walk(to, walkability) {
            return Path { path: vec![] };
        }

        if from.tile_distance(to) == 1 {
            return Path { path: vec![to] };
        }

        let mut frontier = BinaryHeap::new();
        frontier.push(State { position: from, cost: 0.0 });
        let mut came_from = HashMap::new();
        let mut cost_so_far = HashMap::new();

        came_from.insert(from, None);
        cost_so_far.insert(from, 0.0);

        // NOTE: the map is effectively infinite. We need to limit the
        // calculations or the algorithm will try to explore the
        // entire world before it decides that no path exists.
        let mut calculation_steps = 0;

        while let Some(current) = frontier.pop() {
            if stop_when_neighboring && Point::is_next_to(&current.position, to) {
                came_from.insert(to, Some(current.position));
                break
            } else if current.position == to {
                break
            }

            if calculation_steps >= CALCULATION_LIMIT {
                break
            } else {
                calculation_steps += 1;
            }
            let neigh = Path::neighbors(current.position, world, walkability);

            for &next in neigh.iter() {
                // Make this an in-game property toggle?
                // world.draw_calls.push(Draw::Point(next.x, next.y));

                let new_cost = cost_so_far[&current.position] + Path::cost_heuristic(current.position, next);
                let val = cost_so_far.entry(next).or_insert(f32::MAX);
                if new_cost < *val {
                    *val = new_cost;
                    let priority = new_cost + Path::search_heuristic(to, next);
                    frontier.push(State { position: next, cost: priority });
                    came_from.insert(next, Some(current.position));
                }
            }
        }

        // if calculation_steps >= calculation_limit {
        //     println!("Pathfinding calculation exceeded the limit.");
        // } else {
        //     println!("Pathfinding finished in {} calculation steps.", calculation_steps);
        // }

        Path::create_path(from, to, came_from)
    }

    /// The number of steps to necessary to reach the destination. If
    /// no path was found, it is `0`.
    pub fn len(&self) -> usize {
        self.path.len()
    }
}

impl Iterator for Path {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.path.pop()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct State {
    cost: f32,
    position: Point,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        assert!(self.cost.is_finite());
        assert!(other.cost.is_finite());
        if other.cost > self.cost {
            Ordering::Greater
        } else if other.cost < self.cost {
            Ordering::Less
        } else if other.cost == self.cost {
            Ordering::Equal
        } else {
            unreachable!()
        }
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(never)]
#[cfg(test)]
mod test {
    use super::Path;
    use ecs::EcsWorld;
    use world::traits::*;
    use point::{Point, POINT_ZERO};
    use data::Walkability;
    use testbed::make_grid_from_str;
    use tile::{self, Tile};
    use tile::TileType::{Wall, Floor};

    struct Board {
        start: Point,
        destination: Point,
        level: EcsWorld,
    }

    fn make_board(text: &str) -> Board {
        let start = Point{x: 0, y: 0};
        let destination = Point{x: 0, y: 0};

        let lines = text.split('\n').filter(|l| l.len() > 0).collect::<Vec<_>>();
        let height = lines.len();
        assert!(height > 0);
        let width = lines[0].len();
        assert!(width > 0);
        assert!(lines.iter().all(|line| line.chars().count() == width));

        let level = EcsWorld::generate(EcsWorldType::Instanced(Point::new(width as i32, height as i32)), 128, tile::WALL);

        Board {
            start: start,
            destination: destination,
            level: level,
        }
    }

    fn get_board(board: &str) -> Board {
        let callback = |pt: &Point, c: char, board: &mut Board| {
                if c == 's' {
                    board.start = pt.clone();
                }

                if c == 'd' {
                    board.destination = pt.clone();
                }

                let tile_kind = match c {
                    '.' => Floor,
                    '*' => Floor,
                    's' => Floor,
                    'd' => Floor,
                    'x' => Wall,
                    _   => unreachable!(),
                };
                board.level.set_tile(pt.clone(), Tile {
                    type_: tile_kind,
                    feature: None,
                });
        };
        let make = |dim: Point| {
            let world = EcsWorld::generate(EcsWorldType::Instanced(Point::new(dim.x,
                    dim.y)), 128, tile::WALL);
            Board {
                start: POINT_ZERO,
                destination: POINT_ZERO,
                level: world,
            }
        };
        make_grid_from_str(board, make, callback)
    }

    fn test_harness(board: &str, expected_len: usize, expected_path: &[(i32, i32)]) {
        let board = get_board(board);
        let path: Path = Path::find(board.start, board.destination, &board.level,
                                    Walkability::MonstersWalkable);
        assert_eq!(expected_len, path.len());
        let expected = expected_path.iter()
            .cloned().map(Into::into).collect::<Vec<Point>>();
        assert_eq!(expected, path.collect::<Vec<_>>());
    }

    #[test]
    fn test_neighbor() {
        test_harness("
...........
.sd........
...........
...........
", 1, &[(2, 1)]);
    }

    #[test]
    fn test_start_and_destination_identical() {
        let mut board = make_board("
...........
.s.........
...........
...........
");
        board.destination = board.start;
        let path: Path = Path::find(board.start, board.destination, &board.level,
                                    Walkability::MonstersWalkable);
        assert_eq!(0, path.len());
        let expected: Vec<Point> = vec![];
        assert_eq!(expected, path.collect::<Vec<_>>());
    }

    #[test]
    fn test_straight_path() {
        test_harness("
...........
.s******d..
...........
...........
", 7, &[(2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), (8, 1)]);
    }

    #[test]
    fn test_diagonal_path() {
        test_harness("
s..........
.*.........
..*........
...d.......
", 3, &[(1, 1), (2, 2), (3, 3)]);
    }

    #[test]
    fn test_no_path() {
        test_harness("
....x......
.s..x...d..
....x......
....x......
", 0, &[]);
    }

    #[test]
    fn test_line_obstacle() {
        test_harness("
....x......
.s..x......
..*.x......
...*****d..
", 7, &[(2, 2), (3, 3), (4, 3), (5, 3), (6, 3), (7, 3), (8, 3)]);
    }

    #[test]
    fn test_concave_obstacle() {
        test_harness("
......x....
.s....xd...
..*...x*...
..*xxxx*...
...****....
", 9, &[(2, 2), (2, 3), (3, 4), (4, 4), (5, 4), (6, 4), (7, 3), (7, 2), (7, 1)]);
    }
}
