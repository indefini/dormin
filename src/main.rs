//#![feature(log_syntax)]
//#![feature(trace_macros)]
//#![feature(slicing_syntax)]
#![feature(box_syntax)]
#![feature(step_by)]
#![feature(zero_one)]

//TODO remove
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(improper_ctypes)]
//#![allow(ctypes)]

#![feature(plugin)]
//#![plugin(clippy)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate rustc_serialize;

extern crate byteorder;
extern crate libc;
//extern crate sync;
extern crate png;
extern crate toml;
//extern crate debug;
extern crate uuid;
extern crate core;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate lua;

use std::collections::HashMap;
use std::sync::{RwLock, Arc};
//use std::rc::Rc;
//use std::cell::RefCell;
use std::mem;
//use std::any::{Any, AnyRefExt};

use std::io::{self, Write};
use std::path::Path;
use std::process;

#[macro_use]
mod property;

mod resource;
mod shader;
mod material;
mod armature;
mod mesh;
//mod mesh_render;
mod render;
mod object;
mod uniform;
mod matrix;
mod vec;
mod camera;
mod camera2;
mod scene;
mod texture;
mod geometry;
mod intersection;
mod fbo;
mod factory;

mod transform;

mod model;

mod component;
use component::manager;

mod util;
mod input;


static mut S_TEST : i32 = 5;

fn main() {
    let files = util::get_files_in_dir("scene");
    let cs = util::to_cstring(files);
    util::print_vec_cstring(cs);
    util::pass_slice();
    unsafe {
    S_TEST = 4432;
    }

    {
     println!("The map has {} entries.", *component::manager::COUNT);
    }

    {
    let mut hash = &mut component::manager::HASHMAP.lock().unwrap();
    println!("going to insert 5");
    hash.insert(5, "cinq");
    println!("The entry for `1` is \"{}\".", hash.get(&1).unwrap());
    println!("The entry for `0` is \"{}\".", hash.get(&0).unwrap());
    println!("The entry for `5` is \"{}\".", hash.get(&5).unwrap());
    }

    {
     println!("The map has {} entries.", *component::manager::COUNT);
    }

    {
        //let mut cm = component::Manager::new();
        let mut cm = component::manager::COMP_MGR.lock().unwrap();
        cm.register_component("player_behavior", component::player::player_new);
        cm.register_component(
            "armature_animation",
            component::armature_animation::new);
    }

    {

    let mut lua = lua::State::new();
    lua.openlibs();
    /*
    match lua.loadfile(None) {
        Ok(()) => (),
        Err(lua::LoadFileError::ErrSyntax) => panic!("syntax error"),
        Err(lua::LoadFileError::ErrMem) => panic!("memory allocation error"),
        Err(lua::LoadFileError::ErrFile) => panic!("file error (?!?)")
    }
    lua.call(0, 0);
    */

    // Load the file containing the script we are going to run
    let path = Path::new("simpleapi.lua");
    match lua.loadfile(Some(&path)) {
        Ok(_) => (),
        Err(_) => {
            // If something went wrong, error message is at the top of the stack
            let _ = writeln!(&mut io::stderr(),
            "Couldn't load file: {}", lua.describe(-1));
            process::exit(1);
        }
    }

    /*
     * Ok, now here we go: We pass data to the lua script on the stack.
     * That is, we first have to prepare Lua's virtual stack the way we
     * want the script to receive it, then ask Lua to run it.
     */
    lua.newtable(); // We will pass a table

    for i in 1..6 {
        lua.pushinteger(i);   // Push the table index
        lua.pushinteger(i*2); // Push the cell value
        lua.rawset(-3);       // Stores the pair in the table
    }

    // By what name is the script going to reference our table?
    lua.setglobal("foo");

    // Ask Lua to run our little script
    match lua.pcall(0, lua::MULTRET, 0) {
        Ok(()) => (),
        Err(_) => {
            let _ = writeln!(&mut io::stderr(),
                             "Failed to run script: {}", lua.describe(-1));
            process::exit(1);
        }
    }

    // Get the returned value at the to of the stack (index -1)
    let sum = lua.tonumber(-1);

    println!("Script returned: {}", sum);

    lua.pop(1); // Take the returned value out of the stack

    }


}


