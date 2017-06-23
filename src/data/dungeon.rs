use std::collections::HashSet;
use std::fmt;

use rand::{self, Rng};
use rand::distributions::{Weighted, WeightedChoice, IndependentSample};
use world::{MapId, World};
use world::traits::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dungeon {
    adjacencies: Vec<HashSet<usize>>,
    sections: Vec<DungeonSection>,
}

impl Dungeon {
    /// Generate the next floor of this dungeon, given the world representing the current dungeon
    /// floor.
    pub fn generate_next_floor(&mut self, current_floor: &World) -> Option<World> {
        let section = self.section_for_map_id_mut(current_floor.flags().map_id);
        section.generate_next_floor(current_floor)
    }

    pub fn generate_branch(&mut self, current_floor: &World, branch: usize) -> Option<World> {
        assert!(self.is_branch_point(current_floor.flags().map_id));
        assert!(branch < self.sections.len());
        assert!(!self.sections[branch].exists());

        let section = &mut self.sections[branch];
        section.generate_next_floor(current_floor)
    }

    pub fn has_floor(&self, id: MapId) -> bool {
        self.sections.iter().any(|s| s.has_floor(id))
    }

    fn map_id_to_section_index(&self, map_id: MapId) -> Option<usize> {
        self.sections.iter().position(|s| s.has_floor(map_id))
    }

    fn section_for_map_id(&self, id: MapId) -> &DungeonSection {
        let found_section = self.sections.iter().find(|section| section.has_floor(id));
        if !found_section.is_some() {
            if !self.exists() {
                // Dungeon hasn't been entered before, so start off in the first section.
                return self.sections.first().unwrap();
            } else {
                panic!("Map id {} doesn't exist in dungeon!", id);
            }
        }

        found_section.unwrap()
    }

    fn section_for_map_id_mut(&mut self, id: MapId) -> &mut DungeonSection {
        // to get around borrowing
        let found_section = self.sections
                                .iter_mut()
                                .find(|section| section.has_floor(id))
                                .is_some();
        if !found_section {
            if !self.exists() {
                // Dungeon hasn't been entered before, so start off in the first section.
                return self.sections.first_mut().unwrap();
            } else {
                panic!("Map id {} doesn't exist in dungeon!", id);
            }
        }
        self.sections
            .iter_mut()
            .find(|section| section.has_floor(id))
            .unwrap()
    }

    pub fn is_leaf(&self, id: MapId) -> bool {
        self.map_id_to_section_index(id)
            .map_or(true, |i| self.adjacencies[i].is_empty())
    }

    pub fn is_branch_point(&self, id: MapId) -> bool {
        self.section_for_map_id(id).is_branch_point(id)
    }

    pub fn branches(&self, id: MapId) -> Option<&HashSet<usize>> {
        self.map_id_to_section_index(id)
            .map(|i| &self.adjacencies[i])
    }

    pub fn exists(&self) -> bool {
        self.sections.first().unwrap().exists()
    }

    pub fn sections(&self) -> usize {
        self.adjacencies.len()
    }
}

fn prune_adjacencies(adjacencies: &mut Vec<HashSet<usize>>,
                     current_section: usize,
                     visited: &mut HashSet<usize>) {
    for section in adjacencies[current_section].clone() {
        if !visited.contains(&section) {
            {
                let next = &mut adjacencies[section];
                next.remove(&current_section);
            }

            visited.insert(section);
            prune_adjacencies(adjacencies, section, visited);
        }
    }
}

impl fmt::Display for Dungeon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, section) in self.sections.iter().enumerate() {
            writeln!(f,
                     "section: {} ({}) len: {} children: {:?}",
                     i,
                     section.kind,
                     section.floor_ids.len(),
                     self.adjacencies[i])?;
        }
        Ok(())
    }
}

pub struct DungeonPlan {
    prufer_code: Vec<usize>,
    min_section_length: usize,
    max_section_length: usize,
    kind: String,
}

impl DungeonPlan {
    pub fn new(prufer_code: Vec<usize>, len: usize, kind: String) -> Self {
        DungeonPlan {
            prufer_code: prufer_code,
            min_section_length: len,
            max_section_length: len + 1,
            kind: kind,
        }
    }

    pub fn easy() -> Self {
        DungeonPlan {
            prufer_code: generate_prufer_code(2, 4, 20, 10),
            min_section_length: 2,
            max_section_length: 5,
            kind: "dood".to_string(),
        }
    }

    pub fn build(&self) -> Dungeon {
        let vertex_count = self.prufer_code.len() + 2;

        let mut adjacencies = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            adjacencies.push(HashSet::new());
        }

        let mut sections = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            sections.push(generate_section(self.min_section_length,
                                           self.max_section_length,
                                           &self.kind));
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
            sections: sections,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct DungeonSection {
    kind: String,
    floor_ids: Vec<Option<MapId>>,
}

impl DungeonSection {
    pub fn new(kind: &str, length: usize) -> Self {
        assert!(length > 0);
        DungeonSection {
            kind: kind.to_string(),
            floor_ids: vec![None; length],
        }
    }

    pub fn exists(&self) -> bool {
        self.floor_ids.first().unwrap().is_some()
    }

    pub fn generate_next_floor(&mut self, current_floor: &World) -> Option<World> {
        self.next_ungenerated_floor_idx().and_then(|idx| {
            let world = World::new()
                .from_other_world(current_floor)
                .with_prefab(&self.kind)
                .build()
                .unwrap();

            if self.exists() {
                // Do a sanity check to ensure we're directly a floor above the world to be
                // generated
                let floor_above_ungenerated = self.floor_ids[idx - 1].unwrap();
                if floor_above_ungenerated != current_floor.flags().map_id {
                    return None;
                }
            };

            self.floor_ids[idx] = Some(world.flags().map_id);

            Some(world)
        })
    }

    fn has_floor(&self, target: MapId) -> bool {
        self.floor_ids
            .iter()
            .any(|id| id.map_or(false, |i| i == target))
    }

    fn is_branch_point(&self, target: MapId) -> bool {
        self.floor_ids
            .last()
            .unwrap()
            .map_or(false, |i| i == target)
    }

    fn next_ungenerated_floor_idx(&self) -> Option<usize> {
        self.floor_ids.iter().position(|&id| id.is_none())
    }
}

fn generate_section(min_len: usize, max_len: usize, kind: &str) -> DungeonSection {
    DungeonSection::new(kind, rand::thread_rng().gen_range(min_len, max_len))
}

fn generate_prufer_code(min_len: usize, max_len: usize, weight: u32, step: u32) -> Vec<usize> {
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

        assert_eq!(dungeon.sections(), 6);
        assert_eq!(dungeon.adjacencies[0], hashset(&[2]));
        assert_eq!(dungeon.adjacencies[1], hashset(&[]));
        assert_eq!(dungeon.adjacencies[2], hashset(&[1, 3, 4]));
        assert_eq!(dungeon.adjacencies[3], hashset(&[5]));
        assert_eq!(dungeon.adjacencies[4], hashset(&[]));
        assert_eq!(dungeon.adjacencies[5], hashset(&[]));
    }

    #[test]
    fn test_generate_dungeon() {
        let mut context = test_context();
        let world = &mut context.state.world;

        let section = DungeonSection::new("blank", 2);
        let mut dungeon = Dungeon {
            adjacencies: vec![HashSet::new()],
            sections: vec![section],
        };

        let floor_a = dungeon.generate_next_floor(world);
        assert!(floor_a.is_some());
        world.move_to_map(floor_a.unwrap(), POINT_ZERO).unwrap();

        let floor_b = dungeon.generate_next_floor(world);
        assert!(floor_b.is_some());
        world.move_to_map(floor_b.unwrap(), POINT_ZERO).unwrap();

        let floor_c = dungeon.generate_next_floor(world);
        assert!(floor_c.is_none());
    }
}
