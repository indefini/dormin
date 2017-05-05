use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::{Read,Write};
use uuid::Uuid;
use std::path::Path;
use std::fmt;
use serde;
use serde_json;
use toml;
use armature;
use input;

use object;
use camera;
use component;
use resource;

use transform;

#[derive(Serialize, Deserialize)]
pub struct Scene
{
    pub name : String,
    pub id : Uuid,
    #[serde(serialize_with="serialize_refcell", deserialize_with="deserialize_option_refcell")]
    pub camera : Option<Rc<RefCell<camera::Camera>>>,
    #[serde(skip_serializing, skip_deserializing)]
    pub cameras : Vec<Rc<RefCell<camera::Camera>>>,

    #[serde(serialize_with="serialize_vec_arc", deserialize_with="deserialize_vec_arc")]
    pub objects : Vec<Arc<RwLock<object::Object>>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub transforms : Vec<transform::Transform>,
}

//TODO remove all serde manual implementations.
fn deserialize_option_refcell<D, T>(d : D) ->
    Result<Option<Rc<RefCell<T>>>, D::Error> where D: serde::Deserializer, T : serde::Deserialize
{
    if let Ok(r) = deserialize_refcell(d) {
        Ok(Some(Rc::new(r)))
    }
    else {
        Ok(None)
    }
}

fn deserialize_refcell<D, T>(d : D) -> Result<RefCell<T>, D::Error> where D: serde::Deserializer, T : serde::Deserialize
{
    let value = try!(T::deserialize(d));
    Ok(RefCell::new(value))
}


fn serialize_refcell<S,T>(t: &Option<Rc<RefCell<T>>>, s : S) -> Result<S::Ok, S::Error> where S: serde::Serializer, T : serde::Serialize
{
    if let Some(ref t) = *t {
        t.borrow().serialize(s)
    }
    else {
        s.serialize_none()
    }
}

fn deserialize_vec_arc<D, T>(d : D) -> Result<Vec<Arc<RwLock<T>>>, D::Error> where D: serde::Deserializer, T : serde::Deserialize
{
    use std::marker::PhantomData;

    pub struct VisitorVec<T> {
        marker: PhantomData<T>,
    }

    impl<T> VisitorVec<T>
            where T : serde::Deserialize
        {
            pub fn new() -> Self {
                VisitorVec {
                    marker: PhantomData,
                }
            }
}

    impl<T> serde::de::Visitor for VisitorVec<T> where T: serde::Deserialize {
        type Value = Vec<Arc<RwLock<T>>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<Vec<Arc<RwLock<T>>>, E> where E: serde::de::Error,
        {
            Ok(Vec::new())
        }

        #[inline]
        fn visit_seq<V>(self, mut v: V) -> Result<Vec<Arc<RwLock<T>>>, V::Error>
            where V: serde::de::SeqVisitor,
        {
            let mut values = Vec::new();

            while let Some(value) = try!(v.visit()) {
                Vec::push(&mut values, Arc::new(RwLock::new(value)));
            }

            Ok(values)
        }
    }

    d.deserialize_seq(VisitorVec::new())
}

use serde::ser::SerializeSeq;
fn serialize_vec_arc<S,T>(t: &Vec<Arc<RwLock<T>>>, s : S) -> Result<S::Ok, S::Error> where S: serde::Serializer, T : serde::Serialize
{
    let mut seq = s.serialize_seq(Some(t.len()))?;
    for e in t {
        seq.serialize_element(&*e.read().unwrap())?;
    }
    seq.end()
}


pub enum CompIdent {
    String(String),
    Enemy,
}

struct CompRef
{
    kind : String,
    index : usize,
    used_count : usize
}

pub struct Object2
{
    pub name : String,
    id : Uuid,
    index : usize,
    alive : bool,
    used_count : usize,

    components : Vec<CompRef>
}

impl Object2 {

    fn as_ref(&self) -> ObRef {
        ObRef {
            index : self.index,
            used_count : self.used_count
        }
    }
}

pub struct Scene2
{
    pub name : String,
    pub id : Uuid,
    pub camera : Option<Rc<RefCell<camera::Camera>>>,
    pub cameras : Vec<Rc<RefCell<camera::Camera>>>,

    //object identification
    pub objects : Vec<Object2>,
    //local transform
    transforms : Vec<transform::Transform>,
    //graph
    children : Vec<Vec<usize>>,
    parents : Vec<Option<usize>>,

    //objects that are referenced by others,
    //if removed/changed notice them... really notice them?
    //is_ref_by : Vec<Vec<ObRef>>,
    //has_ref_to : Vec<Vec<ObRef>>
}

struct ObRef
{
    index : usize,
    used_count : usize
}

impl Scene2
{
    fn get_mut(&mut self, oref : &ObRef) -> Option<&mut Object2>
    {
        let o = &mut self.objects[oref.index];
        if o.used_count == oref.used_count {
            Some(o) 
        }
        else {
            None
        }
    }

    fn get(&mut self, oref : &ObRef) -> Option<&Object2>
    {
        let o = &self.objects[oref.index];
        if o.used_count == oref.used_count {
            Some(o) 
        }
        else {
            None
        }
    }

    fn set_parent(&mut self, o : &Object2, new_parent : Option<&Object2>)
    {
        if let Some(p) = new_parent {
            self.parents[o.index] = Some(p.index);
        }
        else {
            self.parents[o.index] = None;
        }
    }

    fn add_child(&mut self, o : &Object2, child : &Object2)
    {
        if self.has_child(o, child) {
            println!("already have this child");
            return;
        }
        
        self.children[o.index].push(child.index);
    }

    fn has_child(&self, o : &Object2, child : &Object2) -> bool
    {
        for c in &self.children[o.index]
        {
            if *c == child.index {
                return true;
            }
        }

        false
    }

    fn remove_child(&mut self, o : &Object2, child : &Object2)
    {
        println!("TODO : don't test all! just remove and return");
        self.children[o.index].retain(|&i| i != child.index);
    }

    fn remove_all_children(&mut self, o : &Object2)
    {
        self.children[o.index].clear();
    }
}

struct Enemy {
    target : Option<ObRef>,
    speed : f64,
}

struct CompAll {
    transform : Pool<transform::Transform>,
    enemies : Pool<Enemy>
}

trait Comp2 {
    fn new() -> Self;
    fn reset(&mut self, scene : &mut Scene) {}
    fn update(&mut self, ob : ObRef, scene : &mut Scene, comp_all : &CompAll) {}
}

pub struct Pool<T>
{
    data : Vec<T>,
    //the number of unused cell starting from the end of the vec
    unused : usize
}

impl<T> Pool<T>
{
    //fn get

}

impl Comp2 for Enemy {
    fn new() -> Enemy {
        Enemy {
            target : None,
            speed : 0f64
        }
    }

    fn update(
        &mut self,
        ob : ObRef,
        scene : &mut Scene,
        comp_all : &CompAll) 
    {
        //let t = comp_all.transform.get(
        //get my pos
    }
}

impl Scene
{
    pub fn new(name : &str, id : Uuid, cam : camera::Camera) -> Scene
    {
        let cam = Rc::new(RefCell::new(cam));
        let cameras = vec![cam.clone()];

        Scene {
            name : String::from(name),
            id : id,
            objects : Vec::new(),
            camera : Some(cam),
            cameras : cameras,
            transforms : Vec::new()
        }
    }

    pub fn new_from_file(file_path : &str) -> Scene
    {
        let mut file = String::new();
        File::open(&Path::new(file_path)).ok().unwrap().read_to_string(&mut file);
        let mut scene : Scene = serde_json::from_str(&file).unwrap();

        scene.post_read();

        scene
    }

    fn post_read(&mut self)
    {
        for o in self.objects.iter()
        {
            post_read_parent_set(o.clone());

            if let Some(ref c) = self.camera {
                self.cameras.push(c.clone());
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
                    Some(component::mesh_render::MeshRenderer::with_names_only(&mr.mesh,&mr.material));
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

        let js = serde_json::to_string_pretty(self);
        let result = file.write(js.unwrap().as_bytes());
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

    pub fn update(
        &mut self,
        dt : f64,
        input : &input::Input,
        resource : &resource::ResourceGroup
        )
    {
        for o in self.objects.iter() {
            o.write().unwrap().update(dt, input, resource);
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
            cameras : self.cameras.clone(),
            objects : objects,
            transforms : self.transforms.clone()
        }
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

