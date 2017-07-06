#![feature(box_syntax)]
#![feature(step_by)]
//#![feature(zero_one)]
#![feature(associated_consts)]

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


#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}

#[macro_use]
pub mod property;

pub mod input;

pub mod resource;
pub mod shader;
pub mod material;
pub mod armature;
pub mod mesh;
//mod mesh_render;
pub mod render;
pub mod object;
pub mod uniform;
pub mod matrix;
pub mod vec;
pub mod camera;
pub mod camera2;
pub mod scene;
pub mod texture;
pub mod geometry;
pub mod intersection;
pub mod fbo;
pub mod mesh_render;

pub mod transform;

pub mod component;
//pub use component::manager;
pub mod armature_animation;


mod util;

#[link(name = "GLESv2", kind="dylib")]
#[link(name = "cypher")]
extern
{
    pub fn cgl_clear() -> ();
}

