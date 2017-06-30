use vec;
use matrix;
use transform;
use shader;
use resource;
use material;
use component;
use input;
use component::{Component,CompData, Components};
use component::mesh_render;
use mesh;

use std::sync::{RwLock, Arc};//,RWLockReadGuard};
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::collections::HashMap;
use uuid;
use uuid::Uuid;
use core::marker::Sized;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

use std::sync::mpsc::channel;
use std::mem;

use std::io::{self, Write};
use std::path::Path;
use lua;
use libc::c_void;


#[derive(Serialize, Deserialize)]
pub struct Object
{
    // Object Architecture
    // object/world architecture is still not well defined... comments are here for thinking...
    
    // - There was an idea before where we could have an Object separated into 2 :
    // The object that we write to a file and the object instance that we use in the world.
    // it was just an idea and this comment can be scrapped if it is not relevant anymore.
    

    // - if we use just an id or an index.
    

    // Problems to always have in mind if/when changing object architecture
    // - how an object reference another.
    // - graph : relations between parents and children
    // - how we de/serialize objects
    //      - how an object is represented on the disk
    //      - how an object is loaded from that representation and created in the world.
    //          - it needs to go into a load state and then an initial state
    //          - we should always be able to reset the object to this initial state (without
    //          loading of course)
    // - what happens when we want to remove an object
    //      - what we do with his components if they are stored somewhere else
    // - how do we updates objects and components
    //      - when we want to update a component, how do we get a reference to
    //          - this object components.
    //          - other objects components : for example doing a search like objects with enemy
    //          components that are around me etc.
    
    // An object (and by extension the world) can exist in multiple forms
    // - on the disk, on files, serialized.
    // - loading state
    // - loaded
    //      - init state (must be saved)
    //      - living

    //Identification
    pub name : String,
    pub id : uuid::Uuid,

    // Transform
    //pub transform : Box<transform::Transform>
    pub position : vec::Vec3,
    pub orientation : transform::Orientation,
    pub scale : vec::Vec3,
    #[serde(skip_serializing, skip_deserializing)]
    pub children : Vec<Arc<RwLock<Object>>>,
    #[serde(skip_serializing, skip_deserializing)]
    pub parent : Option<Arc<RwLock<Object>>>,

    // Components
    #[serde(skip_serializing, skip_deserializing)]
    pub mesh_render : Option<mesh_render::MeshRender>,
    #[serde(skip_serializing, skip_deserializing)]
    pub components : Vec<Box<Components>>,
    pub comp_data : Vec<Box<CompData>>,
    pub comp_string : Vec<String>,
    pub comp_lua : Vec<String>,
}

unsafe impl Send for Object {}
unsafe impl Sync for Object {}

impl Clone for Object {

    fn clone(&self) -> Object {
        let mut components = Vec::new();
        for c in self.components.iter() {
            let cc = c.clone();
            components.push(cc);
        }
        let comp_data = self.comp_data.clone();
        Object {
            name : self.name.clone(),
            id : self.id.clone(),//?
            mesh_render : if let Some(ref mr) = self.mesh_render {
                Some(mr.clone())
            } else {None},
            position : self.position.clone(),
            orientation : self.orientation.clone(),
            scale : self.scale.clone(),
            children : self.children.clone(), 
            parent : self.parent.clone(), //None,
            //transform : box transform::Transform::new()
            components : components,
            comp_data : comp_data,
            comp_string : self.comp_string.clone(),
            comp_lua : self.comp_lua.clone(),
        }
    }
}

impl Object
{
    pub fn new(name : &str) -> Object
    {
        Object {
            name : String::from(name),
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
        }
    }

    pub fn get_matrix(&self) -> matrix::Matrix4
    {
        //TODO optim
        let mt = matrix::Matrix4::translation(&self.position);
        let mq = matrix::Matrix4::rotation(&self.orientation.as_quat());
        let ms = matrix::Matrix4::scale(&self.scale);

        &(&mt * &mq) * &ms
    }

    pub fn get_world_matrix(&self) -> matrix::Matrix4
    {
        //TODO optim
        let mt = matrix::Matrix4::translation(&self.world_position());
        let mq = matrix::Matrix4::rotation(&self.world_orientation());
        let ms = matrix::Matrix4::scale(&self.world_scale());

        &(&mt * &mq) * &ms
    }


    /*
    pub fn child_add(&mut self, child : Arc<RwLock<Object>>)
    {
        self.children.push(child);
        child.write().parent = Some(self.clone());
    }
    */


    pub fn world_position(&self) -> vec::Vec3
    {
        match self.parent {
            None => return self.position,
            Some(ref parent) => {
                let wo = parent.read().unwrap().world_orientation();
                let p = wo.rotate_vec3(&self.position);
                return p + parent.read().unwrap().world_position();
            }
        }
    }

    pub fn world_orientation(&self) -> vec::Quat
    {
        match self.parent {
            None => return self.orientation.as_quat(),
            Some(ref p) => {
                return p.read().unwrap().world_orientation() * self.orientation.as_quat();
            }
        }
    }

    pub fn world_scale(&self) -> vec::Vec3
    {
        match self.parent {
            None => return self.scale,
            Some(ref p) => {
                return self.scale * p.read().unwrap().world_scale();
                //return self.scale.mul(p.read().unwrap().world_scale());
            }
        }
    }

    //TODO remove
    pub fn set_uniform_data(
        &mut self,
        name : &str,
        data : shader::UniformData)
    {
        let render = match self.mesh_render {
            Some(ref mut r) => r,
            None => return
        };

        //render.material.write().unwrap().set_uniform_data(name, data);
        render.material.get_instance().unwrap().set_uniform_data(name, data);
    }

    //pub fn get_material(&self) -> Option<Arc<RwLock<material::Material>>>
    pub fn get_material(&mut self) -> Option<&mut material::Material>
    {
        let render = match self.mesh_render {
            Some(ref mut r) => r,
            None => return None
        };

        //Some(render.material.clone())
        render.material.get_instance()
    }

    fn luastuff(&mut self, dt : f64)
    {
        let mut lua = lua::State::new();
        lua.openlibs();
        //lua.registerlib(Some("object"),[("print_ob", print_ob)]);
        let yop = &[
            ("print_ob", print_ob as lua::CFunction),
            ("get_pos", get_pos as lua::CFunction),
            ("__to_string", object_string as lua::CFunction),
            //("__index", object_index as lua::CFunction),
        ];

        let meta = &[
            ("__newindex", object_newindex as lua::CFunction),
            //("__index", object_index as lua::CFunction),
            ("__tostring", tostring as lua::CFunction),
        ];

        /*
        fn set_number(lua : lua::State, f : *mut f64) -> i32
        {
            f = lua.checknumber(L, 3);
            0
        }


        let setters = &[
        ("x",  set_number,    offsetof(your_t,age)  ),
        ("y",    set_number, offsetof(your_t,x)    },
        ("z",    set_number, offsetof(your_t,y)    },
        {0,0}
        };
        */

        lua.registerlib(Some("object"),yop);
        let methods = lua.gettop();

        if lua.newmetatable("object") {
            lua.registerlib(None, meta);
        }
        let metatable = lua.gettop();

        //hide metatable
        {
            lua.pushstring("__metatable");
            lua.pushvalue(methods);
            lua.rawset(metatable);
        }

        {
            lua.pushstring("__index");
            lua.pushvalue(metatable);
            lua.pushvalue(methods);
            lua.pushcclosure(index_handler,2);
            lua.rawset(metatable);
        }

/*

        lua.pushstring("__index");
        lua.pushvalue(-2);
        lua.rawset(-3); //lua.settable(-3);
        */


        //lua.pushstring("__metatable");
        //lua.pushvalue(-2);              
        ////lua.settable(-3);// lua.rawset(-3);  
        //lua.rawset(-3); //lua.settable(-3);

        /*
        lua.pushstring("__newindex");
        lua.newtable();              /* table for members you can set */
        Xet_add(L, your_setters);     /* fill with setters */
        lua.pushcclosure(newindex_handler, 1);
        lua_rawset(L, metatable);
        */


        //lua.registerlib(None, meta);
        //lua.registerlib(Some("object"),yop);

        lua.pop(1);

        create_vec3_metatable(&mut lua);

        for i in 0..self.comp_lua.len() {
            let name = self.comp_lua[i].clone();
            self.update_lua_script(dt, &mut lua, name.as_str());
        }

        /*
        let ll = self.comp_lua.clone();
        for s in &ll {
            self.update_lua_script(dt, &mut lua, s);
        }
        */

    }

    fn update_lua_script(&mut self, dt : f64, lua : &mut lua::State, lua_file : &str)
    {
        let path = Path::new(lua_file);
        match lua.loadfile(Some(&path)) {
            Ok(_) => (),
            Err(_) => {
                // If something went wrong, error message is at the top of the stack
                let _ = writeln!(&mut io::stderr(),
                "Couldn't load file: {}", lua.describe(-1));
            }
        }

        /*
         * Ok, now here we go: We pass data to the lua script on the stack.
         * That is, we first have to prepare Lua's virtual stack the way we
         * want the script to receive it, then ask Lua to run it.
         */
        lua.newtable(); // We will pass a table
        lua.pushstring("ob");
        {
            let ptr : &*mut c_void = unsafe { mem::transmute(&self) };
            lua.pushlightuserdata(*ptr);
            {
                lua.getmetatable_reg("object");
                lua.setmetatable(-2);
            }
        }
        lua.rawset(-3);       // Stores the pair in the table

        // By what name is the script going to reference our table?
        lua.setglobal("foo");

        match lua.pcall(0, lua::MULTRET, 0) {
            Ok(()) => (),
            Err(_) => {
                let _ = writeln!(&mut io::stderr(),
                "Failed to run script: {}", lua.describe(-1));
            }
        }


    }

    pub fn update(
        &mut self,
        dt : f64,
        input : &input::Input,
        resource : &resource::ResourceGroup)
    {
        //self.luastuff(dt);

        let len = self.components.len();

        let mut index = 0;
        loop {
            if index >= self.components.len() {
                break;
            }

            self.components.push(box Components::Empty);
            let mut c = self.components.swap_remove(index);
            c.update(self, dt, input, resource);
            self.components[index] = c;
            index = index +1;
        }
    }

    pub fn add_component(&mut self, c : Box<Components>)
    {
        self.components.push(c);
    }

    pub fn add_comp_string(&mut self, c : &str) 
    {
        self.comp_string.push(c.to_string());

    }

    pub fn add_comp_data(&mut self, c : Box<CompData>)
    {
        self.comp_data.push(c);
        //let (tx, rx) = channel();
    }

    pub fn remove_comp_data(&mut self, c : Box<CompData>)
    {
        self.comp_data.retain(|cd| cd.get_kind_string() != c.get_kind_string());
        //let (tx, rx) = channel();
    }

    pub fn get_comp_data_value<T:Any+Clone>(&self) -> Option<T>
    {
        for c in self.comp_data.iter()
        {
             if let Some(s) = c.get_comp::<T>() {
                 return Some((*s).clone());
             }
        }
        None
    }


    pub fn get_comp_data<T:Any>(&self) -> Option<&T>
    {
        for c in self.comp_data.iter()
        {
             if let Some(s) = c.get_comp::<T>() {
                 return Some(s);
             }
             //if cd.unwrap().is::<T>() {
             //}
        }
        None
    }

    pub fn get_mut_comp_data<T:Any>(&mut self) -> Option<&mut T>
    {
        for c in self.comp_data.iter_mut()
        {
             if let Some(s) = c.get_mut_comp::<T>() {
                 return Some(s);
             }
             //if cd.unwrap().is::<T>() {
             //}
        }
        None
    }

    pub fn init_components(&mut self, comp_mgr : &component::Manager, resource : &resource::ResourceGroup)
    {
        let mut comps = Vec::new();

        for c in self.comp_string.iter() {
            let f = comp_mgr.get_component_create_fn(c.as_ref()).unwrap();
            let pc = f(self, resource);
            comps.push(pc);
        }

        self.components = comps;

        for child in self.children.iter()
        {
            child.write().unwrap().init_components(comp_mgr, resource);
        }
    }

    pub fn get_component<T:Any>(& self) -> Option<& T>
    {
        for c in self.components.iter()
        {
            if let Some(s) = c.get_comp::<T>() {
                return Some(s);
            }

        }
        None
    }

    pub fn get_pos_quat(&self) -> (vec::Vec3, vec::Quat)
    {
        (self.position, self.orientation.as_quat())
    }

    pub fn make_transform(&self) -> transform::Transform
    {
        transform::Transform::from_position_orientation_scale(
            self.position,
            self.orientation,
            self.scale)
    }

    pub fn make_world_transform(&self) -> transform::Transform
    {
        transform::Transform::from_position_orientation_scale(
            self.world_position(),
            transform::Orientation::Quat(self.world_orientation()),
            self.world_scale())
    }

}

pub fn child_add(parent : Arc<RwLock<Object>>, child : Arc<RwLock<Object>>)
{
    parent.write().unwrap().children.push(child.clone());
    child.write().unwrap().parent = Some(parent.clone());
}

/*
impl Clone for Object
{
    fn clone(&self) -> Object
    {
        Object {
            name : self.name.clone(),
            id : uuid.clone(),
            mesh_render : None,
            position : vec::Vec3::zero(),
            orientation : vec::Quat::identity(),
            //angles : vec::Vec3::zero(),
            scale : vec::Vec3::one(),
            children : Vec::new(),
            parent : None
        }

    }
}
*/

pub struct ObjectRef
{
    pub id : uuid::Uuid,
    pub object : Option<Arc<RwLock<Object>>>,
}

impl ObjectRef
{
    pub fn new_with_id(id : uuid::Uuid) -> ObjectRef
    {
        ObjectRef {
            id : id,
            object : None
        }
    }

    pub fn new_with_object(o : Arc<RwLock<Object>>) -> ObjectRef
    {
        let id = o.read().unwrap().id.clone();
        ObjectRef {
            id : id,
            object : Some(o)
        }
    }
}

lua_extern! {
    unsafe fn print_ob(lua: &mut lua::ExternState) -> i32 {
        //let test = lua.checkudata(1, "object");
        //println!("mon test : {:?} ", test);

        let ptr = lua.touserdata(1);
        let obp : *mut Object = mem::transmute(ptr);
        let ob = &*obp;
        println!("ok this is my object : {} ,  {:?} ", ob.name, ob.position);
        0
    }

    unsafe fn get_pos(lua: &mut lua::ExternState) -> i32 {
        let ptr = lua.touserdata(1);
        let obp : *mut Object = mem::transmute(ptr);
        let ob = &*obp;
        lua.pushnumber(ob.position.x);
        lua.pushnumber(ob.position.y);
        lua.pushnumber(ob.position.z);
        3
    }

    unsafe fn object_string(lua: &mut lua::ExternState) -> i32 {
        //let ptr = lua.checkudata(1, "object");
        //*
        let ptr = lua.touserdata(1);
        let obp : *mut Object = mem::transmute(ptr);
        let ob = &*obp;
        lua.pushstring(&ob.name);
        //lua.pushnumber(ob.position.x);
        1
        //*/
        //0
    }

    unsafe fn tostring(lua: &mut lua::ExternState) -> i32 {
        //let ptr = lua.checkudata(1, "object");
        //*
        let ptr = lua.touserdata(1);
        let obp : *mut Object = mem::transmute(ptr);
        let ob = &*obp;
        lua.pushstring(&ob.name);
        //lua.pushnumber(ob.position.x);
        1
        //*/
        //0
    }

    unsafe fn object_index(lua: &mut lua::ExternState) -> i32 {
        let ptr = lua.touserdata(1);
        let obp : *mut Object = mem::transmute(ptr);
        let ob = &mut *obp;
        0
    }

    unsafe fn object_newindex(lua: &mut lua::ExternState) -> i32 {
        let ptr = lua.touserdata(1);
        let obp : *mut Object = mem::transmute(ptr);
        let ob = &mut *obp;
        match lua.checkstring(2) {
            Some(s) => {
                if s == "position" {
                    let ud = lua.checkudata(3, "vec3");
                    let v : *mut LuaData<vec::Vec3> = mem::transmute(ud);
                    ob.position = match *v {
                        LuaData::Pointer(p) => *p,
                        LuaData::Value(v) => v,
                    };
                }
                else {
                    let f = lua.checknumber(3);
                    match s {
                        "x" => ob.position.x = f,
                        "y" => ob.position.y = f,
                        "z" => ob.position.z = f,
                        _ => println!("not supported")
                    }
                }
            },
            None => println!("argument 2 is not a string")
        };


        0
    }

    unsafe fn index_handler(lua: &mut lua::ExternState) -> i32 {
        let ptr = lua.touserdata(1);
        let obp : *mut Object = mem::transmute(ptr);
        let ob = &mut *obp;
        match lua.checkstring(2) {
            Some(s) => {
                if s == "position" {
                    return push_pointer(lua, &mut ob.position, "vec3");
                }
                else {
                    let f = match s {
                        "x" => ob.position.x,
                        "y" => ob.position.y,
                        "z" => ob.position.z,
                    _ => 0f64
                    };
                    lua.pushnumber(f);
                    return 1;
                }
            },
            None => println!("ihihihihih argument 2 is not a string")
        };


        //put index on the top
        lua.pushvalue(2);
        // upvalueindex(2) gets the methods table
        lua.rawget(lua::upvalueindex(2));
        if lua.isnil(-1) {
            println!("cannot get member : {}", lua.tostring(2).unwrap());
            return 0;
        }

        1
    }

    unsafe fn vec3_index_handler(lua: &mut lua::ExternState) -> i32 {
        let ptr = lua.checkudata(1,"vec3");
        let ld : *mut LuaData<vec::Vec3> = mem::transmute(ptr);
        let v = match *ld {
            LuaData::Pointer(p) => &*p,
            LuaData::Value(ref v) => v
        };
        //let v = &*(*vp).pointer;
        match lua.checkstring(2) {
            Some(s) => {
                let f = match s {
                    "x" => v.x,
                    "y" => v.y,
                    "z" => v.z,
                    _ => 0f64
                };
                lua.pushnumber(f);
                return 1;
            },
            None => println!("vec3 argument 2 is not a string")
        };

        lua.pushvalue(2);
        // upvalueindex(2) gets the methods table 
        lua.rawget(lua::upvalueindex(2));
        if lua.isnil(-1) {
            println!("vec3 : cannot get member : {}", lua.tostring(2).unwrap());
            return 0;
        }

        1
    }

    unsafe fn vec3_newindex_handler(lua: &mut lua::ExternState) -> i32 {
        let ptr = lua.checkudata(1,"vec3");
        let ld : *mut LuaData<vec::Vec3> = mem::transmute(ptr);
        let v = match *ld {
            LuaData::Pointer(p) => &mut *p,
            LuaData::Value(ref mut  v) => v
        };
        //let v = &mut *(*vp).pointer;
        match lua.checkstring(2) {
            Some(s) => {
                let f = lua.checknumber(3);
                match s {
                    "x" => v.x = f,
                    "y" => v.y = f,
                    "z" => v.z = f,
                    _ => {}
                };
            },
            None => println!("vec3 argument 2 is not a string")
        };

        0
    }

    unsafe fn vec3_add(lua: &mut lua::ExternState) -> i32 {
        let ptr1 = lua.checkudata(1,"vec3");
        let ptr2 = lua.checkudata(2,"vec3");
        let ld1 : *mut LuaData<vec::Vec3> = mem::transmute(ptr1);
        let ld2 : *mut LuaData<vec::Vec3> = mem::transmute(ptr2);
        let v1 = match *ld1 {
            LuaData::Pointer(p) => *p,
            LuaData::Value(v) => v
        };

        let v2 = match *ld2 {
            LuaData::Pointer(p) => *p,
            LuaData::Value(v) => v
        };

        push_data(lua, v1 + v2, "vec3")
    }

    unsafe fn vec3_sub(lua: &mut lua::ExternState) -> i32 {
        let v1 : &vec::Vec3 = LuaData::from_lua_get_ref(lua, 1, "vec3");
        let v2 : &vec::Vec3 = LuaData::from_lua_get_ref(lua, 2, "vec3");
        let dot = v1.dot(v2);
        push_data(lua, *v1 - *v2, "vec3")
    }

    unsafe fn vec3_zero(lua: &mut lua::ExternState) -> i32 {
        push_data(lua, vec::Vec3::zero(), "vec3")
    }

    unsafe fn vec3_new(lua: &mut lua::ExternState) -> i32 {
        let x = lua.checknumber(1);
        let y = lua.checknumber(2);
        let z = lua.checknumber(3);
        push_data(lua, vec::Vec3::new(x,y,z), "vec3")
    }

    unsafe fn vec3_dot(lua: &mut lua::ExternState) -> i32 {
        let v1 : &vec::Vec3 = LuaData::from_lua_get_ref(lua, 1, "vec3");
        let v2 : &vec::Vec3 = LuaData::from_lua_get_ref(lua, 2, "vec3");
        let dot = v1.dot(v2);
        lua.pushnumber(dot);
        1
    }

}

enum LuaData<T>
{
    Pointer(*mut T),
    Value(T)
}

/*
struct Pointer<T>
{
    pointer : *mut T
}
*/

impl<T> LuaData<T> {
    fn new(p : *mut T) -> LuaData<T> {
        LuaData::Pointer(p)
    }

    fn from_pointer(ptr : *mut c_void) -> *mut LuaData<T> {
        let p : *mut LuaData<T> = unsafe { mem::transmute(ptr) };
        p
    }

    fn from_lua_mut<'a, 'b>(lua: &'a mut lua::ExternState, narg : i32, kind : &str) -> &'b mut LuaData<T> {
        let ptr = unsafe {lua.checkudata(narg, kind) };
        let ld : &mut LuaData<T> = unsafe { mem::transmute(ptr) };
        ld
    }

    fn from_lua<'a, 'b>(lua: &'a mut lua::ExternState, narg : i32, kind : &str) -> &'b LuaData<T> {
        let ptr = unsafe {lua.checkudata(narg, kind) };
        let ld : &LuaData<T> = unsafe { mem::transmute(ptr) };
        ld
    }

    fn from_lua_get_ref<'a, 'b>(lua: &'a mut lua::ExternState, narg : i32, kind : &str) -> &'b T {
        let ld = LuaData::from_lua(lua, narg, kind);
        let d = ld.get_ref();
        d
    }

    fn from_lua_get_mut<'a, 'b>(lua: &'a mut lua::ExternState, narg : i32, kind : &str) -> &'b mut T {
        let d1 = unsafe {lua.checkudata(narg, kind) };
        let v1r : &mut LuaData<T> = unsafe { mem::transmute(d1) };
        let v1 = v1r.get_mut();
        v1
    }

    fn get_value(&self) -> T where T: Copy
    {
        match *self {
            LuaData::Pointer(p) => unsafe { *p },
            LuaData::Value(v) => v
        }
    }

    fn get_ref(&self) -> &T
    {
        match *self {
            LuaData::Pointer(p) => unsafe { &*p },
            LuaData::Value(ref v) => v
        }
    }

    fn get_mut(& mut self) -> &mut T
    {
        match *self {
            LuaData::Pointer(p) => unsafe { &mut *p },
            LuaData::Value(ref mut v) => v
        }
    }

    fn set_value(&mut self, value: T)
    {
        match *self {
            LuaData::Pointer(p) => unsafe {*p = value},
            LuaData::Value(ref mut v) => *v = value
        }
    }
}

fn set_pointer<T>(p : *mut c_void, data : *mut T)
{
    let ld : *mut LuaData<T> = unsafe { mem::transmute(p) };
    unsafe {
        *ld = LuaData::Pointer(data);
    }
}

fn set_data<T>(p : *mut c_void, data : T)
{
    let ld : *mut LuaData<T> = unsafe { mem::transmute(p) };
    unsafe {
        *ld = LuaData::Value(data);
    }
}


fn create_vec3_metatable(lua : &mut lua::State)
{

    let fns = &[
        ("new", vec3_new as lua::CFunction),
        ("zero", vec3_zero as lua::CFunction),
        ("dot", vec3_dot as lua::CFunction),
    ];

    lua.registerlib(Some("vec3"), fns);
    let methods = lua.gettop();


    if lua.newmetatable("vec3") {
        //lua.registerlib(None, meta);
    }
    else {
        panic!("table vec3 already exists");
    }

    let metatable = lua.gettop();

    //hide metatable
    {
        lua.pushstring("__metatable");
        lua.pushvalue(methods);
        lua.rawset(metatable);
    }

    {
        lua.pushstring("__index");
        lua.pushvalue(metatable);
        lua.pushvalue(methods);
        lua.pushcclosure(vec3_index_handler,2);
        lua.rawset(metatable);
    }

    {
        lua.pushstring("__newindex");
        lua.pushvalue(metatable);
        lua.pushcclosure(vec3_newindex_handler,1);
        lua.rawset(metatable);
    }

    {
        lua.pushstring("__add");
        lua.pushcclosure(vec3_add,0);
        lua.rawset(metatable);

        lua.pushstring("__sub");
        lua.pushcclosure(vec3_sub,0);
        lua.rawset(metatable);
    }

    //TODO
    /*
     *
    __unm - Unary minus. When writing "-myTable", if the metatable has a __unm key pointing to a function, that function is invoked (passing the table), and the return value used as the value of "-myTable".
    __mul - Multiplication. Similar to addition, using the '*' operator.
    __div - Division. Similar to addition, using the '/' operator.
    __mod - Modulo. Similar to addition, using the '%' operator.
    __pow - Involution. Similar to addition, using the '^' operator.
    __concat - Concatenation. Similar to addition, using the '..' operator. 
     */


    lua.pop(1);

}

fn push_pointer<T>(lua: &mut lua::ExternState, v : &mut T, table : &str) -> i32 {

    let data = 
        unsafe {
            lua.newuserdata(mem::size_of::<LuaData<T>>())
        };

    set_pointer(data, v);

    unsafe {
        lua.getmetatable_reg(table);
        lua.setmetatable(-2);
    }

    1
}

fn push_data<T>(lua: &mut lua::ExternState, v : T, table : &str) -> i32 {

    let data = 
        unsafe {
            lua.newuserdata(mem::size_of::<LuaData<T>>())
        };

    set_data(data, v);

    unsafe {
        lua.getmetatable_reg(table);
        lua.setmetatable(-2);
    }

    1
}


fn debug_lua(lua : &mut lua::State)
{
    let top = lua.gettop();
    for i in 1..top+1 { 
        let t = lua.type_(i);
        match t {
            _ => {println!("{}, t : {:?}", i, t) }
    
        }
        println!("  ");
    }
    println!("");
}

use intersection::MeshTransform;
//TODO remove 'a from Object by using get_ref_no_instance instead of get_ref
pub fn object_to_mt<'a>(o : &'a Object, mm : &'a mut resource::ResourceManager<mesh::Mesh>) -> Option<MeshTransform<'a>>
{
    match o.mesh_render {
        None => None,
        Some(ref mr) => {
            let wp = o.world_position();
            let wq = o.world_orientation();
            let ws = o.world_scale();

            let m = &mr.mesh;
            if let Some(mesh) = m.get_ref(mm) {
                let mt = MeshTransform {
                    mesh : mesh,
                    position : wp,
                    orientation : wq,
                    scale : ws
                };
                Some(mt)
            }
            else {
                None
            }
        }
    }
}

use geometry;
use intersection;
pub fn is_object_in_planes(
    planes : &[geometry::Plane],
    o : &Object,
    resource : &resource::ResourceGroup
    ) 
    -> bool
{
    match o.mesh_render {
        None => 
            intersection::is_position_in_planes(planes, o.position),
        Some(ref mr) => {
            match object_to_mt(o, &mut *resource.mesh_manager.borrow_mut()) {
                Some(ref mt) =>
                intersection::is_mesh_transform_in_planes(planes, mt),
                None => intersection::is_position_in_planes(planes, o.position)
            }
        }
    }
}

