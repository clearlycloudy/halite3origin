use rand::Rng;
use rand::distributions::{Distribution,Uniform};
use rand::SeedableRng;
use std::collections::{HashMap,HashSet};
use std::borrow::BorrowMut;

use mapping::mapraw::{ResourceMap, RawMaps, UnitMap, DropoffMap, Unit};
use common::agent::{Agent,AgentStatus};
use common::coord::{Coord,Dir};
use common::player::{Player,PlayerStats};
use hlt;

fn approx_path_cost() -> i32 {
    unimplemented!();
}

fn plan_mine_locations( agents: &[Agent], maps: &RawMaps ) -> Vec<(u32, Coord)> {
    unimplemented!();
}

pub fn schedule_v2( my_id: &usize,
                    queued: Vec<(i32,Coord,Coord)>,
                    agents: &HashMap<Player, HashMap<i32,Agent> >,
                    map: & mut RawMaps,
                    is_end_game: &bool )
                    -> Vec<(i32,Dir)> {

    let map_dim = map.map_r.dim();
    
    let mut ret : Vec<(i32,Dir)> = vec![];

    let mut q_move_to_mine = vec![];
    let mut q_move_to_dropoff = vec![];
    let mut q_move_others = vec![];
    
    let my_agents = agents.get( &Player( *my_id ) ).expect("player not found");
    
    for (id,from,to) in queued.iter() {
        let a = my_agents.get( id ).expect("playe agent not found");
        assert_eq!( a.pos, *from );
        match a.status {
            AgentStatus::MoveToMine => { q_move_to_mine.push( (id,from,to) ); },
            AgentStatus::MoveToDropoff | AgentStatus::EndGame => { q_move_to_dropoff.push( (id,from,to) ); },
            _ => { q_move_others.push( (id,from,to) ); },
        }
    }

    q_move_to_mine.sort_unstable_by( |a,b| (*a.1-*a.2).abs().cmp( &(*b.1-*b.2).abs() ) );

    q_move_to_dropoff.sort_unstable_by( |a,b| (*a.1-*a.2).abs().cmp( &(*b.1-*b.2).abs() ) );

    q_move_others.sort_unstable_by( |a,b| (*a.1-*a.2).abs().cmp( &(*b.1-*b.2).abs() ) );

    let mut q_reordered = vec![];
    q_reordered.extend_from_slice( q_move_to_mine.as_slice() );
    q_reordered.extend_from_slice( q_move_to_dropoff.as_slice() );
    q_reordered.extend_from_slice( q_move_others.as_slice() );

    //clear this since it will be rewritten
    map.map_u.clear_player_agents( *my_id );

    //alternatives: do a path search or do greedy neighbouring selection

    // let mut processed = HashMap::new(); //map id -> (from, next_pos)
    // let mut unprocessed = HashMap::new(); //map id -> (from,to)
    // let mut own_ship_locations = HashMap::new(); //map coord -> id
        
    // for i in q_reordered.iter() {
    //     unprocessed.insert( *i.0, (*i.1,*i.2) );
    //     own_ship_locations.insert( *i.1, *i.0 );
    // }
        
    // for i in q_reordered.iter() {
    //     if let None = processed.get( &i.0 ) {
    //         let dir_priority = i.1.get_prioritized_dir( i.2, &map );
            
    //         for d in dir_priority {
    //             let coord_new = (*i.1 + d).mod_bound( &map.map_r.dim );
    //             match map.map_u.get( &coord_new ){
    //                 Unit::Ship{..} => {},
    //                 _ => {
    //                     // own_ship_locations.get( coord_new );
                            
    //                     // processed.insert( i.0, ( (i.1).0, coord_new ) );
    //                     break;
    //                 },
    //             }
    //         }
    //     }
    // }
    
    ret
}

pub fn schedule( my_id: &usize,
                 queued: Vec<(i32,Coord,Coord)>,
                 agents: &HashMap<Player, HashMap<i32,Agent> >,
                 map: & mut RawMaps,
                 is_end_game: &bool )
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
    if max_turns - turn_num <= (rawmaps.map_r.dim().y() * 6 / 10) as usize {
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

pub fn plan_strategy_around_dropoff( log: & mut hlt::log::Log, myid: &usize, player_agents: & mut HashMap<usize,Agent>, map_r: & mut ResourceMap, map_d: & mut DropoffMap, map_u: & mut UnitMap ) {
    
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
                            if resource_count < 30 && assign_new_mine {
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

    let mut agent_idx = 0;

    let dropoffs = map_d.get_player_dropoffs(*myid);
    let num_drop_offs = dropoffs.len();

    let num_agents_per_dropoff = agent_action_change.len() / num_drop_offs;
    let mut count_dropoff = 0;
    
    //assign each associated agent with a new location to mine
    'loop_dropoff: for (dropoff_id,dropoff_pos) in dropoffs.iter() {
        
        let dropoff_coord = *dropoff_pos;
        
        let mut p = *dropoff_pos;
        let mut step_stop = 2;
        let mut p_n = p + Coord( (step_stop/2, step_stop/2 ) );
        log.log(&format!("p_n init: {:?}", p_n));
        let mut step_count = 0;
        #[derive(Clone,Copy)]
        enum TraceDir {
            L,R,U,D
        }
        let mut d = TraceDir::U;

        let mut cell_processed = 0;
        let mut cell_total = 0;
        
        while cell_total < map_r.dim().y() * map_r.dim().x() && agent_idx < agent_action_change.len() {

            let agent_dropoff_partition = agent_idx / num_agents_per_dropoff;
            if agent_dropoff_partition > count_dropoff {
                count_dropoff += 1;
                continue 'loop_dropoff;
            }
            
            let a_id = &agent_action_change[agent_idx];

            let mut a = player_agents.get_mut(a_id).expect("agent id not found");
            
            if step_count >= step_stop {
                let new_d = { match d {
                    TraceDir::L => { TraceDir::D },
                    TraceDir::D => { TraceDir::R },
                    TraceDir::R => {
                        step_stop += 1;
                        p_n = p_n + Coord( ( (step_stop)/2, (step_stop)/2 ) );
                        TraceDir::U
                    },
                    TraceDir::U => { TraceDir::L },
                } };
                d = new_d;
                step_count = 0;
            }
            match d {
                TraceDir::L => { p_n = p_n + Coord( (0, -1) ); },
                TraceDir::D => { p_n = p_n + Coord( (1, 0 ) ); },
                TraceDir::R => { p_n = p_n + Coord( (0, 1) ); },
                TraceDir::U => { p_n = p_n + Coord( (-1, 0) ); },
            };
            
            step_count += 1;
            
            let halite_in_cell = map_r.get( Coord::from( p_n ) );

            let mut rng = rand::thread_rng();
            let num_gen_2: f32 = rng.gen();
            
            if (halite_in_cell >= 750 && num_gen_2 < 0.9 ) ||
                (halite_in_cell >= 500 && halite_in_cell < 750 && num_gen_2 < 0.5) ||
                ( halite_in_cell >= 250 && halite_in_cell < 500 && num_gen_2 < 0.2) ||
                ( halite_in_cell >= 100 && halite_in_cell < 200 && num_gen_2 < 0.01) ||
                ( halite_in_cell < 30 && num_gen_2 < 0.00001 ) {
                match map_u.get( p_n ) {
                    Unit::None => {
                        let Coord( (y,x) ) = p_n.modulo( &map_r.dim() );
                        
                        let mut processed = false;
                        if let AgentStatus::Idle = a.status {
                            a.status = AgentStatus::MoveToMine;
                            a.reset_cooldown_movetomine();
                        }

                        a.assigned_mine = Some( Coord( (y,x) ) );

                        //assign a drop off point for the agent
                        let mut best_dropoff = None;
                        let mut min_dropoff_norm = std::i32::MAX;
                                           
                        for (dropoff_id,dropoff_pos) in map_d.get_player_dropoffs( *myid ).iter() {

                            let coord_diff = *dropoff_pos - Coord( (y,x) );

                            let diff_norm = (coord_diff.0).0.abs() + (coord_diff.0).1.abs();
                            if diff_norm < min_dropoff_norm {
                                min_dropoff_norm = diff_norm;
                                best_dropoff = Some( *dropoff_pos );
                            }
                        }
                        
                        a.assigned_dropoff = Some( best_dropoff.unwrap() );
                        processed = true;
                        agent_idx += 1;
                        
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
        count_dropoff += 1;
    }
}

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
            // if stats.score > 1000 
            //     && pos_empty
            //     && !*is_end_game
            //     && *turn_num <= (*max_turn * 65 ) / 100 {
            //     true
            // } else {
            //     false
            // }
            
            if stats.score > 1000 &&
                pos_empty &&
                !*is_end_game &&
                (stats.ships == 0 || (rawmaps.map_r.total_remain() as f32 / stats.ships as f32 > 6000. && ( *turn_num <= (*max_turn * 7 ) / 10 ) ) ) {
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


