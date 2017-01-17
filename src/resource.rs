use mesh;
use texture;
use shader;
use fbo;
use material;
use armature;
use camera;
use object;
use vec;
use transform;

use rustc_serialize::{Encodable, Encoder, Decoder, Decodable};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::sync::{RwLock, Arc, Mutex};
use std::sync::mpsc::channel;
//use std::time::Duration;
use self::ResTest::{ResData,ResWait,ResNone};
use std::thread;

use std::rc::Rc;
use std::cell::RefCell;
use uuid;


pub trait ResourceT  {
    fn init(&mut self);
}

pub enum ResTest<T>
{
    ResData(Arc<RwLock<T>>),
    ResWait,
    ResNone
}

impl<T:'static+Create+Sync+Send> ResTest<T> {

    fn get_or_load_instant(&mut self, name : &str) -> Arc<RwLock<T>>
    {
        match *self
        {
            ResNone | ResWait => {
                let mt : T = Create::create(name);
                let m = Arc::new(RwLock::new(mt));
                m.write().unwrap().inittt();

                *self = ResData(m.clone());
                return m.clone();
            },
            ResData(ref yep) => {
                return yep.clone();
            },
        }
    }
}

pub struct ResTT<T>
{
    pub name : String,
    pub resource : Option<usize>,
    pub instance : Option<T>,
}

impl<T:Create+Send+Sync+'static> ResTT<T>
{
    pub fn new(name : &str) -> ResTT<T>
    {
        ResTT {
            name : String::from(name),
            resource : None,
            instance : None
        }
    }

    pub fn new_instant(name : &str, rm : &mut ResourceManager<T>) -> ResTT<T>
    {
        let mut r = ResTT::new(name);
        r.load_instant(rm);

        r
    }

    pub fn new_with_res(name : &str, res : T) -> ResTT<T>
    {
        ResTT {
            name : String::from(name),
            resource : None,
            instance : Some(res),
        }
    }
}

impl<T> Clone for ResTT<T>
{
    fn clone(&self) -> ResTT<T>
    {
        ResTT {
            name : self.name.clone(),
            resource : self.resource.clone(),
            instance : None //TODO
        }
    }
}

impl <T:'static+Create+Send+Sync> ResTT<T>
{
    pub fn get_resource<'a>(&mut self, manager : &'a mut ResourceManager<T>, load : Arc<Mutex<usize>> ) -> Option<&'a mut T>
    {
        resource_get(manager, self, load)
    }

    fn load_instant(&mut self, manager : &mut ResourceManager<T> )
    {
        self.resource = Some(manager.request_use_no_proc_new(self.name.as_ref()));
    }

    pub fn load_instant_no_manager(&mut self)
    {
        if self.instance.is_none() {
            let mut mt : T = Create::create(self.name.as_ref());
            mt.inittt();
            self.instance = Some(mt);
        }
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
        if self.state == 0 {
            //TODO can be read anywhere
            self.file_read();
        }
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

impl Create for camera::Camera
{
    fn create(name : &str) -> camera::Camera
    {
        println!("review this of course");
        let o = object::Object {
            name : String::from("camera"),
            id : uuid::Uuid::new_v4(),
            mesh_render : None,
            position : vec::Vec3::zero(),
            //orientation : vec::Quat::identity(),
            orientation : transform::Orientation::new_quat(),
            //angles : vec::Vec3::zero(),
            scale : vec::Vec3::one(),
            children : Vec::new(),
            parent : None,
            //transform : box transform::Transform::new()
            components : Vec::new(),
            comp_data : Vec::new(),
            comp_string : Vec::new(),
            comp_lua : Vec::new(),
        };

        camera::Camera {
            data : Default::default(),
            object : Arc::new(RwLock::new(o)),
            id : uuid::Uuid::new_v4(),
            object_id : None
        }
    }

    fn inittt(&mut self)
    {
    }
}

#[derive(Clone,Copy)]
pub enum StateOld
{
    Loading(usize),
    Using(usize),
}

pub enum State<T>
{
    Loading(Arc<RwLock<Option<T>>>),
    Using(T),
}

impl<T> State<T>
{
    /*
    fn is_loading(&self) -> bool
    {
        match self {
            is_loading

        }
    }
    */

    fn finalize(& self) -> (bool, Option<T>)
    {
        match *self {
            State::Loading(ref l) => {
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
    pub resources : HashMap<String, Arc<RwLock<ResTest<T>>>>,

    //TODO new style (wip)
    map : HashMap<String, usize>,
    // really use arc/rwlock?
    // rwlock just needed when resource is not done loading and we will still write.
    res : Vec<Arc<RwLock<ResTest<T>>>>,
    loaded : Vec<State<T>>,

    //TODO
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

type ReceiveResource<T> = fn(ResTest<T>);

impl<T:'static+Create+Sync+Send> ResourceManager<T> {
    pub fn new() -> ResourceManager<T>
    {
        ResourceManager {
            resources : HashMap::new(),

            map : HashMap::new(),
            res : Vec::new(),
            loaded : Vec::new(),
        }
    }

    pub fn request_use_old(&mut self, name : &str, load : Arc<Mutex<usize>>) -> ResTest<T>
    {
        let key = String::from(name);

        let va : Arc<RwLock<ResTest<T>>> = match self.resources.entry(key) {
            Entry::Vacant(entry) => entry.insert(Arc::new(RwLock::new(ResTest::ResNone))).clone(),
            Entry::Occupied(entry) => entry.into_mut().clone(),
        };

        {
            let v : &mut ResTest<T> = &mut *va.write().unwrap();

            match *v {
                ResTest::ResData(ref yep) => {
                    return ResTest::ResData(yep.clone());
                },
                ResTest::ResWait => {
                    return ResTest::ResWait;
                },
                ResTest::ResNone => {
                    *v = ResTest::ResWait;
                },
            }
        }

        {
            let mut l = load.lock().unwrap();
            *l += 1;
            println!("      ADDING {}", *l);
        }

        let s = String::from(name);

        let (tx, rx) = channel::<Arc<RwLock<T>>>();
        let guard = thread::spawn(move || {
            //thread::sleep(::std::time::Duration::seconds(5));
            //thread::sleep_ms(5000);
            let mt : T = Create::create(s.as_ref());
            let m = Arc::new(RwLock::new(mt));
            m.write().unwrap().inittt();
            let result = tx.send(m.clone());
        });

        //let result = guard.join();

        thread::spawn( move || {
            loop {
                match rx.try_recv() {
                    Err(_) => {},
                    Ok(value) =>  { 
                        let entry = &mut *va.write().unwrap();
                        *entry = ResTest::ResData(value.clone());
                        let mut l = load.lock().unwrap();
                        *l -= 1;
                        println!("      SUBBBBBB {}", *l);
                        break; }
                }
            }
        });

        return ResTest::ResWait;


    }

    pub fn request_use(&mut self, name : &str, load : Arc<Mutex<usize>>) -> ResTest<T>
    {
        panic!("dance");
    }

    /*
        let key = String::from(name);

        let va : Arc<RwLock<ResTest<T>>> = match self.map.entry(key) {
            Vacant(entry) => {
                let n = Arc::new(RwLock::new(ResNone));
                entry.insert(State::Loading(n.clone()));
                //self.res.push(n.clone());
                n
            }
            Occupied(entry) => self.res[entry.get().index()].clone(),
        };

        {
            let v : &mut ResTest<T> = &mut *va.write().unwrap();

            match *v {
                ResTest::ResData(ref yep) => {
                    return ResTest::ResData(yep.clone());
                },
                ResTest::ResWait => {
                    return ResTest::ResWait;
                },
                ResTest::ResNone => {
                    *v = ResTest::ResWait;
                },
            }
        }

        {
            let mut l = load.lock().unwrap();
            *l += 1;
            println!("      ADDING {}", *l);
        }

        let s = String::from(name);

        let (tx, rx) = channel::<Arc<RwLock<T>>>();
        let guard = thread::spawn(move || {
            //thread::sleep(::std::time::Duration::seconds(5));
            //thread::sleep_ms(5000);
            let mt : T = Create::create(s.as_ref());
            let m = Arc::new(RwLock::new(mt));
            m.write().unwrap().inittt();
            let result = tx.send(m.clone());
        });

        //let result = guard.join();

        thread::spawn( move || {
            loop {
                match rx.try_recv() {
                    Err(_) => {},
                    Ok(value) =>  { 
                        let entry = &mut *va.write().unwrap();
                        *entry = ResTest::ResData(value.clone());
                        let mut l = load.lock().unwrap();
                        *l -= 1;
                        println!("      SUBBBBBB {}", *l);
                        break; }
                }
            }
        });

        return ResTest::ResWait;


    }
    */

    pub fn request_use_new(&mut self, name : &str, load : Arc<Mutex<usize>>) -> (usize, Option<&mut T>)
    {
        let key = String::from(name);

        let (i,va) : (usize, Arc<RwLock<Option<T>>>) = match self.map.entry(key) {
            Vacant(entry) => {
                let index = self.loaded.len();
                entry.insert(index);
                let n = Arc::new(RwLock::new(None));
                self.loaded.push(State::Loading(n.clone()));
                (index, n)
            }
            Occupied(entry) => {
                let i = *entry.get();
                let li = &mut self.loaded[i];
                let (was_loading, op) = li.finalize();
                if was_loading {
                    if let Some(s) = op {
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
            println!("      ADDING {}", *l);
        }

        let s = String::from(name);

        let (tx, rx) = channel::<T>();
        let guard = thread::spawn(move || {
            //thread::sleep(::std::time::Duration::seconds(5));
            //thread::sleep_ms(5000);
            let mut m : T = Create::create(s.as_ref());
            m.inittt();
            let result = tx.send(m);
        });

        //let result = guard.join();

        thread::spawn( move || {
            loop {
                match rx.try_recv() {
                    Err(_) => {},
                    Ok(value) =>  { 
                        let entry = &mut *va.write().unwrap();
                        *entry = Some(value);
                        let mut l = load.lock().unwrap();
                        *l -= 1;
                        println!("      SUBBBBBB {}", *l);
                        break; }
                }
            }
        });

        (i, None)
    }

    pub fn arequest_use_copy_test<F>(&mut self, name : &str, on_ready : F) -> ResTest<T>
        //where F : Fn(), F:Send +'static
        where F : Fn() + Send + 'static + Sync
    {
        let key = String::from(name);

        let va : Arc<RwLock<ResTest<T>>> = match self.resources.entry(key) {
            Entry::Vacant(entry) => entry.insert(Arc::new(RwLock::new(ResTest::ResNone))).clone(),
            Entry::Occupied(entry) => entry.into_mut().clone(),
        };

        {
            let v : &mut ResTest<T> = &mut *va.write().unwrap();

            match *v {
                ResTest::ResData(ref yep) => {
                    return ResTest::ResData(yep.clone());
                },
                ResTest::ResWait => {
                    return ResTest::ResWait;
                },
                ResTest::ResNone => {
                    *v = ResTest::ResWait;
                },
            }
        }

        let s = String::from(name);

        let (tx, rx) = channel::<Arc<RwLock<T>>>();
        let guard = thread::spawn(move || {
            //thread::sleep(::std::time::Duration::seconds(5));
            //thread::sleep_ms(5000);
            let mt : T = Create::create(s.as_ref());
            let m = Arc::new(RwLock::new(mt));
            m.write().unwrap().inittt();
            let result = tx.send(m.clone());
        });

        //let result = guard.join();

        thread::spawn( move || {
            loop {
                match rx.try_recv() {
                    Err(_) => {},
                    Ok(value) =>  { 
                        let entry = &mut *va.write().unwrap();
                        *entry = ResTest::ResData(value.clone());
                        on_ready();
                        break; }
                }
            }
        });

        return ResTest::ResWait;


    }


        //TODO wip
        /*
    pub fn request_use_and_call<F>(&mut self, name : &str, f : F) 
        -> ResTest<T> where F : Fn(ResTest<T>), F:Send +'static
    {
        let ms1 = self.resources.clone();
        let mut ms1w = ms1.write().unwrap();

        let key = String::from(name);

        let v : &mut ResTest<T> = match ms1w.entry(key) {
        //let v : &mut ResTest<T> = match ms1w.entry(&s) {
            Entry::Vacant(entry) => entry.insert(ResTest::ResNone),
            Entry::Occupied(entry) => entry.into_mut(),
        };

        let s = String::from(name);
        let msc = self.resources.clone();

        match *v 
        {
            ResNone => {
                *v = ResTest::ResWait;

                let ss = s.clone();

                let (tx, rx) = channel::<Arc<RwLock<T>>>();
                let guard = thread::scoped(move || {
                    //sleep(::std::time::duration::Duration::seconds(5));
                    let mt : T = Create::create(ss.as_ref());
                    let m = Arc::new(RwLock::new(mt));
                    m.write().unwrap().inittt();
                    let result = tx.send(m.clone());
                });

                let result = guard.join();

                thread::spawn( move || {
                    loop {
                    match rx.try_recv() {
                        Err(_) => {},
                        Ok(value) =>  { 
                            let mut mscwww = msc.write().unwrap();
                            let rd = ResTest::ResData(value.clone());
                            f(rd);

                            match mscwww.entry(s.clone()) {
                                //Entry::Vacant(entry) => entry.insert(ResTest::ResNone),
                                Entry::Vacant(entry) => entry.insert(ResTest::ResData(value.clone())),
                                Entry::Occupied(mut entry) => { 
                                    *entry.get_mut() = ResTest::ResData(value.clone());
                                    entry.into_mut()
                                }
                            };

                            break; }
                    }
                    }
                });

                return ResTest::ResWait;
            },
            ResTest::ResData(ref yep) => {
                return ResTest::ResData(yep.clone());
            },
            ResTest::ResWait => {
                return ResTest::ResWait;
            }
        }
    }
    */

    pub fn request_use_no_proc_old(&mut self, name : &str) -> Arc<RwLock<T>>
    {
        let key = String::from(name);

        let va : Arc<RwLock<ResTest<T>>> = match self.resources.entry(key) {
            Vacant(entry) => entry.insert(Arc::new(RwLock::new(ResNone))).clone(),
            Occupied(entry) => entry.into_mut().clone(),
        };

        let v : &mut ResTest<T> = &mut *va.write().unwrap();

        match *v 
        {
            ResNone | ResWait => {
                let mt : T = Create::create(name);
                let m = Arc::new(RwLock::new(mt));
                m.write().unwrap().inittt();

                *v = ResData(m.clone());
                return m.clone();
            },
            ResData(ref yep) => {
                return yep.clone();
            },
        }
    }

    pub fn request_use_no_proc(&mut self, name : &str) -> Arc<RwLock<T>>
    {
        self.request_use_no_proc_old(name)
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


    pub fn get_from_index(&self,index : usize) -> Arc<RwLock<T>>
    {
        let r = &self.res[index];
        match *r.read().unwrap()
        {
            ResNone | ResWait => {
                //TODO we should not need to panic as calling this function means you know you already
                // loaded it, there should be be wait stuff only already loaded data to look into.
                panic!("it is not loaded yet");
            },
            ResData(ref yep) => {
                return yep.clone();
            },
        }

    }

    pub fn get_from_index2(&mut self,index : usize) -> &mut T
    {
        match self.loaded[index] {
            State::Loading(_) => {
                panic!("should return an option");
            },
            State::Using(ref mut u) => {
                u
            }
        }
    }


    pub fn get_or_create(&mut self, name : &str) -> Option<&mut T>
    {
        let index = self.request_use_no_proc_new(name);
        Some(self.get_from_index2(index))
    }
}

//#[deriving(Decodable, Encodable)]
/*
pub struct ResourceRef
{
    pub name : String,
    pub resource : Resource
}
*/

impl <T> Encodable for ResTT<T> {
    fn encode<S: Encoder>(&self, encoder: &mut S) -> Result<(), S::Error> {
        encoder.emit_struct("NotImportantName", 1, |encoder| {
            try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.name.encode(encoder)));
            Ok(())
        })
    }
}

impl<T> Decodable for ResTT<T> {
    fn decode<D : Decoder>(decoder: &mut D) -> Result<ResTT<T>, D::Error> {
        decoder.read_struct("root", 0, |decoder| {
            Ok(
                ResTT{
                    name : try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
                    resource : None,
                    instance : None
                }
              )
        })
    }
}

/*
pub fn resource_get<T:'static+Create+Send+Sync>(
    manager : &mut ResourceManager<T>,
    res: &mut ResTT<T>,
    load : Arc<Mutex<usize>>
    )
    -> Option<Arc<RwLock<T>>>
{
    let mut the_res : Option<Arc<RwLock<T>>> = None;
    match res.resource{
        ResNone | ResWait => {
            res.resource = manager.request_use(res.name.as_ref(), load);
            match res.resource {
                ResData(ref data) => {
                    the_res = Some(data.clone());
                }
                _ => {}
            }
        },
        ResData(ref data) => {
            the_res = Some(data.clone());
        },
    }

    the_res
}
*/

pub fn resource_get<'a, T:'static+Create+Send+Sync>(
    manager : &'a mut ResourceManager<T>,
    res: &mut ResTT<T>,
    load : Arc<Mutex<usize>>
    )
    -> Option<&'a mut T>
{
    if let Some(i) = res.resource {
        Some(manager.get_from_index2(i))
    }
    else {
        let (i, r) = manager.request_use_new(res.name.as_ref(), load);
        res.resource = Some(i);
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
