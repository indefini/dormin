use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead, Read};
use libc::{c_char, c_uint, c_void};
use std::ptr;
use std::str::FromStr;
use std::ffi::CString;
use std::path::Path;
use std::mem;
use std::fmt;
//use std::default::Default;
//use toml;

use util;
use vec;
use resource;
use uniform::UniformSend;
use uniform::TextureSend;
use texture;

#[repr(C)]
pub struct CglShader;
#[repr(C)]
pub struct CglShaderAttribute;
#[repr(C)]
pub struct CglShaderUniform;

#[derive(Serialize, Deserialize)]
pub struct Shader
{
    pub name : String,
    #[serde(skip_serializing, skip_deserializing)]
    pub attributes : HashMap<String, *const CglShaderAttribute>,
    #[serde(skip_serializing, skip_deserializing)]
    pub uniforms : HashMap<String, *const CglShaderUniform>,
    #[serde(skip_serializing, skip_deserializing)]
    pub state : i32,

    pub vert_path : Option<String>,
    pub frag_path : Option<String>,

    pub vert : Option<String>,
    pub frag : Option<String>,

    #[serde(skip_serializing, skip_deserializing)]
    cgl_shader : Option<*const CglShader>, 
}

unsafe impl Send for Shader {}
unsafe impl Sync for Shader {}

extern fn shader_uniform_add(
    data : *const c_void,
    name : *const c_char,
    cgl_uni : *const CglShaderUniform)
{
    let uniforms : &mut HashMap<String, *const CglShaderUniform> = unsafe {mem::transmute(data) };
    uniforms.insert(util::c_char_to_string(name), cgl_uni);
}

extern fn shader_attribute_add(
    data : *const c_void,
    name : *const c_char,
    cgl_att : *const CglShaderAttribute)
{
    let attributes : &mut HashMap<String, *const CglShaderAttribute> = unsafe {mem::transmute(data) };
    attributes.insert(util::c_char_to_string(name), cgl_att);
}


impl Shader
{
    /*
    fn attribute_add(&mut self, name : &str, size : u32)
    {
        let attc = CString::new(name.as_bytes()).unwrap();

        match self.cgl_shader {
            None => {},
            Some(cs) =>
                unsafe {
                    let cgl_att = cgl_shader_attribute_new(cs, attc.as_ptr(), size);
                    if cgl_att != ptr::null() {
                        self.attributes.insert(String::from(name), cgl_att);
                    }
                }
        }

    }

    fn uniform_add(&mut self, name : &str)
    {
        let unic = CString::new(name.as_bytes()).unwrap();

        match self.cgl_shader {
            None => {},
            Some(cs) =>
                unsafe {
                    let cgl_uni = cgl_shader_uniform_new(cs, unic.as_ptr());
                    if cgl_uni != ptr::null() {
                        self.uniforms.insert(String::from(name), cgl_uni);
                    }
                }
        }
    }
    */

    pub fn uniform_set(&self, name : &str, value : &UniformSend)
    {
        match self.uniforms.get(&String::from(name)) {
            Some(uni) => value.uniform_send(*uni),
            None => {
                println!("ERR!!!! : could not find such uniform '{}'",name)
            }
        }
    }

    pub fn texture_set(&self, name : &str, value : &TextureSend, index : u32)
    {
        match self.uniforms.get(&String::from(name)) {
            Some(uni) => value.uniform_send(*uni, index),
            None => {println!("ERR!!!! : could not find such uniform '{}'",name)}
        }
    }

    pub fn utilise(&self)
    {
        match self.cgl_shader {
            None => {},
            Some(cs) =>
                unsafe {
                    cgl_shader_use(cs);
                }
        }
    }

    pub fn new(name : &str) -> Shader
    {
        Shader {
            name : String::from(name),
            cgl_shader : None,
            attributes : HashMap::new(),
            uniforms : HashMap::new(),
            vert_path : None,
            frag_path : None,
            vert : None,
            frag : None,
            state : 0
        }
    }

    pub fn read(&mut self)
    {
        let mut file = {
            let path = Path::new(&self.name);
            BufReader::new(File::open(&path).ok().unwrap())
        };

        let mut frag = String::new();
        let mut vert = String::new();

        match file.read_line(&mut vert) {
            Ok(_) => { vert.pop(); },
            Err(_) => return
        }
 
        match file.read_line(&mut frag) {
            Ok(_) => { frag.pop();},
            Err(_) => return
        }

        self.read_vert_frag(vert.as_ref(), frag.as_ref());
        self.vert_path = Some(vert);
        self.frag_path = Some(frag);

        //TODO remove from here
        self.cgl_init();

        unsafe { cgl_shader_attributes_init(
                self.cgl_shader.unwrap(),
                shader_attribute_add,
                mem::transmute(&mut self.attributes)); }

        unsafe { cgl_shader_uniforms_init(
                self.cgl_shader.unwrap(),
                shader_uniform_add,
                mem::transmute(&mut self.uniforms)); }


        self.state = 2;
    }

    fn read_vert_frag(&mut self, vertpath : &str, fragpath : &str)
    {
        if self.state > 1 {
            return
        }

        {
            let mut contents = String::new();
            match File::open(&Path::new(fragpath)).ok().unwrap().read_to_string(&mut contents){
                Ok(_) => self.frag = Some(contents),
                _ => return
            }
        }

        {
            let mut contents = String::new();
            match File::open(&Path::new(vertpath)).ok().unwrap().read_to_string(&mut contents) {
                Ok(_) => self.vert = Some(contents),
                _ => return
            }
        }

        self.state = 1;
    }

    pub fn set_vert_frag(&mut self, vert : String, frag : String)
    {
        self.vert = Some(vert);
        self.frag = Some(frag);
        self.state = 1;
    }

    pub fn cgl_init(&mut self)
    {
        let vertc;
        match self.vert {
            None => return,
            Some(ref v) => {
                vertc = CString::new(v.as_bytes()).unwrap();
            }
        }

        let fragc;
        match self.frag {
            None => return,
            Some(ref f) => {
                fragc = CString::new(f.as_bytes()).unwrap();
            }
        }

        let vertcp = vertc.as_ptr();
        let fragcp = fragc.as_ptr();

        println!("shader::: begin to init : {}", self.name);

        unsafe {
            let shader = cgl_shader_init_string(vertcp, fragcp);
            self.cgl_shader = Some(shader);
        }

        self.state = 3;
    }

    /// returns true if it actually reloads
    pub fn reload(&mut self) -> bool
    {
        println!("RELOAD");
        println!("TODO free resource of old shader");

        let vert = if let Some(ref vert) = self.vert_path {
            vert.clone()
        }
        else {
            println!("reload early return");
            return false;
        };

        let frag = if let Some(ref frag) = self.frag_path {
            frag.clone()
        }
        else {
            println!("reload early return2");
            return false;
        };
        
        
        self.read_vert_frag(&vert, &frag);
        self.state = 1;
        
        true
    }

    pub fn load_gl(&mut self)
    {
        self.cgl_init();

        unsafe { cgl_shader_attributes_init(
                self.cgl_shader.unwrap(),
                shader_attribute_add,
                mem::transmute(&mut self.attributes)); }

        unsafe { cgl_shader_uniforms_init(
                self.cgl_shader.unwrap(),
                shader_uniform_add,
                mem::transmute(&mut self.uniforms)); }
    }
}


#[derive(Clone,Serialize, Deserialize, Debug)]
pub enum UniformData
{
    Int(i32),
    Float(f32),
    Vec2(vec::Vec2),
    Vec3(vec::Vec3),
    Vec4(vec::Vec4),
}

macro_rules! unimatch(
    ($inp:expr, $uni:expr, [ $($sp:ident)|+ ]) => (
        match $inp {
            $(
                UniformData::$sp(ref x) => { x.uniform_send($uni); }
             )+
            //_ => {}
        }
    );
);

impl UniformSend for UniformData
{
    fn uniform_send(&self, uni : *const CglShaderUniform) ->()
    {
        unimatch!(*self, uni, [Int|Float|Vec2|Vec3|Vec4]);
    }
}

type ShaderUniformAddFn = extern fn(
    data : *const c_void,
    name : *const c_char,
    cgl_uni : *const CglShaderUniform);

type ShaderAttributeAddFn = extern fn(
    data : *const c_void,
    name : *const c_char,
    cgl_att : *const CglShaderAttribute);

#[link(name = "cypher")]
extern {
    fn cgl_shader_init_string(
        vert : *const c_char,
        frat : *const c_char) -> *const CglShader;

    pub fn cgl_shader_use(shader : *const CglShader);

    /*
    pub fn cgl_shader_attribute_new(
        shader : *const CglShader,
        name : *const c_char,
        size : c_uint) -> *const CglShaderAttribute;

    pub fn cgl_shader_uniform_new(
        shader : *const CglShader,
        name : *const c_char) -> *const CglShaderUniform;
        */

    fn cgl_shader_attributes_init(
        shader : *const CglShader, 
        cb : ShaderAttributeAddFn,
        data : *const c_void);

    fn cgl_shader_uniforms_init(
        shader : *const CglShader, 
        cb : ShaderUniformAddFn,
        data : *const c_void);

}

impl fmt::Debug for Shader
{
    fn fmt(&self, fmt : &mut fmt::Formatter) -> fmt::Result
    {
        write!(fmt, "I am a shader:{}", self.name)
    }
}

