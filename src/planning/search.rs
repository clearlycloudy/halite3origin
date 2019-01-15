
use rand::Rng;

use std::collections::{BinaryHeap,HashSet,HashMap};
use std::cmp::Ordering;

use mapping::mapraw::{ResourceMap, RawMaps, UnitMap, DropoffMap, Unit};
use common::agent::{Agent,AgentStatus};
use common::coord::{Coord,Dir};
use common::player::{Player,PlayerStats};

struct CoordCost((Coord,usize));

impl Ord for CoordCost {
    fn cmp( & self, other: &CoordCost ) -> Ordering {
        (other.0).1.cmp( &(self.0).1 )
    }
}

impl PartialOrd for CoordCost {
    fn partial_cmp( &self, other: &CoordCost ) -> Option<Ordering> {
        Some( self.cmp(other) )
    }
}

impl PartialEq for CoordCost {
    fn eq( & self, other: &CoordCost ) -> bool {
        (other.0).1.cmp( &(self.0).1 ) == Ordering::Equal
    }
}

impl Eq for CoordCost {}

fn get_unexplored( current: Coord,
                   explored: &HashSet<Coord>,
                   map: &UnitMap ) -> Vec<Coord> {
    let dim = map.dim();
    let mut ret = vec![];

    let mut offsets = [ Coord::from( (-1,0) ),
                    Coord::from( (1,0) ),
                    Coord::from( (0,-1) ),
                    Coord::from( (0,1) ) ];

    rand::thread_rng().shuffle( & mut offsets[..] );
    
    
    for i in offsets.iter() {
        let pos = (current + *i).modulo( &map.dim() );
        if !explored.contains(&pos) {
            match map.get(pos) {
                Unit::None => {
                    ret.push( pos );
                },
                _ => {},
            }
        }
    }
    
    ret
}
    
fn heuristic_cost( start: Coord, dest: Coord, boundary: Coord ) -> usize {
    ///returns L1 distance
    let ret = (start-dest).modulo(&boundary).abs();
    assert!( ret >= 0 );
    ret as usize
}

pub fn search_path( start: Coord, dest: Coord, map: &UnitMap ) -> Vec<Coord> {

    let dim = map.dim();
    let mut processed : HashSet<Coord> = HashSet::new();
    let mut costs : HashMap<Coord, usize> = HashMap::new();

    let mut h : BinaryHeap<CoordCost> = BinaryHeap::new();
    
    h.push( CoordCost( (start,0) ) );
    costs.insert( start, 0 );

    let mut path : HashMap<Coord,Coord> = HashMap::new();

    let mut count_steps = 0;
    
    while !h.is_empty() {

        let frontier = h.pop().unwrap();

        let frontier_coord = (frontier.0).0;

        if frontier_coord == dest {
            break;
        }
        
        let choices = get_unexplored( frontier_coord, & processed, map );

        let cost_frontier = (frontier.0).1;
        
        for i in choices.iter() {
            
            let cost_to_i = cost_frontier + 1;
            let cost_to_dest = heuristic_cost( *i, dest, dim );
            let cost_estimate = cost_to_i + cost_to_dest;

            costs.insert( *i, cost_to_i );
            
            processed.insert( *i );
            
            h.push( CoordCost( ( *i, cost_estimate ) ) );
            path.insert( *i, frontier_coord );
        }
        count_steps += 1;
        if count_steps > ((dim.y() * dim.x()) as f32 * 0.5) as usize {
            break;
        }
    }

    let mut ret = vec![];
    
    if let Some(x) = path.get( &dest ) { //if a path to dest exists return it
        ret.push(dest);
        ret.push(*x);
        let mut cur = x;
        loop {
            match path.get(cur) {
                Some(y) => {
                    ret.push(*y);
                    cur = y;
                    if *cur == start {
                        break;
                    }
                },
                _ => {
                    break;
                },
            }
            
        }
        ret.reverse();
    }
    
    ret
}
