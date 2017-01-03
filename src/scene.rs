use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::{Read,Write};
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use uuid::Uuid;
use std::path::Path;
use toml;
use armature;
use input;

use object;
use camera;
use component;
use resource;

pub struct Scene
{
    pub name : String,
    pub id : Uuid,
    pub camera : Option<Rc<RefCell<camera::Camera>>>,

    pub objects : Vec<Arc<RwLock<object::Object>>>,
}

impl Scene
{
    pub fn new(name : &str, id : Uuid, cam : camera::Camera) -> Scene
    {
        Scene {
            name : String::from(name),
            id : id,
            objects : Vec::new(),
            camera : Some(Rc::new(RefCell::new(cam)))
        }
    }

    pub fn new_from_file(file_path : &str, resource :&resource::ResourceGroup) -> Scene
    {
        let mut file = String::new();
        File::open(&Path::new(file_path)).ok().unwrap().read_to_string(&mut file);
        let scene : Scene = json::decode(file.as_ref()).unwrap();

        scene.post_read(resource);

        scene
    }

    fn post_read(&self, resource : &resource::ResourceGroup)
    {
        for o in self.objects.iter()
        {
            post_read_parent_set(o.clone());

            if let Some(ref c) = self.camera {
                let mut cam = c.borrow_mut();
                let id = match cam.object_id {
                    Some(ref id) => id.clone(),
                    None => {
                        println!("camera has no id");
                        continue;
                    }
                };

                let (ob_id, name) = {
                    let ob = o.read().unwrap();
                    (ob.id.clone(), ob.name.clone())
                };

                if ob_id == id {
                    println!("fiiiiiiiiiiiiiiiiiiiiind");
                    cam.object = o.clone();
                }
                else if name == "robot"{
                /*
                    println!("it is not {}", o.read().unwrap().name);
                    let comp_mgr = component::manager::COMP_MGR.lock().unwrap();
                    let pc = comp_mgr.create_component("player_behavior").unwrap();
                    o.write().unwrap().add_component(
                        //Rc::new(RefCell::new(Box::new(component::player::Player::new()))));
                        Rc::new(RefCell::new(pc)));
                */
                    /*
                    let mut p = component::player::Player::new();
                p.speed = 5.0f;
                    o.write().unwrap().add_comp_data(box component::CompData::Player(p));
                    */
                    /*
                    let mut a = armature::Armature::new("armature/robot_armature.arm");
                    println!("++++++++++ arm name : {}", a.name);
                    a.file_read();
                    println!("++++++++++ arm name _after_read_ :  {}", a.name);
                    o.write().unwrap().add_comp_data(box component::CompData::Armature(a));
                    */

                    //let arm_path = String::from_str("armature/robot_armature.arm");
                    //o.write().unwrap().add_comp_data(box component::CompData::Armature(arm_path));

                    //o.write().unwrap().add_comp_string("player_behavior");
                    //o.write().unwrap().add_comp_string("armature_animation");
                }
            }
            else {
                println!("nooooooooo camera");
            }

            /*
            let mut ob =  o.write().unwrap();
            let mr = match ob.mesh_render {
                Some(ref mr) => {
                    Some((mr.mesh.name.clone(), mr.material.name.clone()))
                },
                None => None
            };

            if let Some((mesh, mat)) = mr {
                let mere = component::mesh_render::MeshRender::new(mesh.as_ref(), mat.as_ref());
                ob.add_comp_data(box component::CompData::MeshRender(mere));
            }
            */

            let mut ob =  o.write().unwrap();
            let b =false;
            if let Some(mrd) = ob.get_comp_data::<armature::ArmaturePath>(){
                println!("there is armature path");
            }

            let omr = ob.get_comp_data_value::<component::mesh_render::MeshRender>();
            if let Some(ref mr) = omr {
                ob.mesh_render = 
                    Some(component::mesh_render::MeshRenderer::with_mesh_render(mr,resource));
            }
        }
    }

    pub fn init_components(&self, resource : &resource::ResourceGroup)
    {
        let comp_mgr = component::manager::COMP_MGR.lock().unwrap();

        for o in self.objects.iter()
        {
            o.write().unwrap().init_components(&comp_mgr, resource);
        }
    }

    pub fn save(&self)
    {
        println!("save scene todo serialize");
        let path : &Path = self.name.as_ref();
        let mut file = File::create(path).ok().unwrap();
        let mut s = String::new();
        {
            let mut encoder = json::Encoder::new_pretty(&mut s);
            let _ = self.encode(&mut encoder);
        }

        //let result = file.write(s.as_ref().as_bytes());
        let result = file.write(s.as_bytes());
    }

    pub fn object_find(&self, name : &str) -> Option<Arc<RwLock<object::Object>>>
    {
        for o in self.objects.iter()
        {
            if o.read().unwrap().name == name {
                return Some(o.clone());
            }
        }

        None
    }

    pub fn find_object_by_id(&self, id : &Uuid) -> Option<Arc<RwLock<object::Object>>>
    {
        fn find(list : &[Arc<RwLock<object::Object>>], id : &Uuid) ->
            Option<Arc<RwLock<object::Object>>>
            {
                for o in list.iter()
                {
                    if o.read().unwrap().id == *id {
                        return Some(o.clone());
                    }
                    else {
                        if let Some(aro) = find(&o.read().unwrap().children, id) {
                            return Some(aro);
                        }
                    }
                }
                None
            }

        if id.is_nil() {
            None
        }
        else {
            find(&self.objects, id)
        }
    }

    pub fn find_objects_by_id(&self, ids : &mut Vec<Uuid>) -> Vec<Arc<RwLock<object::Object>>>
    {
        let mut return_list = Vec::new();
        fn find(
            list : &[Arc<RwLock<object::Object>>],
            ids : &mut Vec<Uuid>,
            return_list : &mut Vec<Arc<RwLock<object::Object>>>
            )
            {
                for o in list.iter()
                {
                    let mut found = false;
                    for i in 0..ids.len() {
                        if o.read().unwrap().id == ids[i] {
                            ids.remove(i);
                            return_list.push(o.clone());
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        find(&o.read().unwrap().children, ids, return_list);
                    }
                }
            }

        find(&self.objects, ids, &mut return_list);
        return_list
    }

    pub fn find_objects_by_id_or_none(&self, ids : &[Uuid]) -> 
        Vec<Option<Arc<RwLock<object::Object>>>>
    {
        let mut return_list = Vec::new();
        for i in ids {
            if i.is_nil() {
                return_list.push(None);
            }
            else {
                return_list.push(self.find_object_by_id(i));
            }
        }

        return_list
    }


    pub fn add_objects(&mut self, parents : &[Uuid], obs : &[Arc<RwLock<object::Object>>])
    {
        let pvec = self.find_objects_by_id_or_none(parents);

        for (i,p) in pvec.iter().enumerate() {
            if let Some(ref par) = *p {
                par.write().unwrap().children.push(obs[i].clone());
            }
            else {
                self.objects.push(obs[i].clone());
            }
        }
    }

    pub fn add_objects_by_vec(&mut self, obs : &mut Vec<Arc<RwLock<object::Object>>>)
    {
        self.objects.append(obs);
    }

    pub fn remove_objects(&mut self, parents : &[Uuid], obs : &[Arc<RwLock<object::Object>>])
    {
        let pvec = self.find_objects_by_id_or_none(parents);

        fn remove(
            list : &mut Vec<Arc<RwLock<object::Object>>>,
            id : Uuid
            )
            {
                let mut index = None;
                for (j,o) in list.iter().enumerate() {
                    if o.read().unwrap().id == id {
                        index = Some(j);
                        break;
                    }
                }

                if let Some(idx) = index {
                    list.swap_remove(idx);
                }
            }

        for (i,p) in pvec.iter().enumerate() {
            let rem_id = obs[i].read().unwrap().id;

            if let Some(ref par) = *p {
                remove(&mut par.write().unwrap().children, rem_id);
            }
            else {
                remove(&mut self.objects, rem_id);
            };
        }

        /*
        let mut to_remove = Vec::new();

        let mut obs = obs.to_vec();

        for (i,o) in self.objects.iter().enumerate() {

            let mut j = 0;
            for r in obs.iter() {
                if o.read().unwrap().id == r.read().unwrap().id {
                    println!("found the id, break {}", o.read().unwrap().name);
                    to_remove.push(i);
                    break;
                }
                j = j + 1;
            }
            if j < obs.len() {
                obs.swap_remove(j);
            }
        }

        for r in &to_remove {
            //TODO change parent and child
            self.objects.swap_remove(*r);
        }
        */
    }

    pub fn savetoml(&self)
    {
        let s = toml::encode_str(self);
        println!("encoder toml : {} ", s );
    }

    /*
    pub fn new_toml(s : &str) -> Material
    {
        let mat : Material = toml::decode_str(s).unwrap();
        mat
    }
    */

    pub fn update(&mut self, dt : f64, input : &input::Input)
    {
        for o in self.objects.iter() {
            o.write().unwrap().update(dt, input);
        }
    }
}

impl Clone for Scene {

    //TODO
    fn clone(&self) -> Scene {
        println!("TODO ERR scene cloning is not done yet, camera too");
        let mut objects = Vec::new();
        for o in self.objects.iter() {
            let oc = o.read().unwrap().clone();
            objects.push(Arc::new(RwLock::new(oc)));
        }

        let cam = if let Some(ref cc) = self.camera {
            let camc = cc.borrow().clone();
            Some(Rc::new(RefCell::new(camc)))
        }
        else {
            None
        };

        Scene {
            name : self.name.clone(),
            id : self.id.clone(),//?
            camera : cam,
            objects : objects
        }
    }
}


impl Encodable for Scene {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      encoder.emit_struct("Scene", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.name.encode(encoder)));
          try!(encoder.emit_struct_field( "id", 1usize, |encoder| self.id.encode(encoder)));
          try!(encoder.emit_struct_field( "objects", 2usize, |encoder| self.objects.encode(encoder)));
          try!(encoder.emit_struct_field( "camera", 3usize, |encoder| self.camera.encode(encoder)));
          Ok(())
      })
  }
}

impl Decodable for Scene {
  fn decode<D : Decoder>(decoder: &mut D) -> Result<Scene, D::Error> {
      decoder.read_struct("root", 0, |decoder| {
         Ok(Scene{
          name: try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
          id: try!(decoder.read_struct_field("id", 0, |decoder| Decodable::decode(decoder))),
         //id : Uuid::new_v4(),
          objects: try!(decoder.read_struct_field("objects", 0, |decoder| Decodable::decode(decoder))),
          //tests: try!(decoder.read_struct_field("objects", 0, |decoder| Decodable::decode(decoder))),
          //camera : None //try!(decoder.read_struct_field("camera", 0, |decoder| Decodable::decode(decoder)))
          camera : try!(decoder.read_struct_field("camera", 0, |decoder| Decodable::decode(decoder)))
          //camera : None
        })
    })
  }
}

pub fn post_read_parent_set(o : Arc<RwLock<object::Object>>)
{
    for c in o.read().unwrap().children.iter()
    {
        c.write().unwrap().parent = Some(o.clone());
        post_read_parent_set(c.clone());
    }
}

