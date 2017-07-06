use component::{Component,Components};
use component::manager::Encode;
use object::Object;
use transform;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use resource;

use property::{PropertyRead, PropertyGet, PropertyWrite, WriteValue};
use std::any::Any;
use input;
use uuid;

#[derive(Clone, RustcEncodable, RustcDecodable,Default)]
pub struct CameraFollowData
{
    pub object : uuid::Uuid
}

#[derive(Clone)]
pub struct CameraFollowBehavior
{
    object : Rc<RefCell<Object>>
}

impl CameraFollow
{
    pub fn new() -> CameraFollowBehavior
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

impl Component for CameraFollowBehavior
{
    fn update(&mut self, ob : &mut Object, dt : f64, input : &input::Input)
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

property_set_impl!(Player,[speed]);
property_get_impl!(Player,[speed]);

