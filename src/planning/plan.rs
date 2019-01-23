use std::cmp::Ordering;

use rand::Rng;
use rand::distributions::{Distribution,Uniform};
use rand::SeedableRng;
use std::collections::{HashMap,HashSet,VecDeque,BinaryHeap};
use std::borrow::BorrowMut;

use mapping::mapraw::{ResourceMap, RawMaps, UnitMap, DropoffMap, Unit, *};
use common::agent::{Agent,AgentStatus};
use common::coord::{Coord,Dir};
use common::player::{Player,PlayerStats};
use hlt;

use planning::search::{search_path};

fn approx_path_cost() -> i32 {
    unimplemented!();
}

fn plan_mine_locations( agents: &[Agent], maps: &RawMaps ) -> Vec<(u32, Coord)> {
    unimplemented!();
}

fn get_num_free_neighours( c: Coord, map: & RawMaps ) -> usize {
    
    let mut offsets = [ Coord::from( (-1,0) ),
                    Coord::from( (1,0) ),
                    Coord::from( (0,-1) ),
                    Coord::from( (0,1) ) ];

    let mut count = 0;
    for i in offsets.iter() {
        let pos = (c + *i).modulo( &map.map_u.dim() );
        match map.map_u.get(pos) {
            Unit::None => {
                count += 1;
            },
            _ => {},
        }
    }
    count
}

fn get_free_neighours( c: Coord, d: Coord, map: & RawMaps, is_end_game: bool ) -> Vec<(Coord,usize)> {
    
    let mut offsets = [ Coord::from( (-1,0) ),
                        Coord::from( (1,0) ),
                        Coord::from( (0,-1) ),
                        Coord::from( (0,1) )];

    rand::thread_rng().shuffle( & mut offsets[..] );

    let mut ret = vec![];
    
    for i in offsets.iter() {
        let pos = (c + *i).modulo( &map.map_u.dim() );
        match map.map_u.get(pos) {
            Unit::None => {
                
                let delta = d.diff_wrap_around( &pos, &map.map_u.dim() );
                
                ret.push( ( pos, delta.abs() as usize ) );
            },
            _ => {},
        }
    }

    ret.sort_unstable_by(|a,b| a.1.cmp(&b.1) );
    
    let mut rng = rand::thread_rng();    
    let num_gen: f32 = rng.gen();
    if num_gen < 0.3 && !is_end_game {
        rand::thread_rng().shuffle( & mut ret[..] );
    }
    
    ret
}

//returns vector of neightbour position sorted in descending order of most free neighour spaces
fn get_neighours_most_free( c: Coord, map: & RawMaps ) -> Vec<(Coord,usize)> {
    
    let mut offsets = [ Coord::from( (-1,0) ),
                        Coord::from( (1,0) ),
                        Coord::from( (0,-1) ),
                        Coord::from( (0,1) )];

    rand::thread_rng().shuffle( & mut offsets[..] );

    let mut ret = vec![];
    
    for i in offsets.iter() {
        let pos = (c + *i).modulo( &map.map_u.dim() );
        let free = get_num_free_neighours( pos, map );
        ret.push( (pos,free) );
    }

    rand::thread_rng().shuffle( & mut ret[..] );    
    ret.sort_unstable_by(|a,b| b.1.cmp(&a.1) );
    
    ret
}

struct NeighCost((i32,usize));

impl Ord for NeighCost {
    fn cmp( & self, other: &NeighCost ) -> Ordering {
        (other.0).1.cmp( &(self.0).1 )
    }
}

impl PartialOrd for NeighCost {
    fn partial_cmp( &self, other: &NeighCost ) -> Option<Ordering> {
        Some( self.cmp(other) )
    }
}

impl PartialEq for NeighCost {
    fn eq( & self, other: &NeighCost ) -> bool {
        (self.0).1.cmp( &(other.0).1 ) == Ordering::Equal
    }
}

impl Eq for NeighCost {}

    
pub fn schedule_v2( log: & mut hlt::log::Log,
                    my_id: &usize,
                    queued: Vec<(i32,Coord,Coord)>,
                    agents: & mut HashMap<Player, HashMap<i32,Agent> >,
                    map: & mut RawMaps,
                    is_end_game: &bool,
                    new_agent_id: Option<i32> )
                    -> Vec<(i32,Dir)> {

    let mut count_neighbours = HashMap::new();

    let mut h = BinaryHeap::new();

    let mut agent_dest = HashMap::new();
    
    for (id,from,to) in queued.iter() {

        // if let Some(spawned_id) = new_agent_id {
        //     if spawned_id == *id {
        //         continue;
        //     }
        // }
        
        let num = get_num_free_neighours( *from, map );
        count_neighbours.insert( *id, num );
        h.push( NeighCost( (*id, num) ) );
        agent_dest.insert( *id, *to );
    }

    let mut processed = HashSet::new();

    let my_agents = agents.get_mut( &Player( *my_id ) ).expect("player not found");

    let mut ret = vec![];
    
    while !h.is_empty() {
        let node = h.pop().unwrap();
        let (id,neigh_val)= node.0;
        
        if !processed.contains( &id ) {
            processed.insert( id );

            let dest = *agent_dest.get( &id ).unwrap();

            let a = my_agents.get_mut( &id ).unwrap();

            let from = a.pos.modulo( &map.map_u.dim() );

            if dest == a.pos.modulo( &map.map_u.dim() ) {
                continue;
            }
            
            let path = search_path( from, dest, &map.map_u );
            let avai = if path.len() > 1 {
                if let Unit::None = map.map_u.get( path[1] ) {
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if path.len() > 1 && avai {

                assert!( path[0] == from );
                let coord_next = path[1].modulo( &map.map_u.dim() );
                let diff = coord_next.diff_wrap_around( &from, &map.map_u.dim() );
                log.borrow_mut().log(&format!("diff:{:?}", diff ) );
                log.borrow_mut().log(&format!("path:{:?}", path ) );
                assert!( diff.abs() == 1 );
                let unit = Unit::Ship { player: *my_id,
                                        id: a.id,
                                        halite: a.halite };
                
                a.pos = coord_next;

                //map.map_u.remove( from );
                // map.map_u.set( from, Unit::None );
                map.map_u.set( coord_next, unit );

                ret.push( (id, Dir::from(diff) ) );

            } else {
            // if true {
                let choices = get_free_neighours( from, dest, &map, *is_end_game );
                if ( dest.diff_wrap_around( &from, &map.map_u.dim() ).abs() == 1 )
                    && *is_end_game {
                        let unit = Unit::Ship { player: *my_id,
                                            id: a.id,
                                            halite: a.halite };

                        let coord_next = dest.modulo( &map.map_u.dim() );
                        a.pos = coord_next;

                        map.map_u.remove( from );
                        
                        map.map_u.set( coord_next, unit );
                        // map.map_u.set( coord_next, Unit::None );

                        match map.map_u.get( coord_next ) {
                            Unit::Ship{..} => {},
                            _ => {panic!("unexpected item in map"); },
                        }
                            
                        let diff = coord_next.diff_wrap_around( &from, &map.map_u.dim() );
                        
                        ret.push( (id, Dir::from(diff) ) );
                    } else if choices.len() > 0 {

                        // let mut rng = rand::thread_rng();
                        // let idx = rng.gen_range(0, choices.len());

                        let idx = 0;
                        let coord_next = choices[idx].0.modulo( &map.map_u.dim() );

                        match map.map_u.get( coord_next ) {
                            Unit::Ship{..} => { panic!("unexpected item in map"); },
                            _ => {},
                        }
                        
                        let diff = coord_next.diff_wrap_around( &from, &map.map_u.dim() );

                        log.borrow_mut().log(&format!("diff:{:?}", diff ) );
                        assert!( diff.abs() == 1 );
                        let unit = Unit::Ship { player: *my_id,
                                                id: a.id,
                                                halite: a.halite };

                        a.pos = coord_next;

                        // map.map_u.remove( from );
                        // map.map_u.set( from, Unit::None );
                        map.map_u.set( coord_next, unit );

                        match map.map_u.get( coord_next ) { 
                            Unit::Ship{id,..} => {
                            },
                            _ => {panic!("unexpected item in map"); },
                        }

                        // match map.map_u.get( from ) { 
                        //     Unit::Ship{..} => {
                        //         panic!("unexpected item in map");
                        //     },
                        //     _ => {},
                        // }

                        ret.push( (id, Dir::from(diff) ) );
                    }
                else {
                        // let neighbour_ranking = get_neighours_most_free( from, &map );
                        // assert!( neighbour_ranking.len() > 0 );

                        // for nei in neighbour_ranking.iter() {
                        //     match map.map_u.get( nei.0 ) {
                        //         Unit::Ship{id:nei_id,..} => {
                        //             if !processed.contains( &nei_id ) {

                        //                 let unit = Unit::Ship { player: *my_id,
                        //                                         id: a.id,
                        //                                         halite: a.halite };
                                        
                        //                 let coord_next = nei.0.modulo( &map.map_u.dim() );
                        //                 a.pos = coord_next;

                        //                 let u = map.map_u.get( nei.0 );
                                        
                        //                 // map.map_u.remove( from );
                        //                 map.map_u.set( from, u );
                        //                 map.map_u.set( coord_next, unit );

                        //                 let diff = coord_next.diff_wrap_around( &from, &map.map_u.dim() );
                        //                 let diff_nei = from.diff_wrap_around( &coord_next, &map.map_u.dim() );
                                        
                        //                 ret.push( (id, Dir::from(diff) ) );
                        //                 ret.push( (nei_id, Dir::from(diff_nei) ) );
                                        
                        //                 // h.push( NeighCost( (nei_id, 0) ) );
                        //                 processed.insert( nei_id );
                        //                 break;
                        //             }
                        //         },
                        //         _ => {},
                        //     }
                        // }
                }
            }
        }
    }
    ret.reverse();
    ret
}

pub fn schedule( log: & mut hlt::log::Log,
                 my_id: &usize,
                 queued: Vec<(i32,Coord,Coord)>,
                 agents: &HashMap<Player, HashMap<i32,Agent> >,
                 map: & mut RawMaps,
                 is_end_game: &bool,
                 new_agent_id: Option<i32> )
                 -> Vec<(i32,Dir)> {
                     
    let map_r = & map.map_r;
    let map_d = & map.map_d;
    let map_u = & mut map.map_u;

    let map_dim = map_r.dim();
    
    let mut ret : Vec<(i32,Dir)> = vec![];
    
    for (id,from,to) in queued {

        if from == to {
            continue;
        }

        if let Some(spawned_id) = new_agent_id {
            if spawned_id == id {
                continue;
            }
        }
        
        let mut dif = to-from;

        if (dif.0).0 > map_r.dim().y()/2 {
            (dif.0).0 -= map_r.dim().y();
        }
        if (dif.0).0 < -map_r.dim().y()/2 {
            (dif.0).0 += map_r.dim().y();
        }

        if (dif.0).1 > map_r.dim().x()/2 {
            (dif.0).1 -= map_r.dim().x();
        }
        if (dif.0).1 < -map_r.dim().x()/2 {
            (dif.0).1 += map_r.dim().x();
        }

        let mut choices_no = vec![];
        
        let mut choices = vec![];
        if (dif.0).0 >= 1 {
            choices.push( (from, Dir((1,0))) );
            choices_no.push( (from, Dir((-1,0))) );
            choices_no.push( (from, Dir((0,-1))) );
            choices_no.push( (from, Dir((0,1))) );
        } else if (dif.0).0 <= -1 {
            choices.push( (from, Dir((-1,0))) );
            choices_no.push( (from, Dir((1,0))) );
            choices_no.push( (from, Dir((0,-1))) );
            choices_no.push( (from, Dir((0,1))) );
        } else {
            choices_no.push( (from, Dir((0,0))) );
        }
        
        if (dif.0).1 >= 1 {
            choices.push( (from, Dir((0,1))) );
            choices_no.push( (from, Dir((0,-1))) );
            choices_no.push( (from, Dir((-1,0))) );
            choices_no.push( (from, Dir((1,0))) );
        } else if (dif.0).1 <= -1 {
            choices.push( (from, Dir((0,-1))) );
            choices_no.push( (from, Dir((0,1))) );
            choices_no.push( (from, Dir((-1,0))) );
            choices_no.push( (from, Dir((1,0))) );
        }else {
            choices_no.push( (from, Dir((0,0))) );
        }

        rand::thread_rng().shuffle( & mut choices[..] );

        rand::thread_rng().shuffle( & mut choices_no[..] );
        choices.extend_from_slice( &choices_no[..] );


        let mut found = false;
        let choices_filtered = choices
            .iter()
            .filter_map(|(fr,dir)| {
                let y = (fr.0).0 + (dir.0).0;
                let x = (fr.0).1 + (dir.0).1;
                let agent = map_u.get( Coord( ( (fr.0).0, (fr.0).1 ) ) );
                if let Unit::None = agent {
                    // panic!("ship not found at expected location: {:?}, map_u: {:?}", fr.0, map_u.invmap );
                    //already processed, skip it
                    None                    
                } else {
                    if let Unit::None = map_u.get( Coord( (y, x) ) ) {
                        if found {
                            None
                        } else {
                            // map_u.set( (fr.0).0, (fr.0).1, Unit::None );
                            map_u.set( Coord( (y, x) ), agent );
                            found = true;
                            Some(dir)
                        }
                    } else {
                        if *is_end_game {
                            if let Some(dropoff_player_id) = map_d.get( Coord( (y, x) ) ) {
                                if dropoff_player_id.0 == *my_id {
                                    map_u.set( Coord( (y, x) ), agent );
                                    found = true;
                                    Some(dir)
                                } else { None }
                            } else {
                                None
                            }
                        } else {
                            if let Unit::Ship{ player,.. } = map_u.get( Coord( (y, x) ) ) {
                                if player != *my_id {
                                    if let Some(dropoff_player_id) = map_d.get( Coord( (y, x) ) ) {
                                        if dropoff_player_id.0 == *my_id {
                                            map_u.set( Coord( (y, x) ), agent );
                                            found = true;
                                            Some(dir)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    }
                }
            })
            .cloned()
            .collect::<Vec<_>>();
            
        if choices_filtered.is_empty() {

        } else {   
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0, choices_filtered.len());
            ret.push( ( id as i32, choices_filtered[idx] ) );
        }
    }
    
    ret
}

pub fn synthesize_agent_actions( log: & mut hlt::log::Log,
                                 turn_num: usize,
                                 max_turns: usize,
                                 my_id: &usize,
                                 agents: & mut HashMap<Player, HashMap<i32,Agent> >,
                                 rawmaps: &RawMaps )
                                 -> ( Vec<(i32,Coord,Coord)>, Vec<i32> ) {
    //execute agent action
    let mut queued_movements = vec![];
    let mut queued_new_dropoffs = vec![];
    
    let mut my_agents = agents.get_mut( &Player(*my_id) );// .expect("agents not found for input player id");
    for a in my_agents.iter_mut() {
        a.iter_mut().for_each(|( agent_id, agent )| {

            match agent.status {
                AgentStatus::CreateDropoff => {
                    queued_new_dropoffs.push( agent.id );
                },
                _ => {
                    queued_movements.push( agent.execute( &rawmaps.map_r ) );
                },
            }
        });
    }
    ( queued_movements, queued_new_dropoffs )
}

//pick locations to mine
pub fn find_resource_to_mine( log: & mut hlt::log::Log, myid: &usize, player_agents: & mut HashMap<i32,Agent>, rawmaps: & mut RawMaps, kernel_dim: usize ) -> Vec<Coord> {
    
    //pick a suitable kernel size, do convolution on map
    //pick out best places from filtered resource map
    // let best_locs = get_best_locations( ConvolveKernelVal::Uniform, kernel_dim, &rawmaps.map_r.base.map, None );
    let best_locs = get_best_locations( ConvolveKernelVal::Gaussian, kernel_dim, &rawmaps.map_r.base.map, None );
    best_locs
}

//assign agents to selected resource locations
pub fn assign_agents_to_mine( log: & mut hlt::log::Log, myid: &usize, player_agents: & mut HashMap<i32,Agent>, rawmaps: & mut RawMaps, locations: &Vec<Coord> ) {

    //update agent status and assign resource if possible
    let mut agent_action_change = vec![];

    for (id,a) in player_agents.iter_mut() {
        match a.status {
            AgentStatus::Idle => {
                agent_action_change.push(*id);
                a.status = AgentStatus::MoveToMine;
            },
            AgentStatus::MoveToDropoff => {
                let mut best_dropoff = None;
                let mut min_dropoff_norm = std::i32::MAX;
                for (dropoff_id,dropoff_pos) in rawmaps.map_d.get_player_dropoffs(*myid).iter() {

                    let coord_diff = dropoff_pos.diff_wrap_around( &a.pos, &rawmaps.map_d.dim() ).abs();

                    if coord_diff < min_dropoff_norm {
                        min_dropoff_norm = coord_diff;
                        best_dropoff = Some( *dropoff_pos );
                    }
                }
                a.assigned_dropoff = Some( best_dropoff.unwrap() );
            }
            _ => {},
        }
    }
    
    let mut processed_cell = HashSet::new();

    let avg_map_resource = rawmaps.map_r.avg_remain();
    
    for i in agent_action_change.iter() {
        let mut a = player_agents.get_mut(i).expect("agent id not found");
        let pos_start = a.pos;
        let mut pos_best = None;
        let mut halite_best = std::f32::MIN;
        for j in locations.iter(){

            if processed_cell.contains( j ) {
                continue;
            }
            
            let pos_end = *j;
            let halite_agent = a.halite;

            let dropoff_point = {
                let mut dropoff = None;
                let mut min_dropoff_norm = std::i32::MAX;
                
                for (dropoff_id,dropoff_pos) in rawmaps.map_d.get_player_dropoffs(*myid).iter() {

                    let diff_norm = dropoff_pos.diff_wrap_around( &pos_end, &rawmaps.map_r.base.dim ).abs();
                    if diff_norm < min_dropoff_norm {
                        min_dropoff_norm = diff_norm;
                        dropoff = Some( *dropoff_pos );
                    }
                }
                dropoff.expect("dropoff invalid")
            };
            
            let halite_1 = get_approx_halite_in_path( halite_agent as f32,
                                                      pos_start,
                                                      pos_end,
                                                      &rawmaps,
                                                      PathAction::Subtract,
                                                      avg_map_resource );
            
            let halite_2 = get_approx_halite_in_path( halite_1,
                                                      pos_end,
                                                      dropoff_point,
                                                      &rawmaps,
                                                      PathAction::Add,
                                                      avg_map_resource );
            let halite_remain = halite_2;
            
            if halite_remain > halite_best {

                // let d0 = pos_end.diff_wrap_around( &pos_start, &rawmaps.map_r.base.dim );
                // let d1 = pos_best.unwrap().diff_wrap_around( &pos_start, &rawmaps.map_r.base.dim );
                // if d0.abs() < d1.abs() {
                //     halite_best = halite_remain;
                //     pos_best = Some(pos_end);
                //     processed_cell.insert( j );
                //     break;
                // }
                // } else {
                    halite_best = halite_remain;
                    pos_best = Some(pos_end);
                    processed_cell.insert( j );
                    break;
                // }
            }
        }
        if pos_best.is_some() {
            log.log(&format!("agent assigned resource location: {:?}", pos_best.unwrap()));            
            a.assigned_mine = pos_best;

            let mut best_dropoff = None;
            let mut min_dropoff_norm = std::i32::MAX;
            
            for (dropoff_id,dropoff_pos) in rawmaps.map_d.get_player_dropoffs(*myid).iter() {

                let coord_diff = *dropoff_pos - pos_best.unwrap();

                let diff_norm = (coord_diff.0).0.abs() + (coord_diff.0).1.abs();
                if diff_norm < min_dropoff_norm {
                    min_dropoff_norm = diff_norm;
                    best_dropoff = Some( *dropoff_pos );
                }
            }
            
            a.assigned_dropoff = Some( best_dropoff.unwrap() );
        }
    }
}

pub fn determine_create_new_agent( player_stats: &HashMap< Player, PlayerStats >,
                                   my_id: &usize,
                                   my_agents: & mut HashMap<i32,Agent>,
                                   rawmaps: &RawMaps,
                                   shipyard_pos: &HashMap<usize,Coord>,
                                   turn_num: &usize,
                                   max_turn: &usize,
                                   is_end_game: &bool ) -> bool {

    let my_shipyard_pos = shipyard_pos.get( my_id ).expect("shipyard position not found");

    let pos_empty = if let Unit::None = rawmaps.map_u.get( *my_shipyard_pos ) {
        true
    } else {
        false
    };

    let mydropoffs = rawmaps.map_d.get_player_dropoffs( *my_id );
    let agent_dropoff_density = my_agents.len() as f32 / mydropoffs.len() as f32;
    if agent_dropoff_density > 10. {
        return false
    }
    
    let create = match player_stats.get( &Player(*my_id) ) {
        Some(stats) => {

            if stats.score > 1000 &&
                pos_empty &&
                !*is_end_game &&
                  ( stats.ships == 0 || 
                   ( rawmaps.map_r.total_remain() as f32 / stats.ships as f32 > 6200.
                     && ( *turn_num <= (*max_turn * 62 ) / 100 )
                   )
            )
            {
                    true
                } else {
                    false
                }
        },
        _ => { false },
    };

    create
}

pub fn determine_create_dropoff( log: & mut hlt::log::Log,
                                 player_stats: &HashMap< Player, PlayerStats >,
                                 myid: &usize,
                                 my_agents: & mut HashMap<i32,Agent>,
                                 rawmaps: & mut RawMaps,
                                 turn_num: &usize,
                                 max_turn: &usize,
                                 is_end_game: &bool ) -> Option<i32> {

    let mut create = false;
    match player_stats.get( &Player(*myid) ) {
        Some(stats) => {

            if stats.score > 5000 &&
                !*is_end_game &&
                stats.ships > 0 {
                    create = true;
                }
        },
        _ => {},
    }

    if !create {
        return None
    }
    
    let mydropoffs = rawmaps.map_d.get_player_dropoffs( *myid );
    let agent_dropoff_density = my_agents.len() as f32 / mydropoffs.len() as f32;

    let mut in_process = false;
    for (id,agent) in my_agents.iter() {
        match agent.status {
            AgentStatus::CreateDropoff => {
                in_process = true;
                break;
            },
            _ => {},
        }
    }
    if in_process {
        return None
    }
    
    if agent_dropoff_density > 10. {
        
        let kernel_dim = 20;

        //find suitable resource locations
        let best_locs = find_resource_to_mine( & mut log.borrow_mut(),
                                                 myid, my_agents,
                                                 rawmaps, kernel_dim );

        let mut new_dropoff_pos = None;
        
        for candidate_pos in best_locs.iter() {
            let mut valid = true;
            for (id,dropoff_pos) in mydropoffs.iter() {
                let dist = candidate_pos.diff_wrap_around( dropoff_pos, &rawmaps.map_d.dim() ).abs();
                if dist < 25 {
                    valid = false;
                }
            }
            if valid {
                new_dropoff_pos = Some( *candidate_pos );
                break;
            }
        }
        
        let mut selected_id = None;
        
        match new_dropoff_pos {
            Some(pos) => {
                let mut min_dist = std::i32::MAX;
                for (id,agent) in my_agents.iter_mut() {
                    let dist = pos.diff_wrap_around( &agent.pos, &rawmaps.map_d.dim() ).abs();
                    if min_dist > dist {
                        min_dist = dist;
                        selected_id = Some(*id);
                    }
                }
            },
            _ => {},
        }

        if let Some(id) = selected_id {
            let a = my_agents.get_mut(&id).unwrap();
            a.create_dropoff = new_dropoff_pos;
            a.status = AgentStatus::CreateDropoff;
            return Some(id)
        }
    }

    None
}

