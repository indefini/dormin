use std::rc::Rc;
use std::cell::RefCell;
use libc::{c_uint, c_int};
use std::sync;
use std::sync::{RwLock, Arc, RwLockReadGuard, Mutex};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use uuid;

use resource;
use resource::ResTT;
use shader;
use material;
use mesh;
use camera;
use matrix;
use texture;
use mesh_render;
use fbo;
use vec;
use transform;
use uniform;

use armature;
use armature_animation;

use geometry;

use mesh::BufferSend;

#[link(name = "cypher")]
extern {
    pub fn cgl_draw(vertex_count : c_uint) -> ();
    pub fn cgl_draw_lines(vertex_count : c_uint) -> ();
    pub fn cgl_draw_faces(buffer : *const mesh::CglBuffer, index_count : c_uint) -> ();
    pub fn cgl_draw_end() -> ();

    pub fn cgl_clear() -> ();

    pub fn cypher_init_simple();
    pub fn cypher_draw_start(w : i32, h : i32);
    pub fn cypher_draw_end();
}

/*
pub extern fn init_cb(r : *mut Render) -> () {
    unsafe {
        return (*r).init();
    }
}

pub extern fn draw_cb(r : *mut Render) -> () {
    unsafe {
        return (*r).draw();
    }
}

pub extern fn resize_cb(r : *mut Render, w : c_int, h : c_int) -> () {
    unsafe {
        return (*r).resize(w, h);
    }
}
*/
#[derive(Clone, Debug)]
pub struct MatrixMeshRender
{
    pub mat : matrix::Matrix4,
    pub mr : mesh_render::MeshRender
}

impl MatrixMeshRender {
    pub fn new(mat : matrix::Matrix4, mr : mesh_render::MeshRender) -> MatrixMeshRender
    {
        MatrixMeshRender {
        mat : mat,
        mr : mr
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransformMeshRender
{
    pub transform : transform::Transform,
    pub mr : mesh_render::MeshRender
}

impl TransformMeshRender {
    pub fn new(t : transform::Transform, mr : mesh_render::MeshRender) -> TransformMeshRender
    {
        TransformMeshRender {
            transform : t,
            mr : mr
        }
    }

    pub fn with_mesh(mr : mesh_render::MeshRender) -> TransformMeshRender
    {
        TransformMeshRender {
            transform : Default::default(),
            mr : mr
        }
    }

    pub fn to_mmr(&self) -> MatrixMeshRender
    {
        MatrixMeshRender::new(self.transform.compute_return_local_matrix(), self.mr.clone())
    }

    pub fn set_uniform_data(
        &mut self,
        name : &str,
        data : shader::UniformData)
    {
        println!("set uniform data : {}", name);
        self.mr.material.get_instance().unwrap().set_uniform_data(name, data);
    }
}


pub struct CameraPass
{
    matrix : matrix::Matrix4,
    mmr : Vec<MatrixMeshRender>,
}

impl CameraPass
{
    fn new(mat : matrix::Matrix4) -> CameraPass
    {
        CameraPass {
            matrix : mat,
            mmr : Vec::new()
        }
    }

    pub fn add_mmr(&mut self, mr : MatrixMeshRender)
    {
        self.mmr.push(mr);
    }
}

pub struct RenderPass<Id:Hash+Clone+Eq>
{
    pub name : String,
    //pub shader : Arc<RwLock<shader::Shader>>,
    pub shader : ResTT<shader::Shader>,
    //uuid is the camera id
    pub passes : HashMap<Id, Box<CameraPass>>,
}

impl<Id:Hash+Eq+Clone> RenderPass<Id>
{
    pub fn new(
        shader : ResTT<shader::Shader>
        )
        -> RenderPass<Id>
    {
        RenderPass {
                  name : String::from("passtest"),
                  shader : shader,//.clone(),
                  passes : HashMap::new()
              }
    }

    pub fn draw_frame(
        &self,
        resource : &resource::ResourceGroup,
        load : Arc<Mutex<usize>>
        ) -> usize
    {
        //let shader = &mut *self.shader.write().unwrap();
        let shader_manager = &mut *resource.shader_manager.borrow_mut();
        let shader = self.shader.get_from_manager_instant(shader_manager);

        if shader.state == 0 {
            shader.read();
        }
        else if shader.state == 1 {
            shader.load_gl();
        }

        shader.utilise();

        let mut not_loaded = 0;

        for (_,p) in self.passes.iter() {

            for m in p.mmr.iter() {
                let not = self.draw_mmr(
                    shader,
                    &m.mat,
                    &m.mr,
                    &p.matrix,
                    resource,
                    load.clone()
                    );

                not_loaded += not;
            }
        }

        not_loaded
    }

    fn draw_armature(
        &self,
        armature : &armature::ArmatureInstance,
        matrix : &matrix::Matrix4)
    {
        /*
        let color = vec::Vec4::new(1f64,1f64,1f64,1.0f64);

        for b in armature.bones.iter() {
            let p1 = armature.position + b.position;
            let p2 = p1 + b.tail;
            let s = geometry::Segment::new(p1,p2);
            self.line.add_line(s, color);
        }
        */
    }

    fn draw_mmr(
        &self,
        shader : &shader::Shader,
        world_matrix : &matrix::Matrix4,
        mesh_render : &mesh_render::MeshRender,
        matrix : &matrix::Matrix4,
        resource : &resource::ResourceGroup,
        load : Arc<Mutex<usize>>
        ) -> usize
    {
        let mut not_loaded = 0;

        let init_material = |mr : &mesh_render::MeshRender| -> usize
        {
            let material_manager = &mut *resource.material_manager.borrow_mut();
            let m = mr.material.get_ref(material_manager).unwrap();

            object_init_mat(m, shader, resource, load)
        };

        not_loaded = init_material(mesh_render);
        if not_loaded > 0 { println!("not loaded {}, init material", not_loaded); }

        let init_mesh_render = |mr : &mesh_render::MeshRender|  -> ((bool, usize), bool)
        {
            let mesh_manager = &mut *resource.mesh_manager.borrow_mut();

            if let Some(ref m) = mr.mesh.instance {
               (init_mesh(m, shader), true)
            }
            else if let Some(m) = resource::resource_get_ref(mesh_manager, &mr.mesh) {
                (init_mesh(m, shader), false)
            }
            else {
                ((false, 0usize), false)
            }
        };

        let ((can_render, vertex_data_count), instance) = init_mesh_render(mesh_render);

        if can_render {
            let object_mat_world = matrix * world_matrix ;
            shader.uniform_set("matrix", &object_mat_world);

            let draw_mesh = |mr : &mesh_render::MeshRender|
            {
                let mesh_manager = &mut *resource.mesh_manager.borrow_mut();
                let m = mr.mesh.get_ref(mesh_manager).unwrap();
                object_draw_mesh(m, vertex_data_count);
            };

            draw_mesh(mesh_render);
        }
        else if instance {
            println!("TODO instance");
        }
        else {
            not_loaded += 1;
            if not_loaded > 0 { println!("not loaded {}, cannot render : {:?}", not_loaded, mesh_render); }
        }

        not_loaded
    }
}

pub fn get_pass_from_mesh_render<'a, Id:Hash+Eq+Clone>(
    mr : &mesh_render::MeshRender,
    passes : &'a mut HashMap<String, Box<RenderPass<Id>>>, 
    material_manager : &mut resource::ResourceManager<material::Material>,
    shader_manager : &mut resource::ResourceManager<shader::Shader>,
    camera : &CameraIdMat<Id>,
    load : Arc<Mutex<usize>>
    ) -> Option<&'a mut CameraPass>
{
        let mat = &mr.material;

        let matname = mat.name.clone();
        let mmm = if let Some(m) = resource::resource_get_ref(material_manager, mat) {
            &m.shader
        }
        else {
            return None
        };

        let shader_yep = match *mmm {
            Some(ref s) => s,
            None =>  {
                println!("material problem, return, {}", matname);
                return None
            }
        };

        let shader_copy = shader_yep.clone();
        let shadername = shader_yep.name.clone();
        let shader = match shader_yep.get_or_load_ref(shader_manager) {
        //let shader = match shader_yep.get_no_load(shader_manager) {
            Some(s) => s,
            None => {
                println!("shader problem, return, {}", shadername);
                return None
            }
        };

        {
            let key = shader.name.clone();
            let rp = match passes.entry(key) {
                Vacant(entry) => 
                    entry.insert(box RenderPass::new(shader_copy)),
                Occupied(entry) => entry.into_mut(),
            };

            let key_cam = camera.id.clone();
            let cam_pass = match rp.passes.entry(key_cam) {
                Vacant(entry) => {
                    entry.insert(box CameraPass::new(camera.matrix))
                },
                Occupied(entry) => entry.into_mut(),
            };
            
            Some(cam_pass)
        }

}

fn send_shader_sampler(
    shader : &shader::Shader,
    sampler : &HashMap<String, material::Sampler>,
    texture_manager : &mut resource::ResourceManager<texture::Texture>,
    fbo_manager : &mut resource::ResourceManager<fbo::Fbo>,
    ) -> usize
{
    let mut not_loaded = 0;

    let mut i = 0u32;
    for (name,t) in sampler.iter() {
        match *t {
            material::Sampler::ImageFile(ref img) => {
                let r = resource::resource_get_ref(texture_manager, img);
                match r {
                    Some(tex) => {
                        tex.init();
                        //TODO must release but this is mut,  so release somewhere else
                        //tex.release();
                        shader.texture_set(name.as_ref(), & *tex, i);
                        i = i +1;
                    },
                    None => {
                        not_loaded = not_loaded + 1;
                        println!("-------------------------there is NONONO tex........ {}", name);
                    }
                }
            },
            material::Sampler::Fbo(ref fbo, ref attachment) => {
                let yoyo = fbo_manager.get_or_create(fbo.name.as_str());
                let fbosamp = uniform::FboSampler { 
                    fbo : yoyo,
                    attachment : *attachment
                };
                shader.texture_set(name.as_ref(), &fbosamp,i);
                i = i +1;
            },
        }
    }

    not_loaded
}

fn send_shader_uniforms(
    shader : &shader::Shader,
    uniforms : &HashMap<String, Box<shader::UniformData>>
    )
{
    for (k,v) in uniforms.iter() {
        shader.uniform_set(k.as_ref(), &(**v));
    }
}


fn object_init_mat(
        material : &material::Material,
        shader : &shader::Shader,
        resource : &resource::ResourceGroup,
        load : Arc<Mutex<usize>>
    ) -> usize
{
    let mut not_loaded = 0;

    let mut i = 0u32;
    for (name,t) in material.textures.iter() {
        match *t {
            material::Sampler::ImageFile(ref img) => {
                let texture_manager = &mut *resource.texture_manager.borrow_mut();
                let r = resource::resource_get_mut_no_instance(texture_manager, img, load.clone());
                match r {
                    Some(tex) => {
                        tex.init();
                        tex.release();
                        shader.texture_set(name.as_ref(), & *tex, i);
                        i = i +1;
                    },
                    None => {
                        not_loaded = not_loaded +1;
                        //println!("there is NONONO tex........ {}", name);
                    }
                }
            },
            material::Sampler::Fbo(ref fbo, ref attachment) => {
                let mut rm = resource.fbo_manager.borrow_mut();
                let yoyo = rm.get_or_create(fbo.name.as_str());
                let fbosamp = uniform::FboSampler { 
                    fbo : yoyo,
                    attachment : *attachment
                };
                shader.texture_set(name.as_ref(), &fbosamp,i);
                i = i +1;
            },
        }
    }

    for (k,v) in material.uniforms.iter() {
        shader.uniform_set(k.as_ref(), &(**v));
    }

    not_loaded
}

fn init_mesh(
    mb : &mesh::Mesh,
    shader : &shader::Shader) -> (bool, usize)
{
    mb.init_buffers();

    let mut can_render = true;
    let mut vertex_data_count = 0;
    for (name, cgl_att) in shader.attributes.iter() {

        match mb.buffer_f32_get(name.as_ref()){
            Some(ref cb) => {
                cb.utilise(*cgl_att);
                if name == "position" {
                    vertex_data_count = cb.size_get();
                }
                continue;
            },
            None => (),
        }

        match mb.buffer_u32_get(name.as_ref()){
            Some(ref cb) => {
                cb.utilise(*cgl_att);
                if name == "position" {
                    vertex_data_count = cb.size_get();
                }
                continue;
            },
            None => {
                println!("while sending attributes, this mesh does 
                not have the '{}' buffer, not rendering", name);
                can_render = false;
                break;
            }
        }
    }

    (can_render, vertex_data_count)
}

fn object_draw_mesh(
    mb : &mesh::Mesh,
    vertex_data_count : usize)
{
    match mb.buffer_u32_get("faces") {
        //Some(ref bind) =>
        Some(bind) => unsafe {
            match (**bind).cgl_buffer_get() {
                Some(b) => {
                    let faces_data_count = bind.size_get();
                    cgl_draw_faces(b, faces_data_count as c_uint);
                    cgl_draw_end();
                },
                None => ()
            }
        },
        None => {
            match mb.draw_type {
                mesh::DrawType::Lines => {
                    let vc : usize = vertex_data_count/3;
                    unsafe {
                        cgl_draw_lines(vc as c_uint);
                    }
                },
                _ => {
                    unsafe {
                        cgl_draw(vertex_data_count as c_uint);
                    }
                }
            }
        }
    }
}

use camera2;
struct RenderGroup<'a>
{
    camera : (&'a transform::Transform, &'a camera2::Camera),
    renderables : &'a Iterator<Item=(&'a transform::Transform, mesh_render::MeshRender)>,
}

struct RenderByShader
{
    pub name : String,
    pub shader : ResTT<shader::Shader>,
    //TODO uncomment
    //pub passes : Iterator<Item=RenderGroup<'a>>
}

pub struct NewRender
{
    //String is the name you want to give to the pass, for example the shader name
    passes : HashMap<String, Box<RenderByShader>>,
    resource: Rc<resource::ResourceGroup>,
}

pub struct CameraInfo<'a>
{
    // I need each camera data
    cameras : &'a [camera2::Camera],
    //and world matrix.... how to get the world matrix/transform
    transform : &'a [transform::Transform],
    //camera_owners : &'a Vec<usize>,
    //transform_owners : &'a Vec<usize>,
}

impl NewRender
{
    pub fn new(resource : Rc<resource::ResourceGroup>
               ) -> NewRender
    {
        let r = NewRender { 
            passes : HashMap::new(),
            resource : resource,
            //cameras : Vec::new() //camera2::Camera::default(),
        };

        r
    }

    pub fn init(&mut self)
    {
    }

    pub fn resize(&mut self, w : c_int, h : c_int)
    {
        //self.camera.set_resolution(w, h);

        //let mut cam_ortho = self.camera_ortho.borrow_mut();
        //cam_ortho.set_resolution(w, h);
    }

    pub fn draw(
        &mut self,
        //mesh : &[component::mesh_render::MeshRenderer],
        //transform : &Iterator<Item=&transform::Transform>,
        renderables : &Iterator<Item=(&transform::Transform, mesh_render::MeshRender)>,
        loading : Arc<Mutex<usize>>
        ) -> bool
    {
        /*
        self.prepare_passes_objects_per(objects);

        let mut not_yet_loaded = 0;
        for p in self.passes.values()
        {
            let r = p.draw_frame(&self.resource, loading.clone());
            not_yet_loaded += r;
        }

        not_yet_loaded > 0
        */
        false
    }

    pub fn draw_frame(
        )
    {

    }

    /*
    pub fn draw(
        &self,
        shader : &shader::Shader,
        textures, uniforms, // Material_instance, shader_input...
        mesh,
        matrix)
    {
    }
    */

    fn test<'a>(&self, v : &'a Vec<transform::Transform>) -> Vec<&'a transform::Transform>
    {
        struct Dance<'a> {
            //it : &'a Iterator<Item=(usize,&'a transform::Transform)>
            it : &'a Iterator<Item=&'a transform::Transform>
        }
        //let t : &Iterator<Item=(usize,&transform::Transform)> = &v.iter().enumerate().filter(|&(x,y)| x == 0) as &Iterator<Item=(usize,&transform::Transform)>;
        let t : &Iterator<Item=&transform::Transform> = &v.iter().enumerate().filter_map(|(x,y)| if x == 0 { Some(y)} else { None});// as &Iterator<Item=(usize,&transform::Transform)>;

        let d = Dance {
            it : t
        };

        let vref : Vec<&transform::Transform> = v.iter().enumerate().filter_map(|(x,y)| if x == 0 { Some(y)} else { None}).collect();
        vref
    }

}

pub struct TransformGraph<'a>
{
    parents : Vec<Option<usize>>,
    dirty : Vec<bool>,
    transforms : &'a mut Vec<transform::Transform>,
    matrix : Vec<matrix::Matrix4>,
}

/*
impl<'a> TransformGraph
{
    fn new(t : &Transforms) -> TransformGraph
    {
        let s = t.len();
        TransformGraph {
            parents : vec![None,s],
            dirty : vec![false,s],
            transforms : t,
            matrix : t.iter().map().

        }
    }
}
*/

//T is used to identify your object, is the id of your object/entity,
pub fn get_transforms_of_objects_in_camera_frustum<'a, T:Copy>(
    cam : &camera2::Camera,
    cam_mat : &matrix::Matrix4, 
    world_matrices : &[(&'a T, &'a matrix::Matrix4)]
    //) -> Vec<&'a T> //usize or entity
    ) -> Vec<T> //usize or entity
{
    //TODO
    //world_matrices.iter().map(|x| *x).collect()
    world_matrices.iter().map(|x| *x.0).collect()
}

fn test(w : &mut TransformGraph)
{
    let t : &mut transform::Transform = &mut w.transforms[0];
    t.position.x = 5f64;
}

pub struct ShaderInput
{
    pub textures : HashMap<String, material::Sampler>,
    //pub uniforms : HashMap<String, Box<UniformSend+'static>>,
    pub uniforms : HashMap<String, Box<shader::UniformData>>,
}

impl ShaderInput {
    pub fn new() -> ShaderInput {
        ShaderInput {
            textures : HashMap::new(),
            uniforms : HashMap::new(),
        }
    }

    pub fn from_material(mat : &material::Material) -> ShaderInput
    {
        ShaderInput {
            textures : mat.textures.clone(),
            uniforms : mat.uniforms.clone(),
        }
    }
}

//TODO put shader, mesh, shaderinput as id?
pub fn draw(
    camera_world : &matrix::Matrix4,
    object_world : &matrix::Matrix4,
    shader : &shader::Shader,
    mesh : &mesh::Mesh,
    input : &ShaderInput,
    resource : &resource::ResourceGroup,
    )
{
    //object_init_mat(material, shader, resource, load);
    //TODO do this one level up and dont pass 2 matrix, but only one
    let mat = camera_world * object_world;
    set_matrix(shader, &mat);

    let not_yet_loaded = send_shader_sampler(
        shader,
        &input.textures,
        &mut *resource.texture_manager.borrow_mut(),
        &mut *resource.fbo_manager.borrow_mut(),
        );

    //TODO what to do if not_yet_loaded > 0

    send_shader_uniforms(
        shader,
        &input.uniforms
        );
    
    draw_mesh(shader, mesh);
}

fn set_matrix(shader :&shader::Shader, matrix : &matrix::Matrix4)
{
    shader.uniform_set("matrix", matrix);
}

fn draw_mesh(shader : &shader::Shader, mesh : &mesh::Mesh)
{
    let (can_render, vertex_data_count) = init_mesh(mesh, shader);
    object_draw_mesh(mesh, vertex_data_count);
}

use std::hash::Hash;
pub struct CameraIdMat<Id:Hash+Clone>
{
    pub id : Id,
    pub orientation : transform::Orientation,
    pub matrix : matrix::Matrix4
}

impl CameraIdMat<uuid::Uuid> {
    
    pub fn from_transform_camera(camera : &TransformCamera) -> CameraIdMat<uuid::Uuid>
    {
        let local = camera.transform.compute_return_local_matrix();
        let per = camera.get_perspective();
        let cam_mat_inv = local.get_inverse();
        let matrix = &per * &cam_mat_inv;

        CameraIdMat {
            id : camera.id,
            orientation : camera.transform.orientation,
            matrix : matrix
        }
    }
}

pub struct TransformCamera
{
    pub id : uuid::Uuid,
    pub data : camera::CameraData,
    pub transform : transform::Transform
}

impl TransformCamera
{
    pub fn new() -> TransformCamera
    {
        TransformCamera {
            id : uuid::Uuid::new_v4(),
            data : camera::CameraData::default(),
            transform : Default::default()
        }
    }

    pub fn pan(&mut self, t : &vec::Vec3)
    {
        self.data.local_offset = self.data.local_offset + *t;
        let tt = self.transform.orientation.rotate_vec3(t);
        self.transform.position = self.transform.position + tt;
    }

    pub fn set_resolution(&mut self, w : i32, h : i32)
    {
        if w as f64 != self.data.width || h as f64 != self.data.height {
            self.data.width = w as f64;
            self.data.height = h as f64;
            self.update_projection();
            //cam.update_orthographic(c);
        }
    }

    pub fn update_projection(&mut self)
    {
        self.data.aspect = self.data.width/ self.data.height;
        self.data.fovy = self.data.fovy_base * self.data.height/ (self.data.height_base as f64);
        //mat4_set_perspective(c->projection, c->fovy, c->aspect , c->near, c->far);
    }

    pub fn get_perspective(&self) -> matrix::Matrix4
    {
        match self.data.projection {
            camera::Projection::Perspective =>
                matrix::Matrix4::perspective(
                    self.data.fovy,
                    self.data.aspect,
                    self.data.near,
                    self.data.far),
            camera::Projection::Orthographic => 
                matrix::Matrix4::orthographic(
                    (self.data.width / 2f64) as u32,
                    (self.data.height / 2f64) as u32,
                    self.data.near,
                    self.data.far)
        }
    }
}
