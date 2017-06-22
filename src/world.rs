#![feature(associated_consts)]
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::cell::UnsafeCell;
use std::any::{Any, TypeId};
use std::default::Default;
use serde;

use ::{render,vec,matrix,camera2,mesh,resource,shader,material};
use transform::Transform;
use camera2::Camera;
use component::mesh_render::MeshRender;

#[derive(Serialize, Deserialize, Clone)]
pub struct Entity
{
    pub id : usize,
    pub name : String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct EntityRef
{
    id : usize
}

pub struct EntityWorld<'a>
{
    pub id : usize,
    world : &'a World
}

impl<'a> EntityWorld<'a>
{
    fn new(id : usize, world : &World) -> EntityWorld
    {
        EntityWorld {
            id : id,
            world : world
        }
    }

    fn from_ref(e : EntityRef, world : &World) -> EntityWorld
    {
        EntityWorld {
            id : e.id,
            world : world
        }
    }

    fn get_comp_mut_ptr<T:Component + Any>(&self, data : &mut Data) -> Option<*mut T>
    {
        if let Some(v) = self.world.entities_comps[self.id].get(T::ID) {
            data.get_mut_ptr::<T>(*v)
        }
        else {
            println!("entworld no such thing");
            None
        }
    }

    fn get_comp<'b, T:Component + Any>(&self, data : &'b Data) -> Option<&'b T>
    {
        if let Some(v) = self.world.entities_comps[self.id].get(T::ID) {
            data.get::<T>(*v)
        }
        else {
            None
        }
    }
}

pub struct EntityWorldMut<'a>
{
    pub id : usize,
    world : &'a mut World
}

impl<'a> EntityWorldMut<'a>
{
    fn new(id : usize, world : &mut World) -> EntityWorldMut
    {
        EntityWorldMut {
            id : id,
            world : world
        }
    }

    fn add<'b,T : Component + Any>(&mut self, data : &'b mut Data) -> Option<&'b mut T>
    {
        self.world.add_usize::<T>(self.id, data)
    }
}



impl EntityRef {
    fn new(id : usize) -> EntityRef {
        EntityRef {
            id : id
        }
    }
}

impl EntityMut {
    fn new(id : usize) -> EntityMut {
        EntityMut {
            id : id
        }
    }
}


pub struct EntityMut
{
    pub id : usize
}


impl Entity {
    
    fn new(id : usize, name : String) -> Entity
    {
        Entity {
            id : id,
            name : name,
        }
    }
}

#[derive(Serialize,Deserialize, Clone)]
pub struct Human {
    speed : f64
}

#[derive(Serialize,Deserialize, Clone)]
pub struct Zombie {
    speed : f64
}

#[derive(Serialize,Deserialize, Clone)]
pub struct Weapon;

trait WorldChange {

    fn change(&self, world : &mut World);
}

pub struct Nothing;

impl WorldChange for Nothing
{
    fn change(&self, world : &mut World)
    {
    }
}

pub trait Component {
    const ID : &'static str;

    fn update(&mut self, entity : &EntityMut, world : &World, data : &mut Data) -> Box<WorldChange>
    {
        Box::new(Nothing)
    }

    fn update_entity_world(&mut self, entity : &EntityWorld, world : &World, data : &mut Data) -> Box<WorldChange>
    {
        Box::new(Nothing)
    }


     fn as_any(&self) -> &Any;
     fn as_any_mut(&mut self) -> &mut Any;
}

impl Component for Human {
    const ID : &'static str = "human";

     fn as_any(&self) -> &Any {
        self
    }

     fn as_any_mut(&mut self) -> &mut Any {
        self
     }

    fn update(&mut self, entity : &EntityMut, world : &World, data : &mut Data) -> Box<WorldChange>
    {
        println!("updating human, {}", entity.id); 

        if let Some(t) = world.get_comp_mut_ptr::<Transform>(data, entity)
        {
            let t = unsafe {&mut*t};
            println!("  ---> human pos, {:?}", t); 
            
            if let Some(z) = find_nearest::<Weapon>(world, data, t.position.x)
            {
               println!("the nearest weapon is {:?}", z); 
            }
            else
            {
                println!("there is no weapon");
            }

            if let Some(z) = find_nearest::<Zombie>(world, data, t.position.x)
            {
               println!("the nearest zombie is {:?}", z);
            }
            else
            {
                println!("there is no zombie");
            }
        }
        else {
            println!("no transform!");
        }


        Box::new(Nothing)
    }

    fn update_entity_world(&mut self, entity : &EntityWorld, world : &World, data : &mut Data) -> Box<WorldChange>
    {
        if let Some(t) = entity.get_comp_mut_ptr::<Transform>(data)
        //if let Some(t) = entity.get_comp_mut::<Transform>(data)
        {
            let t = unsafe {&mut*t};
            //t.x = 5f64;
            println!("  ---> human pos, {:?}", t);

            if let Some(z) = find_nearest_comp::<Weapon>(world, data, t.position.x)
            {
            }

            if let Some(z) = find_nearest::<Weapon>(world, data, t.position.x)
            {
               println!("the nearest weapon is {:?}", z);
            }
            else
            {
                println!("there is no weapon");
            }

            if let Some(z) = find_nearest::<Zombie>(world, data, t.position.x)
            {
               println!("the nearest zombie is {:?}", z); 
            }
            else
            {
                println!("there is no zombie");
            }
        }
        else {
            println!("no transform!");
        }

        Box::new(Nothing)
    }
}

fn find_nearest<T:Any>(world : &World, data : &Data, pos : f64) -> Option<EntityRef>
{
    let en = world.get_entities_with::<T>();

    let mut nearest = None;
    for e in &en.v {
        let t = world.get_comp::<Transform>(data, e.clone()).unwrap();

        match nearest {
            None => {
                nearest = Some((t.position.x, e.clone()));
            },
            Some((n,_)) => if (t.position.x - pos).abs() < n {
                nearest = Some((t.position.x, e.clone()));
            }
        }
    }

    nearest.map(|x| x.1)
}

fn find_nearest_comp<'a,T:Component+Any>(world : &World, data : &'a Data, pos : f64) -> Option<&'a T>
{
    if let Some(n) = find_nearest::<T>(world,data,pos) {
        let e = EntityWorld::from_ref(n, world);
        e.get_comp(data)
    }
    else {
        None
    }

}


impl Default for Human {

    fn default() -> Self
    {
        Human { speed : 4f64 }
    }

}

impl Component for Zombie {
    const ID : &'static str = "zombie";

     fn as_any(&self) -> &Any {
        self
     }

     fn as_any_mut(&mut self) -> &mut Any {
        self
     }
}

impl Default for Zombie  {
    fn default() -> Self
    {
        Zombie { speed : 2f64 }
    }
}

trait AddDefault {
    fn add_default(&mut self) -> usize;
    fn add_default_return(&mut self) -> (usize, &mut Any);
}
impl<T:Default+Component> AddDefault for Vec<T> {
    fn add_default(&mut self) -> usize
    {
        let id = self.len();
        self.push(Default::default());
        id
    }

    fn add_default_return(&mut self) -> (usize, &mut Any)
    {
        let id = self.len();
        self.push(Default::default());
        (id, self[id].as_any_mut())
    }
}

impl Component for Weapon {
    const ID : &'static str = "weapon";

     fn as_any(&self) -> &Any {
        self
    }

     fn as_any_mut(&mut self) -> &mut Any {
        self
    }

}

impl Default for Weapon
{
    fn default() -> Self
    {
        Weapon
    }
}

impl Weapon
{

}

impl Component for Transform {
    const ID : &'static str = "transform";

     fn as_any(&self) -> &Any {
        self
    }

     fn as_any_mut(&mut self) -> &mut Any {
        self
    }

}

impl Component for Camera {
    const ID : &'static str = "camera";

     fn as_any(&self) -> &Any {
        self
    }

     fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}

impl Component for MeshRender {
    const ID : &'static str = "mesh_renderer";

     fn as_any(&self) -> &Any {
        self
    }

     fn as_any_mut(&mut self) -> &mut Any {
        self
    }

}



#[derive(Serialize,Deserialize, Clone)]
pub struct Data {
    human : Vec<Human>,
    zombie : Vec<Zombie>,
    weapon : Vec<Weapon>,
    transform : Vec<Transform>,
    camera : Vec<Camera>,
    mesh_render : Vec<MeshRender>
}

impl Data {
    fn new() -> Data {
        Data {
            human : Vec::new(),
            zombie : Vec::new(),
            weapon : Vec::new(),
            transform : Vec::new(),
            camera : Vec::new(),
            mesh_render : Vec::new(),
        }
    }

    fn get<T:Component + Any>(&self, index : usize) -> Option<&T>
    {
        /*
        match T::ID {
            "human" => self.human[index].as_any().downcast_ref::<T>(),
            "zombie" => self.zombie[index].as_any().downcast_ref::<T>(),
            _ => None
        }
        */

        let tt = TypeId::of::<T>();

        if tt == TypeId::of::<Transform>() {
            self.transform[index].as_any().downcast_ref::<T>()
        }
        else if tt == TypeId::of::<Human>() {
            self.human[index].as_any().downcast_ref::<T>()
        }
        else if tt == TypeId::of::<Zombie>() {
            self.zombie[index].as_any().downcast_ref::<T>()
        }
        else if tt == TypeId::of::<Weapon>() {
            self.weapon[index].as_any().downcast_ref::<T>()
        }
        else {
            None
        }
    }

    fn get_mut<T:Component + Any>(&mut self, index : usize) -> Option<&mut T>
    {
        let tt = TypeId::of::<T>();

        if tt == TypeId::of::<Transform>() {
            self.transform[index].as_any_mut().downcast_mut::<T>()
        }
        else if tt == TypeId::of::<Human>() {
            self.human[index].as_any_mut().downcast_mut::<T>()
        }
        else if tt == TypeId::of::<Zombie>() {
            self.zombie[index].as_any_mut().downcast_mut::<T>()
        }
        else if tt == TypeId::of::<Weapon>() {
            self.weapon[index].as_any_mut().downcast_mut::<T>()
        }
        else {
            None
        }
    }

    fn get_mut_ptr<T:Component + Any>(&mut self, index : usize) -> Option<*mut T>
    {
        let tt = TypeId::of::<T>();

        if tt == TypeId::of::<Human>() {
            //self.human[index].as_any_mut().downcast_mut::<T>()
            if let Some(t) = self.human[index].as_any_mut().downcast_mut::<T>(){
                Some(t)
            }
            else {
                None
            }
        }
        else if tt == TypeId::of::<Zombie>() {
            //self.zombie[index].as_any_mut().downcast_mut::<T>()
            if let Some(t) = self.zombie[index].as_any_mut().downcast_mut::<T>(){
                Some(t)
            }
            else {
                None
            }
        }
        else if tt == TypeId::of::<Transform>() {
            if let Some(t) = self.transform[index].as_any_mut().downcast_mut::<T>(){
                Some(t)
            }
            else {
                None
            }
        }
        else if tt == TypeId::of::<Weapon>() {
            if let Some(t) = self.weapon[index].as_any_mut().downcast_mut::<T>(){
                Some(t)
            }
            else {
                None
            }
        }
        else {
            None
        }
    }

    fn get_comp(&self, name : &str, index : usize) -> Option<&Component>
    {
        match name {
            "human" => Some(&self.human[index]),
            "zombie" => Some(&self.zombie[index]),
            "transform" => Some(&self.transform[index]),
            "weapon" => Some(&self.weapon[index]),
            _ => None
        }
    }

    fn get_comp_mut(&mut self, name : &str, index : usize) -> Option<&mut Component>
    {
        match name {
            "human" => Some(&mut self.human[index]),
            "zombie" => Some(&mut self.zombie[index]),
            "transform" => Some(&mut self.transform[index]),
            "weapon" => Some(&mut self.weapon[index]),
            //"human" => Some( unsafe {self.human.get_unchecked_mut(index) }),
            //"zombie" => Some( unsafe {self.zombie.get_unchecked_mut(index)}),
            _ => None
        }
    }

    fn get_comp_mut_ptr(&mut self, name : &str, e : &EntityMut, index : usize) -> Option<*mut Component>
    {
        println!("Data, get_comp_mut_ptr : {}", name);
        match name {
            "human" => Some(&mut self.human[index]),
            "zombie" => Some(&mut self.zombie[index]),
            "transform" => Some(&mut self.transform[index]),
            "weapon" => Some(&mut self.weapon[index]),
            _ => None
        }
    }


    fn add<T:Component + Any>(&mut self) -> Option<usize>
    {
        let tt = TypeId::of::<T>();

        let v : &mut AddDefault =
            if tt == TypeId::of::<Transform>() {
                &mut self.transform
            }
            else if tt == TypeId::of::<Human>() {
                &mut self.human
            }
            else if tt == TypeId::of::<Zombie>() {
                &mut self.zombie
            }
            else if tt == TypeId::of::<Weapon>() {
                &mut self.weapon
            }
            else {
                return None;
            };

        Some(v.add_default())
    }

    fn add_and_return<T:Component + Any>(&mut self) -> Option<(usize, &mut T)>
    {
        let tt = TypeId::of::<T>();

        let v : &mut AddDefault =
            if tt == TypeId::of::<Transform>() {
                &mut self.transform
            }
            else if tt == TypeId::of::<Human>() {
                &mut self.human
            }
            else if tt == TypeId::of::<Zombie>() {
                &mut self.zombie
            }
            else if tt == TypeId::of::<Weapon>() {
                &mut self.weapon
            }
            else {
                return None;
            };
        
        let (id, any) = v.add_default_return();
        return Some((id, any.downcast_mut::<T>().unwrap()));
    }

}

#[derive(Serialize,Deserialize, Clone)]
pub struct DataOwners {
    human : Vec<usize>,
    zombie : Vec<usize>,
    weapon : Vec<usize>,
    transform : Vec<usize>,
    camera : Vec<usize>,
    mesh_render : Vec<usize>,
}

impl DataOwners {
    fn new() -> DataOwners {
        DataOwners {
            human : Vec::new(),
            zombie : Vec::new(),
            weapon : Vec::new(),
            transform : Vec::new(),
            camera : Vec::new(),
            mesh_render : Vec::new(),
        }
    }

    fn set_owner<T:Component + Any>(&mut self, comp_id : usize, e : usize)
    {
        let tt = TypeId::of::<T>();

        let mut v = if tt == TypeId::of::<Transform>() {
            &mut self.transform
        }
        else if tt == TypeId::of::<Human>() {
            &mut self.human
        }
        else if tt == TypeId::of::<Zombie>() {
            &mut self.zombie
        }
        else if tt == TypeId::of::<Weapon>() {
            &mut self.weapon
        }
        else if tt == TypeId::of::<camera2::Camera>() {
            &mut self.weapon
        }
        else {
            panic!("no such component : {}", T::ID);
        };

        if v.is_empty() || comp_id > v.len() -1 {
            v.push(e);
        }
        else {
            v[comp_id] = e;
        }
    }

}

#[derive(Serialize,Deserialize, Clone)]
pub struct World {
    pub name : String,
    pub id : usize,
    entities : Vec<Entity>,
    //data : Box<Data>,
    pub entities_comps : Vec<HashMap<String, usize>>,
    //maybe it is better to do this? :
    //pub entities_comps : Vec<Option<usize>>, or Vec<Vec<usize>> if multiples components are possible
    // or it also possible to do :
    // struct comp {
    //  transform : usize,
    //  player : Option<usize>,
    //  enemy : Option<usize>,
    //  ...
    //  all components... : Option<usize>
    // }
    owners : DataOwners
}

impl World
{
    pub fn new(name : String, id : usize) -> World {
        World {
            entities : Vec::new(),
            entities_comps : Vec::new(),
            owners : DataOwners::new(),
            id : id,
            name : name,
            //data : box Data::new()
        }
    }

    fn update(&self, data : &mut Data)
    {
        //let events = Vec::new();
        
        /*
        for e in &self.entities {
            e.update(self);
        }
        */

        /*
        for (id, entity_comps) in self.entities_comps.iter().enumerate() {
            let e = EntityMut::new(id);
            for (s, c_id) in entity_comps {
                //if let Some(c) = data.get_mut(s, *c_id) {
                 //   c.update(self, data);
                //}
                
                if let Some(c) = data.get_comp_mut_ptr(s, &e, *c_id) {
                    unsafe { (*c).update(&e, self, data); }
                }
            }
        }
        */

        for (id, entity_comps) in self.entities_comps.iter().enumerate() {
            let e = EntityMut::new(id);
            let ew = EntityWorld::new(id,self);
            for (s, c_id) in entity_comps {
                if let Some(c) = data.get_comp_mut_ptr(s, &e, *c_id) {
                    unsafe { (*c).update_entity_world(&ew, self, data); }
                }
            }
        }
    }

    pub fn add_entity(&mut self, name : String) -> Entity {
        let id = self.entities.len();
        self.entities_comps.push(HashMap::new());
        let e = Entity::new(id, name);
        self.entities.push(e.clone());
        e
    }

    fn add_entity_world(&mut self, name : String) -> EntityWorldMut {
        let id = self.entities.len();
        self.entities_comps.push(HashMap::new());
        let e = Entity::new(id, name);
        self.entities.push(e.clone());
        EntityWorldMut::new(id, self)
    }


    fn add_comp<T : Component + Any>(&mut self, e : &Entity, data : &mut Data)
    {
        if let Some(c) = data.add::<T>() {
            self.entities_comps[e.id].insert(T::ID.to_owned(), c);
            self.owners.set_owner::<T>(c, e.id);
        }
        else
        {
            println!("cannot add {}", T::ID);
        }
    }

    fn add<'a,T : Component + Any>(&mut self, e : &Entity, data : &'a mut Data) -> Option<&'a mut T>
    {
        if let Some((id,c)) = data.add_and_return::<T>() {
            self.entities_comps[e.id].insert(T::ID.to_owned(), id);
            self.owners.set_owner::<T>(id, e.id);
            Some(c)
        }
        else
        {
            println!("cannot add {}", T::ID);
            None
        }
    }

    fn add_usize<'a,T : Component + Any>(&mut self, e : usize, data : &'a mut Data) -> Option<&'a mut T>
    {
        if let Some((id,c)) = data.add_and_return::<T>() {
            self.entities_comps[e].insert(T::ID.to_owned(), id);
            self.owners.set_owner::<T>(id, e);
            Some(c)
        }
        else
        {
            println!("cannot add {}", T::ID);
            None
        }
    }



    fn get_comp<'a, T:Component + Any>(&self, data : &'a Data, e : EntityRef) -> Option<&'a T>
    {
        if let Some(v) = self.entities_comps[e.id].get(T::ID) {
            data.get::<T>(*v)
        }
        else {
            None
        }
    }

    pub fn get_comp_mut<'a, T:Component + Any>(&self, data : &'a mut Data, e : &EntityMut) -> Option<&'a mut T>
    {
        if let Some(v) = self.entities_comps[e.id].get(T::ID) {
            data.get_mut::<T>(*v)
        }
        else {
            None
        }
    }

    fn get_comp_mut_ptr<T:Component + Any>(&self, data : &mut Data, e : &EntityMut) -> Option<*mut T>
    {
        println!("World, get_comp_mut_ptr,  {}", T::ID);
        if let Some(v) = self.entities_comps[e.id].get(T::ID) {
            println!("   -> no problem  {}", T::ID);
            data.get_mut_ptr::<T>(*v)
        }
        else {
            println!("no such thing");
            None
        }
    }

    fn get_entities_with<T:Any>(&self) -> Entities
    {
        let tt = TypeId::of::<T>();

        let v = if tt == TypeId::of::<Human>() {
            self.owners.human.clone()
        }
        else if tt == TypeId::of::<Zombie>() {
            self.owners.zombie.clone()
        }
        else if tt == TypeId::of::<Weapon>() {
            self.owners.weapon.clone()
        }
        else {
            Vec::new()
        };

        //Entities::new(EntityRef::new(v))
        Entities::new(v.iter().map(|x| EntityRef::new(*x)).collect())
    }

}

#[derive(Clone)]
struct Entities {
    v : Vec<EntityRef>
}

impl Entities {

    fn new(v : Vec<EntityRef>) -> Entities
    {
        Entities {
            v : v
        }
    }

    fn filter<T:Component+Any>(&self, world : &World) -> Entities
    {
        let entities = world.get_entities_with::<T>();
        self.and(&entities)
    }

    fn and(&self, e : &Entities) -> Entities
    {
        let mut v = self.v.clone();
        v.retain(|x| e.v.contains(x));

        e.clone()
    }

    fn or(&self, e : &Entities) -> Entities
    {
        let mut v = self.v.clone();
        v.extend_from_slice(&e.v);
        v.sort();
        v.dedup();
        Entities::new(v)
    }
}

pub trait WorldTrait : Default {
    fn add_empty(&mut self)
    {

    }
}

pub struct WorldRefDataMut<'a> {
    pub world : &'a World,
    pub data : &'a mut Data
}

pub trait Graph<E> {
    fn get_parent(&self, e : &E) -> Option<E>;
}

pub struct NoGraph;
impl<E> Graph<E> for NoGraph
{
    fn get_parent(&self, e : &E) -> Option<E>
    {
        None
    }
}

