use rand::Rng;
use rand::distributions::{Distribution,Uniform};
use rand::SeedableRng;
use std::collections::{HashMap,HashSet};

use mapping::mapraw::{ResourceMap, RawMaps, UnitMap, DropoffMap, Unit};
use common::agent::{Agent,AgentStatus};
use common::coord::{Coord,Dir};

pub fn synchronize_player_agents( player_agents: & HashMap<usize,Agent>, update: Vec<(usize,(i32,i32),usize)> ) -> ( HashMap<usize,Agent>, Vec<Agent> ) {
    let mut processed_ids = HashSet::new();
    let mut ret = HashMap::new();

    let update_ids = update.iter().map(|x| x.0).collect::<HashSet<_>>();
    
    for (id,(y,x),halite) in update {
        processed_ids.insert(id);
        match player_agents.get(&id) {
            Some(a) => {
                assert_eq!(id, a.id);
                let mut agent_updated = a.clone();
                agent_updated.halite = halite;
                agent_updated.pos = Coord( (y,x) );
                ret.insert(id, agent_updated);
                // assert_eq!( agent_updated.expected_next_pos, Coord((y,x)) );
            },
            _ => {
                //new agent found
                let a = Agent {
                    assigned_mine: None,
                    assigned_dropoff: None,
                    status: AgentStatus::Idle,
                    halite: halite,
                    pos: Coord( (y,x) ),
                    id: id,
                    cooldown_mine: 0i32,
                    cooldown_movetomine: 0i32,
                };
                ret.insert(id, a);
            },
        }
    }

    let mut removed_agents = vec![];
    
    for i in player_agents.keys().cloned().collect::<HashSet<_>>().difference( &update_ids ) {
        let a = player_agents.get(&i).expect("agent not found");
        removed_agents.push( a.clone() );
    }
        
    ( ret, removed_agents )
}
