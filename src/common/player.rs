#[derive(Hash,Eq,PartialEq,Clone,Copy)]
pub struct Player(pub usize);

#[derive(Debug)]
pub struct PlayerStats {
    pub score: usize,
    pub ships: usize,
    pub dropoffs: usize,
}
