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


/*
//#[deriving(Decodable, Encodable)]
pub enum Resource {
    Mesh(mesh::Mesh),
    //Shader(shader::Material)
}

pub struct ResourceS
{
    state : int,
    data : Resource
}
*/

pub trait ResourceT  {
    fn init(&mut self);
}

pub enum ResTest<T>
{
    ResData(Arc<RwLock<T>>),
    ResData2(T),
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
            _ => {
                panic!("yes");
            },
        }
    }

    fn get_or_load_instant_no_arc(&mut self, name : &str)
    {
        match *self
        {
            ResNone | ResWait => {
                let mut m : T = Create::create(name);
                m.inittt();

                *self = ResTest::ResData2(m);
                //return &m;
            },
            ResTest::ResData2(ref yep) => {
                //return yep;
            },
            _ => {
                panic!("yes");
            },
        }
    }


}

pub struct ResTT<T>
{
    pub name : String,
    pub resource : ResTest<T>,
    pub state : Option<State>
}

impl<T:Create+Send+Sync+'static> ResTT<T>
{
    pub fn new(name : &str) -> ResTT<T>
    {
        ResTT {
            name : String::from(name),
            resource : ResTest::ResNone,
            state : None
        }
    }

    pub fn new_instant(name : &str, rm : &mut ResourceManager<T>) -> ResTT<T>
    {
        let mut r = ResTT::new(name);
        r.load_instant(rm);

        r
    }

    pub fn new_with_res(name : &str, res : ResTest<T>) -> ResTT<T>
    {
        ResTT {
            name : String::from(name),
            resource : res,
            state : None
        }
    }
}

impl<T> Clone for ResTT<T>
{
    fn clone(&self) -> ResTT<T>
    {
        let r = match self.resource {
            ResData(ref d) => ResData(d.clone()),
            ResWait => ResWait,
            ResNone => ResNone,
            ResTest::ResData2(_) => panic!("can't clone this"),
        };

        ResTT {
            name : self.name.clone(),
            resource : r,
            state : None
        }
    }
}

impl <T:'static+Create+Send+Sync> ResTT<T>
{
    pub fn get_resource(&mut self, manager : &mut ResourceManager<T>, load : Arc<Mutex<usize>> ) -> Option<Arc<RwLock<T>>>
    {
        match self.resource {
            ResTest::ResData(ref rd) => Some(rd.clone()),
            //ResTest::ResWait => None,
            _ => resource_get(manager, self, load)
        }
    }

    fn load_instant(&mut self, manager : &mut ResourceManager<T> )
    {
        match self.resource {
            ResNone | ResWait => {
                let data = manager.request_use_no_proc(self.name.as_ref());
                self.resource = ResTest::ResData(data);
            },
            _ => {}
        }
    }

    pub fn load_instant_no_manager(&mut self)
    {
        match self.resource {
            ResNone | ResWait => {
                let mut mt : T = Create::create(self.name.as_ref());
                mt.inittt();
                let data = Arc::new(RwLock::new(mt));
                self.resource = ResTest::ResData(data);
            },
            _ => {}
        }
    }


    pub fn get_resource_instant(&mut self, manager : &mut ResourceManager<T> ) -> Arc<RwLock<T>>
    {
        match self.resource {
            ResTest::ResData(ref rd) => rd.clone(),
            _ => {
                let data = manager.request_use_no_proc(self.name.as_ref());
                self.resource = ResTest::ResData(data.clone());
                data
            }
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
pub enum State
{
    Loading(usize),
    Using(usize),
}

impl State
{
    fn index(&self) -> usize
    {
        match *self {
            State::Loading(i) => i,
            State::Using(i) => i
        }
    }
}

pub struct ResourceManager<T>
{
    pub resources : HashMap<String, Arc<RwLock<ResTest<T>>>>,

    //TODO new style (wip)
    map : HashMap<String, State>,
    // really use arc/rwlock?
    // rwlock just needed when resource is not done loading and we will still write.
    res : Vec<Arc<RwLock<ResTest<T>>>>,
    loaded : Vec<T>,

    //TODO
    //map : HashMap<String, usize>, => saves index to ids, and id never change
    //ids : Vec<State>,
    //next_id : usize
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
            loaded : Vec::new()
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
                ResTest::ResData2(ref yep) => {
                    panic!("erase me");
                }
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
        let key = String::from(name);

        let va : Arc<RwLock<ResTest<T>>> = match self.map.entry(key) {
            Vacant(entry) => {
                let index = self.res.len();
                entry.insert(State::Loading(index));
                let n = Arc::new(RwLock::new(ResNone));
                self.res.push(n.clone());
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
                ResTest::ResData2(_) => {
                    panic!("erase me");
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

    /*
    pub fn request_use_new(&mut self, name : &str, load : Arc<Mutex<usize>>) -> (State, Option<&T)
    {
        let key = String::from(name);

        let va : Arc<RwLock<ResTest<T>>> = match self.map.entry(key) {
            Vacant(entry) => {
                let index = self.res.len();
                entry.insert(State::Loading(index));
                let n = Arc::new(RwLock::new(ResNone));
                self.res.push(n.clone());
                n
            }
            Occupied(entry) => {
                let s = match entry.get();

                match s {
                    State::Loading(i) => {
                        let r = self.res[i];
                        //self.res[entry.get().index()].clone()
                        r
                    },
                    State::Using(i) => {
                        return (s, &mut self.loaded[i]);
                    }
                }
            }
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
                ResTest::ResData2(_) => {
                    panic!("erase me");
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
                ResTest::ResData2(_) => {
                    panic!("erase me");
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
            ResTest::ResData2(_) => {
                panic!("old, erase me");
            }
        }
    }

    pub fn request_use_no_proc(&mut self, name : &str) -> Arc<RwLock<T>>
    {
        self.request_use_no_proc_old(name)
    }


    /*
    pub fn request_use_no_proc_no_arc(&mut self, name : &str) -> &T
    {
        let index = self.request_use_no_proc_new(name);

        match *self.res[index].read().unwrap
        {
            ResTest::ResData2(ref yep) => {
                yep
            },
            _ => {
                panic!("yes");
            },
        }
    }
    */

    //TODO
    pub fn request_use_no_proc_new(&mut self, name : &str) -> State
    {
        let key = String::from(name);

        match self.map.entry(key) {
            Vacant(entry) => {
                let s = State::Using(self.loaded.len());
                entry.insert(s);

                let mut m : T = Create::create(name);
                m.inittt();
                self.loaded.push(m);
                s
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
            _ => {
                panic!("erase me");
                //return yep.clone();
            },
        }

    }

    pub fn get_from_state(&mut self, state : State) -> &mut T
    {
        match state
        {
            State::Using(i) =>
                &mut self.loaded[i],
            _ => {
                panic!("erase me");
                //return yep.clone();
            },
        }
    }

    pub fn get_res(&mut self, state : State) -> Option<&mut T>
    {
        match state
        {
            State::Using(i) =>
                Some(&mut self.loaded[i]),
            _ => {
                 None
            },
        }
    }

    pub fn get_or_create(&mut self, name : &str) -> Option<&mut T>
    {
        let state = self.request_use_no_proc_new(name);
        self.get_res(state)
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
                    resource : ResNone,
                    state : None,
                }
              )
        })
    }
}

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
        _ => panic!("erase me")
    }

    the_res
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
