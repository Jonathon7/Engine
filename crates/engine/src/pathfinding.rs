use std::collections::{BinaryHeap, HashMap};
use std::cmp::{Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Point {
    x: i32,
    y: i32
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Node {
    point: Point,
    f_cost: u32
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost.cmp(&self.f_cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn manhattan_distance(a: Point, b: Point) -> u32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs()) as u32
}

fn neighbors(point: Point, grid: &Vec<Vec<bool>>) -> Vec<Point> {
    let rows = grid.len() as i32;
    let columns = grid[0].len() as i32;
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

    directions.iter()
        .map(|(dx, dy)| Point { x: point.x + dx, y: point.y + dy })
        .filter(|p| {
            p.x > 0 && p.y > 0 &&
            p.x < rows && p.y < columns &&  
            grid[p.x as usize][p.y as usize]
        })
        .collect()
}

fn astar(grid: Vec<Vec<bool>>, start: Point, goal: Point) -> Option<Vec<Point>> {
    let mut open = BinaryHeap::new();
    let mut g_cost: HashMap<Point, u32> = HashMap::new();
    let mut came_from: HashMap<Point, Point> = HashMap::new();
    
    g_cost.insert(start, 0);
    open.push(Node { point: start, f_cost: manhattan_distance(start, goal) });
    // implement main loop, popping open to get smallest f score Node
    while let Some(Node { point, .. }) = open.pop() {

    }
    // implement path reconstruction when goal is reached
    //
    // implement g score calculation and neighbor exploration
}

