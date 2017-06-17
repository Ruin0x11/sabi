use std::collections::HashSet;
use std::fmt;

use rand::{self, Rng};
use rand::distributions::{Weighted, WeightedChoice, IndependentSample};
use world::{MapId, World};
use world::traits::*;

fn prufer_to_edges(prufer: &[usize]) -> Vec<(usize, usize)> {
    let vertex_count = prufer.len() + 2;
    let mut vertex_set = Vec::with_capacity(vertex_count);

    for _ in 0..vertex_count {
        vertex_set.push(0);
    }

    let mut edges = Vec::new();

    // Initialize the array of vertex_count
    for i in 0..vertex_count {
        vertex_set[i] = 0;
    }

    // Number of occurrences of vertex in code
    for i in 0..vertex_count - 2 {
        vertex_set[prufer[i] - 1] += 1;
    }

    // Find the smallest label not present in prufer[].
    for i in 0..vertex_count - 2 {
        for j in 0..vertex_count {
            // If j+1 is not present in prufer set
            if vertex_set[j] == 0 {
                // Remove from Prufer set and print pair.
                vertex_set[j] = -1;
                let edge = (j + 1, prufer[i]);
                edges.push(edge);

                vertex_set[prufer[i] - 1] -= 1;

                break;
            }
        }
    }

    // For the last element
    let mut found = false;
    let mut first = 0;
    for i in 0..vertex_count {
        if vertex_set[i] == 0 {
            if !found {
                first = i + 1;
                found = true
            } else {
                let edge = (first, i + 1);
                edges.push(edge);
            }
        }
    }

    edges
}

struct DungeonFloor {
    kind: String,
    map_ids: Vec<Option<MapId>>,
}

impl DungeonFloor {
    pub fn new(kind: &str, length: usize) -> Self {
        assert!(length > 0);
        DungeonFloor {
            kind: kind.to_string(),
            map_ids: vec![None; length],
        }
    }

    pub fn exists(&self) -> bool {
        self.map_ids.first().unwrap().is_some()
    }

    pub fn generate(&mut self, current: &World) -> Option<World> {
        self.frontier().map(|idx| {
            let w = if !self.exists() {
                World::new()
                    .from_other_world(current)
                    .with_prefab(&self.kind)
                    .build()
                    .unwrap()
            } else {
                assert!(idx >= 1);
                assert!(&self.map_ids[idx].is_none());

                let frontier = &self.map_ids[idx - 1].unwrap();
                assert!(*frontier == current.flags().map_id);

                World::new()
                    .from_other_world(current)
                    .with_prefab(&self.kind)
                    .build()
                    .unwrap()
            };

            self.map_ids[idx] = Some(w.flags().map_id);

            w
        })
    }

    fn has_map(&self, target: MapId) -> bool {
        self.map_ids
            .iter()
            .any(|id| id.map_or(false, |i| i == target))
    }

    // The position of the next ungenerated dungeon map in this section
    fn frontier(&self) -> Option<usize> {
        println!("our ids: {:?}", self.map_ids);
        if !self.exists() {
            return Some(0);
        }

        self.map_ids.iter().position(|&id| id.is_none())
    }
}

pub struct Dungeon {
    adjacencies: Vec<HashSet<usize>>,
    floors: Vec<DungeonFloor>,
}

fn prune_adjacencies(
    adjacencies: &mut Vec<HashSet<usize>>,
    current: usize,
    visited: &mut HashSet<usize>,
) {
    for a in adjacencies[current].clone().iter() {
        if !visited.contains(a) {
            {
                let next = &mut adjacencies[*a];
                next.remove(&current);
            }
            {
                visited.insert(*a);
                prune_adjacencies(adjacencies, *a, visited);
            }
        }
    }
}

fn prufer_code(min_len: usize, max_len: usize, weight: u32, step: u32) -> Vec<usize> {
    let len = rand::thread_rng().gen_range(min_len, max_len + 1);
    let mut weights = vec![weight; len];
    let mut code = Vec::new();

    for _ in 0..len {
        let digit = choose_prufer_digit(&mut weights, step);
        code.push(digit);
    }

    code
}

fn choose_prufer_digit(weights: &mut Vec<u32>, step: u32) -> usize {
    let mut choices: Vec<Weighted<usize>> =
        weights.iter()
               .enumerate()
               .map(|(i, &w)| {
                        Weighted {
                            weight: w,
                            item: i + 1,
                        }
                    })
               .collect();

    let wc = WeightedChoice::new(&mut choices);
    let mut rng = rand::thread_rng();

    let res = wc.ind_sample(&mut rng);

    weights[res - 1] += step;

    res
}

pub struct DungeonPlan {
    prufer_code: Vec<usize>,
    min_floor_length: usize,
    max_floor_length: usize,
    kind: String,
}

impl DungeonPlan {
    pub fn new(prufer_code: Vec<usize>, len: usize, kind: String) -> Self {
        DungeonPlan {
            prufer_code: prufer_code,
            min_floor_length: len,
            max_floor_length: len + 1,
            kind: kind,
        }
    }

    pub fn easy() -> Self {
        DungeonPlan {
            prufer_code: prufer_code(2, 4, 20, 10),
            min_floor_length: 2,
            max_floor_length: 5,
            kind: "blank".to_string(),
        }
    }

    pub fn build(&self) -> Dungeon {
        let vertex_count = self.prufer_code.len() + 2;

        let mut adjacencies = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            adjacencies.push(HashSet::new());
        }

        let mut floors = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            floors.push(generate_floor(self.min_floor_length, self.max_floor_length, &self.kind));
        }

        let edges = prufer_to_edges(&self.prufer_code);
        for (to_a, from_a) in edges.into_iter() {
            let (to, from) = (to_a - 1, from_a - 1);
            adjacencies[from].insert(to);
            adjacencies[to].insert(from);
        }

        // DFS to make into directed graph with root 0
        prune_adjacencies(&mut adjacencies, 0, &mut HashSet::new());

        Dungeon {
            adjacencies: adjacencies,
            floors: floors,
        }
    }
}

fn generate_floor(min_len: usize, max_len: usize, kind: &str) -> DungeonFloor {
    DungeonFloor::new(kind, rand::thread_rng().gen_range(min_len, max_len))
}

impl Dungeon {
    /// Generate the next floor of this dungeon, given the world representing the current dungeon
    /// floor.
    pub fn generate(&mut self, current: &World) -> Option<World> {
        let floor = self.floor_for_map_id(current.flags().map_id);
        floor.generate(current)

    }

    fn floor_for_map_id(&mut self, id: MapId) -> &mut DungeonFloor {
        // to get around borrowing
        let found_floor = self.floors
                              .iter_mut()
                              .find(|floor| floor.has_map(id))
                              .is_some();
        if !found_floor {
            return self.floors.first_mut().unwrap();
        }
        self.floors
            .iter_mut()
            .find(|floor| floor.has_map(id))
            .unwrap()
    }

    pub fn branches(&self, idx: usize) -> &HashSet<usize> {
        &self.adjacencies[idx]
    }

    pub fn floors(&self) -> usize {
        self.adjacencies.len()
    }
}

impl fmt::Display for Dungeon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, floor) in self.floors.iter().enumerate() {
            writeln!(f,
                     "Floor: {} ({}) len: {} children: {:?}",
                     i,
                     floor.kind,
                     floor.map_ids.len(),
                     self.adjacencies[i])?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use point::POINT_ZERO;
    use testing::*;

    fn hashset(items: &[usize]) -> HashSet<usize> {
        let mut hs = HashSet::new();
        for item in items {
            hs.insert(*item);
        }
        hs
    }

    #[test]
    fn test_branches() {
        let dungeon = DungeonPlan::new(vec![3, 3, 3, 4], 1, "blank".to_string()).build();

        assert_eq!(dungeon.floors(), 6);
        assert_eq!(dungeon.branches(0), &hashset(&[2]));
        assert_eq!(dungeon.branches(1), &hashset(&[]));
        assert_eq!(dungeon.branches(2), &hashset(&[1, 3, 4]));
        assert_eq!(dungeon.branches(3), &hashset(&[5]));
        assert_eq!(dungeon.branches(4), &hashset(&[]));
        assert_eq!(dungeon.branches(5), &hashset(&[]));
    }

    #[test]
    fn test_generate_dungeon() {
        let mut context = test_context();
        let world = &mut context.state.world;

        let floor = DungeonFloor::new("blank", 2);
        let mut dungeon = Dungeon {
            adjacencies: vec![HashSet::new()],
            floors: vec![floor],
        };

        let floor_a = dungeon.generate(world);
        assert!(floor_a.is_some());
        world.move_to_map(floor_a.unwrap(), POINT_ZERO).unwrap();

        let floor_b = dungeon.generate(world);
        assert!(floor_b.is_some());
        world.move_to_map(floor_b.unwrap(), POINT_ZERO).unwrap();

        let floor_c = dungeon.generate(world);
        assert!(floor_c.is_none());
    }
}
