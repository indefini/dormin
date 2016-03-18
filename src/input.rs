use std::collections::HashSet;

pub struct Input
{
    keys : HashSet<u8>
}

impl Input
{
    pub fn new() -> Input
    {
        Input {
            keys : HashSet::new()
        }
    }

    pub fn is_key_down(&self, k : u8) -> bool
    {
        self.keys.contains(&k)
    }

    pub fn add_key(&mut self, k : u8)
    {
        println!("insert key : {}", k);
        self.keys.insert(k);
    }

    pub fn clear(&mut self)
    {
        self.keys.clear();
    }
}

