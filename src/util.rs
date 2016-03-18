use std::sync::{RwLock, Arc};
use std::fs;
use std::mem;

use vec;
use object;

pub fn objects_center(objects : &[Arc<RwLock<object::Object>>]) -> vec::Vec3
{
    let mut v = vec::Vec3::zero();
    for o in objects.iter()
    {
        v = v + o.read().unwrap().position;
    }

    v = v / objects.len() as f64;

    v
}

use std::path::{Path, PathBuf};
pub fn get_files_in_dir(path : &str) -> Vec<PathBuf>
{
    let files = fs::read_dir(path).unwrap();
    /*
    for file in files {
        println!("Name: {}", file.unwrap().path().display())
    }
    */

    files.map(|x| x.unwrap().path()).collect()
}

use std::ffi::{CString, CStr};
use std::str;
use libc::{c_void, c_int, size_t, c_char};

pub fn to_cstring(v : Vec<PathBuf>) -> Vec<CString>
{
    v.iter().map(|x| CString::new(x.to_str().unwrap()).unwrap()).collect()
}

pub fn string_to_cstring(v : Vec<String>) -> Vec<CString>
{
    v.iter().map(|x| CString::new(x.as_str()).unwrap()).collect()
}

pub fn print_vec_cstring(v : Vec<CString>)
{
    let y : Vec<*const c_char> = v.iter().map( |x| x.as_ptr()).collect();
}

pub fn pass_slice() 
{
    let s = [ 
        CString::new("test").unwrap().as_ptr(),
        CString::new("caca").unwrap().as_ptr(),
        CString::new("bouda").unwrap().as_ptr() ];
}

pub fn c_char_to_string(c : *const c_char) -> String
{
    String::from( unsafe { CStr::from_ptr(c).to_str().unwrap() })
}

//WIP
struct Frame {
    num : i32,
    dt : f32
}

struct FrameEvents
{
    events : Vec<i32>
}



