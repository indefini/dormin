//use component;
use std::rc::Rc;
use std::cell::RefCell;
use component::{Component,Components};
use component::manager::Encode;
//use object::ComponentFunc;
use object::Object;
use transform;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use resource;

use property::{PropertyRead, PropertyGet, PropertyWrite, WriteValue};
use std::any::Any;
use input;

#[derive(Clone, RustcEncodable, RustcDecodable, Serialize, Deserialize, Default)]
pub struct Player
{
    pub speed : f64
}

#[derive(Clone)]
pub struct PlayerBehavior;

impl Player
{
    pub fn new() -> Player
    {
        Player {
            speed : 3f64
        }
    }
}

pub fn player_new(ob : &Object, resource : &resource::ResourceGroup) -> Box<Components>
//pub fn player_new() -> Box<Component>
{
    box Components::PlayerBehavior(PlayerBehavior)
}

impl Component for PlayerBehavior
{
    /*
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(box PlayerBehavior))
    }
    */

    //fn update(&mut self, dt : f64) {}
    fn update(
        &mut self,
        ob : &mut Object,
        dt : f64,
        input : &input::Input,
        resource : &resource::ResourceGroup
        )
    {
        let speed = {
            match ob.get_mut_comp_data::<Player>(){
                Some(s) => s.speed,
                None => 0f64
            }
        };

        if input.is_key_down(26) {
            ob.position.z = ob.position.z + 5f64;
        }
        if input.is_key_down(39) {
            ob.position.x = ob.position.x - 5f64;
        }
        else if input.is_key_down(40) {
            ob.position.z = ob.position.z - 5f64;
        }
        else if input.is_key_down(41) {
            ob.position.x = ob.position.x + 5f64;
        }

        //let yep = ob.get_mut_comp_data::<Player>();

        let mut ori = ob.orientation.get_angle_xyz();
        ori.x += speed;
        //ob.orientation = transform::Orientation::new_with_angle_xyz(&ori);
    }

    fn get_name(&self) -> String {
        "player_behavior".to_owned()
    }

    /*
    fn new(ob : &Object) -> Box<Component>
    {
        box PlayerBehavior
    }
    */

    /*
    fn new(ob : &Object) -> Box<Component>
    {
        box PlayerBehavior
    }
    */
}

#[derive(Clone, RustcEncodable, RustcDecodable, Serialize, Deserialize, Default)]
pub struct Enemy {
    pub name : String
}

#[derive(Clone, RustcEncodable, RustcDecodable, Serialize, Deserialize, Default)]
pub struct Collider {
    pub name : String
}

impl Encode for Player
{
  fn encode_this<E : Encoder>(&self, encoder: &mut E)// -> Result<(), &str>
  {
      let _ = self.encode(encoder);

  }

}

property_set_impl!(Player,[speed]);
property_set_impl!(Enemy,[name]);
property_set_impl!(Collider,[name]);

property_get_impl!(Player,[speed]);
property_get_impl!(Enemy,[name]);
property_get_impl!(Collider,[name]);

