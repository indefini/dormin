use mesh;
use texture;
use shader;
use fbo;
use material;
use armature;
use vec;
use transform;

use std::collections::HashMap;
use std::collections::hash_map::Values;
use std::slice::{Iter,IterMut};
use std::collections::hash_map::Entry;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::sync::{RwLock, Arc, Mutex};
use std::sync::mpsc::channel;
//use std::time::Duration;
use std::thread;

use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::iter;
use std::fmt;

use uuid;


pub trait ResourceT  {
    fn init(&mut self);
}

fn return_none<T>() -> Option<T>
{
    None
}

#[derive(Serialize, Deserialize)]
pub struct ResTT<T>
{
    //TODO remove instance (only have instance_managed);
    // can we remove name?
    pub name : String,
    #[serde(skip_serializing, skip_deserializing)]
    resource : Cell<Option<usize>>,
    #[serde(skip_serializing, skip_deserializing, default="return_none")]
    pub instance : Option<T>,
    #[serde(skip_serializing, skip_deserializing)]
    pub instance_managed : Option<usize>,
}

impl<T> ResTT<T>
{
    pub fn new(name : &str) -> ResTT<T>
    {
        ResTT {
            name : String::from(name),
            resource : Cell::new(None),
            instance : None,
            instance_managed : None
        }
    }

    pub fn reset(&mut self) {
        self.resource.set(None);
    }

    pub fn new_with_instance(name : &str, r : T) -> ResTT<T>
    {
        ResTT {
            name : String::from(name),
            resource : Cell::new(None),
            instance : Some(r),
            instance_managed : None
        }
    }

    fn new_with_index(name : &str, res : usize) -> ResTT<T>
    {
        ResTT {
            name : String::from(name),
            resource : Cell::new(Some(res)),
            instance : None,
            instance_managed : None
        }
    }
}

impl<T:Create+Send+Sync+'static> ResTT<T>
{
    pub fn get_from_manager<'a>(&self, rm : &'a mut ResourceManager<T>) -> Option<&'a mut T>
    {
        if let Some(i) = self.resource.get() {
            rm.get_mut(i)
        }
        else {
            //TODO
            //let (i, r) = rm.request_use_new(self.name.as_ref(), load);
            //self.resource = Some(i);
            //r
            None
        }
    }

    pub fn get_from_manager_instant<'a> (&self, rm : &'a mut ResourceManager<T>) -> &'a mut T
    {
        let i = if let Some(i) = self.resource.get() {
            i
        }
        else {
            let i = rm.request_use_no_proc_new(self.name.as_ref());
            println!("warning !!!! this file gets requested everytime : {}", self.name);
            //self.resource = Some(i);
            i
        };

        rm.get_mut_instant(i)
    }

    pub fn get_mut<'a>(
        &'a mut self,
        rm : &'a mut ResourceManager<T>) -> Option<&'a mut T>
    {
        if self.instance.is_some() {
            self.instance.as_mut()
        }
        else if let Some(i) = self.resource.get() {
            rm.get_mut(i)
        }
        else {
            println!("warning !!!!, resource not loaded yet : {}", self.name);
            None
        }
    }

    pub fn get_resource<'a>(&'a mut self, manager : &'a mut ResourceManager<T>, load : Arc<Mutex<usize>> ) -> Option<&'a mut T>
    {
        resource_get(manager, self, load)
    }

    pub fn get_or_load_ref<'a>(&'a self, manager : &'a mut ResourceManager<T>) -> Option<&'a T>
    {
        resource_get_ref(manager, self)
    }

    pub fn get_ref<'a>(
        &'a self,
        manager : &'a ResourceManager<T>
        ) -> Option<&'a T>
    {
        if self.instance.is_some() {
            self.instance.as_ref()
        }
        else {
            self.get_ref_no_instance(manager)
        }
    }

    pub fn get_ref_no_instance<'a>(
        &self,
        manager : &'a ResourceManager<T>
        ) -> Option<&'a T>
    {
        if let Some(i) = self.resource.get() {
            manager.get_as_ref(i)
        }
        else {
            let o = manager.get_ref_from_name(self.name.as_ref());
            match o {
                Some((i, r)) => {
                    self.resource.set(Some(i));
                    Some(r)
                },
                None => None
            }
        }
    }

    pub fn as_ref_instant<'a>(
        &'a self,
        manager : &'a mut ResourceManager<T>
        ) -> Option<&'a T>
    {
        if self.instance.is_some() {
            self.instance.as_ref()
        }
        else if let Some(i) = self.resource.get() {
            Some(manager.get_ref_instant(i))
        }
        else {
            let i = manager.request_use_no_proc_new(&self.name);
            self.resource.set(Some(i));
            Some(manager.get_ref_instant(i))
        }
    }

    pub fn get_instance(&mut self) -> Option<&mut T>
    {
        self.instance.as_mut()
    }

    pub fn get_or_create_instance(&mut self) -> &mut T
    {
        if self.instance.is_none() {
            let mut mt : T = Create::create(self.name.as_ref());
            mt.inittt();
            self.instance = Some(mt);
        }

        self.instance.as_mut().unwrap()
    }

    pub fn is_instance(&self) -> bool
    {
        self.instance.is_some() || self.instance_managed.is_some()
    }

}

use core;

pub trait Clonable {
    fn make_clone(&self) -> Option<Self> where Self: core::marker::Sized
    {
        None
    }
}

impl<T:Clone> Clonable for T {
    fn make_clone(&self) -> Option<T>
    {
        Some(self.clone())
    }
}

impl Clonable for texture::Texture {}
impl Clonable for fbo::Fbo {}
impl Clonable for shader::Shader {}


//impl<T> Clone for ResTT<T>
impl<T:Clonable> Clone for ResTT<T>
{
    fn clone(&self) -> ResTT<T>
    {
        //println!("WARNING : clone for resource is TODO, because of instance");
        let instance = match self.instance {
            Some(ref i) => i.make_clone(),
            None => None
        };

        ResTT {
            name : self.name.clone(),
            resource : self.resource.clone(),
            //instance : None, //TODO
            instance : instance,
            instance_managed : None //TODO
        }
    }
}


impl <T:'static+Create+Send+Sync+Clone> ResTT<T>
{

    pub fn get_or_create_instance_no_load(
        &mut self,
        manager : &ResourceManager<T>) -> Option<&mut T>
    {
        if self.instance.is_some() {
            self.instance.as_mut()
        }
        else {
            if let Some(i) = self.resource.get() {
                if let Some(r) = manager.get_as_ref(i) {
                    self.instance = Some((*r).clone());
                }
            }
            
            self.instance.as_mut()
        }
    }

    pub fn create_instance_with_manager(&mut self, manager : &ResourceManager<T>)
    {
        if self.instance.is_none() {
            if let Some(i) = self.resource.get() {
                if let Some(r) = manager.get_as_ref(i) {
                    self.instance = Some((*r).clone());
                }
            }
            else {
                println!("TODO could not create instance");
            }
        }

    }
}

impl<T:fmt::Debug> fmt::Debug for ResTT<T>
{
    fn fmt(&self, fmt : &mut fmt::Formatter) -> fmt::Result
    {
        let s = if self.instance.is_none() {
            "no instance".to_owned()
        }
        else {
            format!("there is an instance: {:?}", self.instance)
        };


        write!(fmt, "ResTT Name :{} : resource :{:?} and {}", self.name, self.resource, s)
    }

}

pub trait Create
{
    fn create(name : &str) -> Self;
    fn inittt(&mut self);
}

impl Create for mesh::Mesh
{
    fn create(name : &str) -> mesh::Mesh
    {
        mesh::Mesh::new_from_file(name)
    }

    fn inittt(&mut self)
    {
        self.file_read();
    }
}

impl Create for material::Material
{
    fn create(name : &str) -> material::Material
    {
        material::Material::new(name)
    }

    fn inittt(&mut self)
    {
        //TODO
        self.read();
    }
}

impl Create for shader::Shader
{
    fn create(name : &str) -> shader::Shader
    {
        shader::Shader::new(name)
    }

    fn inittt(&mut self)
    {
        //TODO
        //self.read();
    }
}

impl Create for texture::Texture
{
    fn create(name : &str) -> texture::Texture
    {
        texture::Texture::new(name)
    }

    fn inittt(&mut self)
    {
        //TODO
        self.load();
    }
}

impl Create for fbo::Fbo
{
    fn create(name : &str) -> fbo::Fbo
    {
        fbo::Fbo::new(name)
    }

    fn inittt(&mut self)
    {
        //TODO
    }
}

impl Create for armature::Armature
{
    fn create(name : &str) -> armature::Armature
    {
        armature::Armature::new(name)
    }

    fn inittt(&mut self)
    {
        if self.state == 0 {
            self.file_read();
        }
    }
}

impl<T : Create> Create for Rc<RefCell<T>>
{
    fn create(name : &str) -> Rc<RefCell<T>>
    {
        Rc::new(RefCell::new(T::create(name)))
    }

    fn inittt(&mut self)
    {
        self.borrow_mut().inittt();
    }
}

pub enum State<T>
{
    Loading(Option<thread::JoinHandle<()>>,Arc<RwLock<Option<T>>>),
    Using(T),
}

impl<T> State<T>
{
    fn is_loading(&self) -> bool
    {
        if let State::Loading(_,_) = *self {
            true
        }
        else {
            false
        }
    }

    fn finalize(& self) -> (bool, Option<T>)
    {
        match *self {
            State::Loading(_, ref l) => {
                let is_some = {
                    let v : &Option<T> = &*l.read().unwrap();
                    v.is_some()
                };

                if is_some {
                    return (true, l.write().unwrap().take());
                }
                else {
                    (true, None)
                }
            },
            _ => {
                (false, None)
            }
        }
    }

    fn finalize_with_wait(&mut self) -> (bool, Option<T>)
    {
        match *self {
            State::Loading(ref mut ojh, ref l) => {
                let jh = ojh.take().unwrap();
                jh.join().expect("The thread could not join in finalize2");
                let is_some = {
                    let v : &Option<T> = &*l.read().unwrap();
                    v.is_some()
                };

                if is_some {
                    return (true, l.write().unwrap().take());
                }
                else {
                    (true, None)
                }
            },
            _ => {
                (false, None)
            }
        }
    }
}

pub struct ResourceManager<T>
{
    //usize is the index in loaded vec of the resource
    map : HashMap<String, usize>,
    loaded : Vec<State<T>>,

    //instanced : Vec<T>,

    // Other possible ways
    //map : HashMap<String, usize>, => saves index to ids, and id never change
    //ids : Vec<State>,
    
    // Dont need this for now but for reusing stuff :
    //if unused.is_enpty() {
    //  use next_id;
    //}
    //else {
    //  use unused[0]
    //}
    //next_id : usize,
    //unused : Vec<usize>
}

unsafe impl<T:Send> Send for ResourceManager<T> {}
unsafe impl<T:Sync> Sync for ResourceManager<T> {}


enum Test {
    None,
    Loading(usize),
    Other(usize)
}

impl<T:'static+Create+Sync+Send> ResourceManager<T> {
    pub fn new() -> ResourceManager<T>
    {
        ResourceManager {
            map : HashMap::new(),
            loaded : Vec::new(),
        }
    }

    pub fn loaded_iter(&self) -> 
        iter::FilterMap<Iter<State<T>>, fn(&State<T>) -> Option<&T>>
    {
        fn filter<A:'static+Create+Sync+Send>(s : &State<A>)
            -> Option<&A>
        {
            match *s {
                State::Loading(_,_) => {
                    None
                },
                State::Using(ref u) => {
                    Some(u)
                }
            }
        };

        self.loaded.iter().filter_map(filter)
    }

    pub fn loaded_iter_mut(&mut self) -> 
        iter::FilterMap<IterMut<State<T>>, fn(&mut State<T>) -> Option<&mut T>>
    {
        fn filter<A:'static+Create+Sync+Send>(s : &mut State<A>)
            -> Option<&mut A>
        {
            match *s {
                State::Loading(_,_) => {
                    None
                },
                State::Using(ref mut u) => {
                    Some(u)
                }
            }
        };

        self.loaded.iter_mut().filter_map(filter)
    }

    fn get_res_state(&self, name : &str) -> Test
    {
        match self.map.get(name) {
            None => {
                Test::None
            }
            Some(i) => {
                let li = &self.loaded[*i];

                if li.is_loading() {
                    Test::Loading(*i)
                }
                else {
                    Test::Other(*i)
                }
            }
        }
    }

    fn get_ref_from_name(&self, name : &str) -> Option<(usize,&T)>
    {
        match self.get_res_state(name) {
            Test::Other(i) => {
                let li = &self.loaded[i];
                match *li {
                    State::Using(ref u) => Some((i,u)),
                    _ => None
                }
            },
            _ => None

        }
    }

    pub fn get_or_load_mut_from_name(&mut self, name : &str) -> (usize, Option<&mut T>)
    {
        match self.get_res_state(name) {
            Test::None => {
                let index = self.loaded.len();
                let key = String::from(name);
                self.map.insert(key, index);
                self.load(name);
                (index, None)
            },
            Test::Loading(i) => {
                let li = &mut self.loaded[i];
                let (_, op) = li.finalize();
                if let Some(s) = op {
                    *li = State::Using(s);
                }

                match *li {
                    State::Using(ref mut u) => {
                        (i, Some(u))
                    },
                    _ => (i, None)
                }
            },
            Test::Other(i) => {
                let li = &mut self.loaded[i];
                match *li {
                    State::Using(ref mut u) => (i, Some(u)),
                    _ => (i, None)
                }
            }
        }
    }

    pub fn get_or_load_ref_from_name(&mut self, name : &str) -> (usize, Option<&T>)
    {
        match self.get_res_state(name) {
            Test::None => {
                let index = self.loaded.len();
                let key = String::from(name);
                self.map.insert(key, index);
                self.load(name);
                (index, None)
            },
            Test::Loading(i) => {
                let li = &mut self.loaded[i];
                let (_, op) = li.finalize();
                if let Some(s) = op {
                    *li = State::Using(s);
                }

                match *li {
                    State::Using(ref u) => {
                        (i, Some(u))
                    },
                    _ => (i, None)
                }
            },
            Test::Other(i) => {
                let li = &mut self.loaded[i];
                match *li {
                    State::Using(ref u) => (i, Some(u)),
                    _ => (i, None)
                }
            }
        }
    }


    fn load(&mut self, name : &str)
    {
        let s = String::from(name);

        let (tx, rx) = channel::<T>();
        let guard = thread::spawn(move || {
            let mut m : T = Create::create(s.as_ref());
            m.inittt();
            let result = tx.send(m);
        });

        let n = Arc::new(RwLock::new(None));
        let nn = n.clone();

        let join_handle = thread::spawn( move || {
            loop {
                match rx.try_recv() {
                    Err(_) => {},
                    Ok(value) =>  { 
                        let entry = &mut *nn.write().unwrap();
                        *entry = Some(value);
                        break; }
                }
            }
        });

        //let result = guard.join();
        self.loaded.push(State::Loading(Some(join_handle), n));
    }
    

    //TODO remove
    pub fn request_use_new(&mut self, name : &str, load : Arc<Mutex<usize>>)
        -> (usize, Option<&mut T>)
    {
        println!(">>>request use new :: {}", name);
        let key = String::from(name);

        let i : usize = match self.map.entry(key) {
            Vacant(entry) => {
                let index = self.loaded.len();
                entry.insert(index);
                println!("request use new :: {}, adding index {}", name, index);
                index
            }
            Occupied(entry) => {
                let i = *entry.get();
                let li = &mut self.loaded[i];
                let (was_loading, op) = li.finalize();
                println!("request use new :: {}, index {}, loading : {}", name, i, was_loading);
                if was_loading {
                    if let Some(s) = op {
                        println!("request use new :: {}, index {}, loading : {}, set to using!!!!!!!!", name, i, was_loading);
                        *li = State::Using(s);
                    }
                }

                match *li {
                    State::Using(ref mut u) => {
                        return (i, Some(u));
                    }
                    _ => {return (i, None); }
                }
            }
        };

        {
            let mut l = load.lock().unwrap();
            *l += 1;
            println!("      {}, ADDING {}",name,  *l);
        }

        let s = String::from(name);

        let (tx, rx) = channel::<T>();
        let guard = thread::spawn(move || {
            //thread::sleep(::std::time::Duration::seconds(5));
            //thread::sleep_ms(5000);
            println!(" thread creating {}", s);
            let mut m : T = Create::create(s.as_ref());
            m.inittt();
            println!(" thread creating {}, finish sending", s);
            let result = tx.send(m);
        });

        
        let s2 = String::from(name);

        let n = Arc::new(RwLock::new(None));
        let nn = n.clone();

        let join_handle = thread::spawn( move || {
            loop {
                match rx.try_recv() {
                    Err(_) => {},
                    Ok(value) =>  { 
                        let entry = &mut *nn.write().unwrap();
                        *entry = Some(value);
                        let mut l = load.lock().unwrap();
                        *l -= 1;
                        println!("     {} RECEIVED {}", s2, *l);
                        break; }
                }
            }
        });

        //let result = guard.join();
        self.loaded.push(State::Loading(Some(join_handle), n));

        assert!(self.loaded.len() -1 == i);

        (i, None)
    }

    pub fn request_use_no_proc_tt(&mut self, name : &str) -> ResTT<T>
    {
        let i = self.request_use_no_proc_new(name);
        ResTT::new_with_index(name, i)
    }

    pub fn get_handle_instant(&mut self, name : &str) -> ResTT<T>
    {
        let i = self.request_use_no_proc_new(name);
        ResTT::new_with_index(name, i)
    }

    //TODO
    pub fn request_use_no_proc_new(&mut self, name : &str) -> usize
    {
        let key = String::from(name);

        match self.map.entry(key) {
            Vacant(entry) => {
                let index = self.loaded.len();
                entry.insert(index);

                let mut m : T = Create::create(name);
                m.inittt();
                let s = State::Using(m);
                self.loaded.push(s);
                index
            }
            Occupied(entry) => {
                *entry.get()
            },
        }
    }

    //TODO put to private
    pub fn get_mut_or_panic(&mut self,index : usize) -> &mut T
    {
        match self.loaded[index] {
            State::Loading(_,_) => {
                panic!("should return an option");
            },
            State::Using(ref mut u) => {
                u
            }
        }
    }

    fn get_mut(&mut self, index : usize) -> Option<&mut T>
    {
        let li = &mut self.loaded[index];

        if let State::Using(ref mut u) = *li {
            return Some(u);
        }

        let (was_loading, op) = li.finalize();
        if was_loading {
            if let Some(s) = op {
                *li = State::Using(s);
            }
        }

        match *li {
            State::Using(ref mut u) => {
                return Some(u);
            }
            _ => {return None; }
        }
    }

    fn get_ref(&mut self, index : usize) -> Option<& T>
    {
        let li = &mut self.loaded[index];

        if let State::Using(ref u) = *li {
            return Some(u);
        }

        let (was_loading, op) = li.finalize();
        if was_loading {
            if let Some(s) = op {
                *li = State::Using(s);
            }
        }

        match *li {
            State::Using(ref u) => {
                return Some(u);
            }
            _ => {return None; }
        }
    }

    fn get_mut_instant(&mut self, index : usize) -> &mut T
    {
        let li = &mut self.loaded[index];

        if let State::Using(ref mut u) = *li {
            return u;
        }

        let (was_loading, op) = li.finalize_with_wait();
        if was_loading {
            if let Some(s) = op {
                *li = State::Using(s);
            }
        }

        match *li {
            State::Using(ref mut u) => {
                return u;
            }
            _ => {
                panic!(" why?????");
            }
        }
    }

    fn get_ref_instant(&mut self, index : usize) -> &T
    {
        self.get_mut_instant(index)
    }

    fn get_as_ref(&self,index : usize) -> Option<&T>
    {
        match self.loaded[index] {
            State::Loading(_,_) => {
                None
            },
            State::Using(ref u) => {
                Some(u)
            }
        }
    }

    pub fn get_as_mut(&mut self, index : usize) -> Option<&mut T>
    {
        match self.loaded[index] {
            State::Loading(_,_) => {
                None
            },
            State::Using(ref mut u) => {
                Some(u)
            }
        }
    }


    pub fn get_or_create(&mut self, name : &str) -> &mut T
    {
        let index = self.request_use_no_proc_new(name);
        self.get_mut_or_panic(index)
    }

}

impl<T:'static+Clone+Create+Sync+Send> ResourceManager<T> {
    pub fn request_use_no_proc_tt_instance(&mut self, name : &str) -> ResTT<T>
    {
        let i = self.request_use_no_proc_new(name);
        let mut t = ResTT::new_with_index(name, i);
        t.create_instance_with_manager(self);
        t
    }

}

pub fn resource_get<'a, T:'static+Create+Send+Sync>(
    manager : &'a mut ResourceManager<T>,
    res: &'a mut ResTT<T>,
    load : Arc<Mutex<usize>>
    )
    -> Option<&'a mut T>
{
    if res.instance.is_some() {
        res.instance.as_mut()
    }
    else if let Some(i) = res.resource.get() {
        manager.get_mut(i)
    }
    else {
        let (i, r) = manager.request_use_new(res.name.as_ref(), load);
        res.resource.set(Some(i));
        r
    }
}

pub fn resource_get_mut_no_instance<'a, T:'static+Create+Send+Sync>(
    manager : &'a mut ResourceManager<T>,
    res: &'a ResTT<T>,
    load : Arc<Mutex<usize>>
    )
    -> Option<&'a mut T>
{
    if let Some(i) = res.resource.get() {
        manager.get_mut(i)
    }
    else {
        let (i, r) = manager.request_use_new(res.name.as_ref(), load);
        res.resource.set(Some(i));
        r
    }
}


pub fn resource_get_ref<'a, T:'static+Create+Send+Sync>(
    manager : &'a mut ResourceManager<T>,
    res: &'a ResTT<T>,
    )
    -> Option<&'a T>
{
    if res.instance.is_some() {
        res.instance.as_ref()
    }
    else if let Some(i) = res.resource.get() {
        manager.get_ref(i)
    }
    else {
        let (i, r) = manager.get_or_load_ref_from_name(res.name.as_ref());
        res.resource.set(Some(i));
        r
    }
}

pub fn resource_get_mut<'a, T:'static+Create+Send+Sync>(
    manager : &'a mut ResourceManager<T>,
    res: &'a mut ResTT<T>,
    )
    -> Option<&'a mut T>
{
    if res.instance.is_some() {
        res.instance.as_mut()
    }
    else if let Some(i) = res.resource.get() {
        manager.get_mut(i)
    }
    else {
        let (i, r) = manager.get_or_load_mut_from_name(res.name.as_ref());
        res.resource.set(Some(i));
        r
    }
}



pub struct ResourceGroup
{
    pub mesh_manager : RefCell<ResourceManager<mesh::Mesh>>,
    pub shader_manager : RefCell<ResourceManager<shader::Shader>>,
    pub texture_manager : RefCell<ResourceManager<texture::Texture>>,
    pub material_manager : RefCell<ResourceManager<material::Material>>,
    pub fbo_manager : RefCell<ResourceManager<fbo::Fbo>>,
    pub armature_manager : RefCell<ResourceManager<armature::Armature>>,
}

impl ResourceGroup
{
    pub fn new() -> ResourceGroup
    {
        //let fbo_all = fbo_manager.request_use_no_proc("fbo_all");
        //let fbo_selected = fbo_manager.request_use_no_proc("fbo_selected");

        ResourceGroup {
            mesh_manager : RefCell::new(ResourceManager::new()),
            shader_manager : RefCell::new(ResourceManager::new()),
            texture_manager : RefCell::new(ResourceManager::new()),
            material_manager : RefCell::new(ResourceManager::new()),
            fbo_manager : RefCell::new(ResourceManager::new()),
            armature_manager : RefCell::new(ResourceManager::new()),
        }
    }
}
