use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::path::Path;
use serde_json;
//use std::default::Default;
//use toml;


use vec;
use shader;
//use matrix;
use resource;
//use uniform;
//use uniform::UniformSend;
use uniform::TextureSend;
use texture;
use fbo;
use self::Sampler::{ImageFile,Fbo};

#[derive(Serialize, Deserialize, Clone)]
pub enum Sampler
{
    ImageFile(resource::ResTT<texture::Texture>),
    Fbo(resource::ResTT<fbo::Fbo>, fbo::Attachment)
}

impl Sampler
{
    pub fn name(&self) -> &str
    {
        match *self {
            ImageFile(ref img) => {
                img.name.as_ref()
            },
            Fbo(ref f, _) => {
                f.name.as_ref()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Material
{
    pub name : String,
    pub shader: Option<resource::ResTT<shader::Shader>>,
    #[serde(skip_serializing, skip_deserializing)]
    pub state : i32,
    pub textures : HashMap<String, Sampler>,
    pub uniforms : HashMap<String, Box<shader::UniformData>>,
}

unsafe impl Send for Material {}
unsafe impl Sync for Material {}

/*
impl Default for Material
{
    fn default() -> Material {
          Material {
            name : String::from("default"),
            shader : None,
            state : 0,
            textures : HashMap::new()
            uniforms : HashMap::new()
        }
    }
}
*/

impl Material
{
    pub fn new(name : &str) -> Material
    {
        Material {
            name : String::from(name),
            shader : None,
            state : 0,
            textures : HashMap::new(),
            uniforms : HashMap::new(),
        }
    }

    pub fn new_from_file(file_path : &str) -> Material
    {
        let mut file = String::new();
        File::open(&Path::new(file_path)).ok().unwrap().read_to_string(&mut file);
        let mat : Material = serde_json::from_str(&file).unwrap();
        mat
    }

    pub fn read(&mut self)
    {
        let file = {
            let path : &Path = self.name.as_ref();
            match File::open(path){
                Ok(mut f) => {
                    let mut file = String::new();
                    f.read_to_string(&mut file);
                    file
                },
                Err(e) => {
                    println!("Error reading file '{}. Error : {}", self.name, e);
                    return;
                }
            }
        };

        let mat : Material = match serde_json::from_str(&file){
            Ok(m) => m,
            Err(e) => { 
                println!("{}, line {}: error reading material '{}': {:?}, creating new material",
                         file!(),
                         line!(),
                         self.name,
                         e); 
                Material::new(self.name.as_ref())
            }
        };

        self.name = mat.name.clone();
        match mat.shader {
            Some(s) => 
                self.shader = Some(resource::ResTT::new(s.name.as_ref())),
            None => self.shader = None
        }

        for (k,v) in mat.textures.iter()
        {
            match *v {
                ImageFile(ref img) => {
                    self.textures.insert(k.clone(), ImageFile(resource::ResTT::new(img.name.as_ref())));
                },
                Fbo(ref f, ref a) => {
                    self.textures.insert(k.clone(), Fbo(resource::ResTT::new(f.name.as_ref()), *a));
                }
            }
        }

        self.uniforms = mat.uniforms.clone();
    }

    pub fn save(&self)
    {
        let path : &Path = self.name.as_ref();
        let mut file = File::create(path).ok().unwrap();

        let s = serde_json::to_string_pretty(self).unwrap();
        let result = file.write(s.as_bytes());
    }

    pub fn set_uniform_data(&mut self, name : &str, data : shader::UniformData)
    {
        let key = name.to_string();
        let yep = match self.uniforms.entry(key){
            Vacant(entry) => entry.insert(box data),
            Occupied(entry) => {
                let entry = entry.into_mut();
                *entry = box data;
                entry
            }
        };
    }

}

impl resource::ResourceT for Material
{
    fn init(&mut self)
    {
        match self.shader {
            //TODO now
            _ => {},
            /*
            None => return,
            Some(ref mut s) => {
                s.read();
                //TODO remove
                s.utilise();
                s.uniform_set("color", &vec::Vec4::new(0.0f64, 0.5f64, 0.5f64, 1f64));
            }
            */
        }

    }

}

