use std::cmp::Ordering;

use rand::Rng;
use rand::distributions::{Distribution,Uniform};
use rand::SeedableRng;
use std::collections::{HashMap,HashSet,VecDeque,BinaryHeap};
use std::borrow::BorrowMut;

use mapping::mapraw::{ResourceMap, RawMaps, UnitMap, DropoffMap, Unit};
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
                    Coord::from( (0,1) ) ];

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
    if num_gen < 0.2 && !is_end_game {
        rand::thread_rng().shuffle( & mut ret[..] );
    }
    
    ret
}

struct NeighCost((i32,usize));

impl Ord for NeighCost {
    fn cmp( & self, other: &NeighCost ) -> Ordering {
        (self.0).1.cmp( &(other.0).1 )
    }
}

impl PartialOrd for NeighCost {
    fn partial_cmp( &self, other: &NeighCost ) -> Option<Ordering> {
        Some( self.cmp(other) )
    }
}

impl PartialEq for NeighCost {
    fn eq( & self, other: &NeighCost ) -> bool {
        (other.0).1.cmp( &(self.0).1 ) == Ordering::Equal
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

        if let Some(spawned_id) = new_agent_id {
            if spawned_id == *id {
                continue;
            }
        }
        
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

                // map.map_u.remove( from );
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
                            Unit::Ship{..} => {},
                            _ => {panic!("unexpected item in map"); },
                        }

                        ret.push( (id, Dir::from(diff) ) );
                    }//  else {
                    //     ret.push( (id, Dir( (0,0) ) ) );
                    // }
            }
        }
    }
    
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

pub fn plan_strategy_new( log: & mut hlt::log::Log,
                          turn_num: usize,
                          max_turns: usize,
                          my_id: &usize,
                          agents: & mut HashMap<Player, HashMap<i32,Agent> >,
                          rawmaps: & mut RawMaps ) -> bool {

    let mut my_agents = agents.get_mut( &Player(*my_id) ).expect("agents not found for input player id");
    
    let mut is_end_game = false;
    if max_turns - turn_num <= (rawmaps.map_r.dim().y() * 8 / 10) as usize {
        is_end_game = true;
        for a in my_agents.iter_mut() {
            let mut shortest_dist = 99999999;
            let mut dest = None;

            for (dropoff_id,dropoff_pos) in rawmaps.map_d.get_player_dropoffs(*my_id).iter() {
                
                let dist = (a.1.pos - *dropoff_pos).abs();

                if shortest_dist > dist {
                    shortest_dist = dist;
                    dest = Some(*dropoff_pos);
                }
            }
            a.1.assigned_dropoff = Some(dest.unwrap());
            a.1.status = AgentStatus::EndGame;
        }
    } else {
        plan_strategy( & mut log.borrow_mut(), my_id, my_agents, & mut rawmaps.map_r, & mut rawmaps.map_d, & mut rawmaps.map_u );
        
        determine_create_dropoff( & mut log.borrow_mut(), &my_id, my_agents, &rawmaps.map_r, &rawmaps.map_d, &rawmaps.map_u );
    }
    is_end_game
}

///old stuff

pub fn plan_strategy( log: & mut hlt::log::Log, myid: &usize, player_agents: & mut HashMap<i32,Agent>, map_r: & mut ResourceMap, map_d: & mut DropoffMap, map_u: & mut UnitMap ) {
    
    let mut agent_action_change = vec![];

    //find agents with mine resource amount below a threshold
    for (id,a) in player_agents.iter() {
        match a.status {
            AgentStatus::Idle => {
                agent_action_change.push(*id);
            },
            AgentStatus::MoveToDropoff =>{
                match a.assigned_mine {
                    None => { agent_action_change.push(*id); },
                    Some(x) => {
                        // let (y,x) = (a.pos).0;
                        if let Some(Coord((y,x))) = a.assigned_mine {
                            let resource_count = map_r.get( Coord((y, x)) );
                            let mut assign_new_mine = false;
                            if a.cooldown_movetomine() <= 0 && a.cooldown_mine() <= 0 {
                                assign_new_mine = true;
                            }
                            if resource_count < 40 && assign_new_mine {
                                let mut rng = rand::thread_rng();                            
                                let num_gen: f32 = rng.gen();
                                if num_gen < 0.9 {
                                    agent_action_change.push(*id);
                                }   
                            }   
                        }
                    },
                }
            },
            _ => {},
        }
    }

    //log.log(&format!("agent_action_change: {:?}", agent_action_change));

    //assign each associated agent with a new location to mine
    for a_id in agent_action_change.iter() {
        let mut a = player_agents.get_mut(a_id).expect("agent id not found");
        //trace out a square path and find cells that have halite amount above a threshold
        
        let mut p = a.pos.0;
        let mut step_stop = 2;
        let mut p_n = (p.0+step_stop/2, p.1+step_stop/2);
        log.log(&format!("p_n init: {:?}", p_n));
        let mut step_count = 0;
        #[derive(Clone,Copy)]
        enum TraceDir {
            L,R,U,D
        }
        let mut d = TraceDir::U;

        let mut cell_processed = 0;
        let mut cell_total = 0;
        while cell_processed < agent_action_change.len() && cell_total < map_r.dim().y() * map_r.dim().x() {
            
            if step_count >= step_stop {
                let new_d = { match d {
                    TraceDir::L => { TraceDir::D },
                    TraceDir::D => { TraceDir::R },
                    TraceDir::R => {
                        step_stop += 1;
                        p_n.0 = p.0 + (step_stop)/2;
                        p_n.1 = p.1 + (step_stop)/2;
                        TraceDir::U
                    },
                    TraceDir::U => { TraceDir::L },
                } };
                d = new_d;
                step_count = 0;
            }
            match d {
                TraceDir::L => { p_n.1 -= 1; },
                TraceDir::D => { p_n.0 += 1; },
                TraceDir::R => { p_n.1 += 1; },
                TraceDir::U => { p_n.0 -= 1; },
            };

            // log.log(&format!("p_n: {:?}, step count: {}", p_n, step_count));
            
            step_count += 1;
            let halite_in_cell = map_r.get( Coord( (p_n.0, p_n.1) ) );

            let mut rng = rand::thread_rng();
            let num_gen_2: f32 = rng.gen();
            
            if (halite_in_cell >= 750 && num_gen_2 < 0.85 ) ||
                (halite_in_cell >= 500 && halite_in_cell < 750 && num_gen_2 < 0.45) ||
                ( halite_in_cell >= 250 && halite_in_cell < 500 && num_gen_2 < 0.25) ||
                ( halite_in_cell >= 100 && halite_in_cell < 200 && num_gen_2 < 0.02) ||
                ( halite_in_cell >= 30 && halite_in_cell < 100 && num_gen_2 < 0.001) ||
                ( halite_in_cell < 30 && num_gen_2 < 0.00005 ) {
                match map_u.get( Coord( (p_n.0, p_n.1) ) ) {
                    Unit::None => {
                        let Coord( (y,x) ) = Coord::from( p_n ).modulo( &map_r.dim() );

                        let mut processed = false;
                        if let AgentStatus::Idle = a.status {
                            a.status = AgentStatus::MoveToMine;
                            a.reset_cooldown_movetomine();
                        }

                        a.assigned_mine = Some( Coord( (y,x) ) );

                        //assign a drop off point for the agent
                        let mut best_dropoff = None;
                        let mut min_dropoff_norm = std::i32::MAX;

                        for (dropoff_id,dropoff_pos) in map_d.get_player_dropoffs(*myid).iter() {

                            let coord_diff = *dropoff_pos - Coord( (y,x) );

                            let diff_norm = (coord_diff.0).0.abs() + (coord_diff.0).1.abs();
                            if diff_norm < min_dropoff_norm {
                                min_dropoff_norm = diff_norm;
                                best_dropoff = Some( *dropoff_pos );
                            }
                        }
                        
                        a.assigned_dropoff = Some( best_dropoff.unwrap() );
                        processed = true;
                        
                        log.log(&format!("agent after action change: {:?}", a));
                        if processed {
                            cell_processed += 1;
                        }
                    },
                    _ => {},
                }
            }
            cell_total += 1;
        }
    }
}

// pub fn plan_strategy_around_dropoff( log: & mut hlt::log::Log, myid: &usize, player_agents: & mut HashMap<usize,Agent>, map_r: & mut ResourceMap, map_d: & mut DropoffMap, map_u: & mut UnitMap ) {
    
//     let mut agent_action_change = vec![];

//     //find agents with mine resource amount below a threshold
//     for (id,a) in player_agents.iter() {
//         match a.status {
//             AgentStatus::Idle => {
//                 agent_action_change.push(*id);
//             },
//             AgentStatus::MoveToDropoff =>{
//                 match a.assigned_mine {
//                     None => { agent_action_change.push(*id); },
//                     Some(x) => {
//                         // let (y,x) = (a.pos).0;
//                         if let Some(Coord((y,x))) = a.assigned_mine {
//                             let resource_count = map_r.get( Coord((y, x)) );
//                             let mut assign_new_mine = false;
//                             if a.cooldown_movetomine() <= 0 && a.cooldown_mine() <= 0 {
//                                 assign_new_mine = true;
//                             }
//                             if resource_count < 30 && assign_new_mine {
//                                 let mut rng = rand::thread_rng();                            
//                                 let num_gen: f32 = rng.gen();
//                                 if num_gen < 0.9 {
//                                     agent_action_change.push(*id);
//                                 }   
//                             }   
//                         }
//                     },
//                 }
//             },
//             _ => {},
//         }
//     }

//     let mut agent_idx = 0;

//     let dropoffs = map_d.get_player_dropoffs(*myid);
//     let num_drop_offs = dropoffs.len();

//     let num_agents_per_dropoff = agent_action_change.len() / num_drop_offs;
//     let mut count_dropoff = 0;
    
//     //assign each associated agent with a new location to mine
//     'loop_dropoff: for (dropoff_id,dropoff_pos) in dropoffs.iter() {
        
//         let dropoff_coord = *dropoff_pos;
        
//         let mut p = *dropoff_pos;
//         let mut step_stop = 2;
//         let mut p_n = p + Coord( (step_stop/2, step_stop/2 ) );
//         log.log(&format!("p_n init: {:?}", p_n));
//         let mut step_count = 0;
//         #[derive(Clone,Copy)]
//         enum TraceDir {
//             L,R,U,D
//         }
//         let mut d = TraceDir::U;

//         let mut cell_processed = 0;
//         let mut cell_total = 0;
        
//         while cell_total < map_r.dim().y() * map_r.dim().x() && agent_idx < agent_action_change.len() {

//             let agent_dropoff_partition = agent_idx / num_agents_per_dropoff;
//             if agent_dropoff_partition > count_dropoff {
//                 count_dropoff += 1;
//                 continue 'loop_dropoff;
//             }
            
//             let a_id = &agent_action_change[agent_idx];

//             let mut a = player_agents.get_mut(a_id).expect("agent id not found");
            
//             if step_count >= step_stop {
//                 let new_d = { match d {
//                     TraceDir::L => { TraceDir::D },
//                     TraceDir::D => { TraceDir::R },
//                     TraceDir::R => {
//                         step_stop += 1;
//                         p_n = p_n + Coord( ( (step_stop)/2, (step_stop)/2 ) );
//                         TraceDir::U
//                     },
//                     TraceDir::U => { TraceDir::L },
//                 } };
//                 d = new_d;
//                 step_count = 0;
//             }
//             match d {
//                 TraceDir::L => { p_n = p_n + Coord( (0, -1) ); },
//                 TraceDir::D => { p_n = p_n + Coord( (1, 0 ) ); },
//                 TraceDir::R => { p_n = p_n + Coord( (0, 1) ); },
//                 TraceDir::U => { p_n = p_n + Coord( (-1, 0) ); },
//             };
            
//             step_count += 1;
            
//             let halite_in_cell = map_r.get( Coord::from( p_n ) );

//             let mut rng = rand::thread_rng();
//             let num_gen_2: f32 = rng.gen();
            
//             if (halite_in_cell >= 750 && num_gen_2 < 0.9 ) ||
//                 (halite_in_cell >= 500 && halite_in_cell < 750 && num_gen_2 < 0.5) ||
//                 ( halite_in_cell >= 250 && halite_in_cell < 500 && num_gen_2 < 0.2) ||
//                 ( halite_in_cell >= 100 && halite_in_cell < 200 && num_gen_2 < 0.01) ||
//                 ( halite_in_cell < 30 && num_gen_2 < 0.00001 ) {
//                 match map_u.get( p_n ) {
//                     Unit::None => {
//                         let Coord( (y,x) ) = p_n.modulo( &map_r.dim() );
                        
//                         let mut processed = false;
//                         if let AgentStatus::Idle = a.status {
//                             a.status = AgentStatus::MoveToMine;
//                             a.reset_cooldown_movetomine();
//                         }

//                         a.assigned_mine = Some( Coord( (y,x) ) );

//                         //assign a drop off point for the agent
//                         let mut best_dropoff = None;
//                         let mut min_dropoff_norm = std::i32::MAX;
                                           
//                         for (dropoff_id,dropoff_pos) in map_d.get_player_dropoffs( *myid ).iter() {

//                             let coord_diff = *dropoff_pos - Coord( (y,x) );

//                             let diff_norm = (coord_diff.0).0.abs() + (coord_diff.0).1.abs();
//                             if diff_norm < min_dropoff_norm {
//                                 min_dropoff_norm = diff_norm;
//                                 best_dropoff = Some( *dropoff_pos );
//                             }
//                         }
                        
//                         a.assigned_dropoff = Some( best_dropoff.unwrap() );
//                         processed = true;
//                         agent_idx += 1;
                        
//                         log.log(&format!("agent after action change: {:?}", a));
//                         if processed {
//                             cell_processed += 1;
//                         }
//                     },
//                     _ => {},
//                 }
//             }
//             cell_total += 1;
//         }
//         count_dropoff += 1;
//     }
// }

pub fn determine_create_new_agent( player_stats: &HashMap< Player, PlayerStats >,
                                   my_id: &usize,
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
        
    let create = match player_stats.get( &Player(*my_id) ) {
        Some(stats) => {

            if stats.score > 1000 &&
                pos_empty &&
                !*is_end_game &&
                ( stats.ships == 0 || 
                   ( rawmaps.map_r.total_remain() as f32 / stats.ships as f32 > 6000.
                     && ( *turn_num <= (*max_turn * 7 ) / 10 )
                   )
                ) {
                    true
                } else {
                    false
                }
        },
        _ => { false },
    };

    create
}

pub fn determine_create_dropoff( log: & mut hlt::log::Log, myid: &usize, player_agents: & mut HashMap<i32,Agent>, map_r: &ResourceMap, map_d: &DropoffMap, map_u: &UnitMap ) {
    
}


