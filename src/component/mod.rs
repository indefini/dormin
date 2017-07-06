
pub use self::manager::{
    Component,
    Manager,
    CompData,
    Components
};

pub mod player;
pub mod manager;
pub mod armature_animation;


//start trying to make a trait component
// Problem : will I be able to de/serialize it.

pub trait CompTrait {
    //const ID : &'static str;

    //fn new() -> Self;
    fn update(&mut self);//, world : &mut World);
}


