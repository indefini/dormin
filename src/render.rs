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
    mat : matrix::Matrix4,
    mr : mesh_render::MeshRender
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
    transform : transform::Transform,
    mr : mesh_render::MeshRender
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
        self.mr.material.get_instance().unwrap().set_uniform_data(name, data);
    }
}


struct CameraPass
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

    fn add_mmr(&mut self, mr : MatrixMeshRender)
    {
        self.mmr.push(mr);
    }
}

struct RenderPass
{
    pub name : String,
    //pub shader : Arc<RwLock<shader::Shader>>,
    pub shader : ResTT<shader::Shader>,
    //uuid is the camera id
    pub passes : HashMap<uuid::Uuid, Box<CameraPass>>,
}

impl RenderPass
{
    pub fn new(
        shader : ResTT<shader::Shader>
        )
        -> RenderPass
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

        let init_mesh_render = |mr : &mesh_render::MeshRender|  -> (bool, usize)
        {
            let mesh_manager = &mut *resource.mesh_manager.borrow_mut();

            let debug = mr.mesh.name.clone();
            if let Some(m) = resource::resource_get_ref(mesh_manager, &mr.mesh) {
                init_mesh(m, shader)
            }
            else {
                (false, 0usize)
            }
        };

        let (can_render, vertex_data_count) = init_mesh_render(mesh_render);

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

        return not_loaded;
    }
}

pub struct Render
{
    passes : HashMap<String, Box<RenderPass>>, //TODO check

    resource : Rc<resource::ResourceGroup>,

    //camera_ortho : camera::Camera,
    camera_ortho : TransformCamera,

    //fbo_all : Arc<RwLock<fbo::Fbo>>,
    //fbo_selected : Arc<RwLock<fbo::Fbo>>,
    fbo_all : usize,
    fbo_selected : usize,

    quad_outline : TransformMeshRender,
    quad_all : TransformMeshRender,

    //shoud be view objects?
    grid : TransformMeshRender,
    camera_repere : TransformMeshRender,

    line : TransformMeshRender,
}

impl Render {

    //TODO remove dragger and put "view_objects"
    pub fn new(resource : Rc<resource::ResourceGroup> ) -> Render
    {
        let fbo_all = resource.fbo_manager.borrow_mut().request_use_no_proc_new("fbo_all");
        let fbo_selected = resource.fbo_manager.borrow_mut().request_use_no_proc_new("fbo_selected");

        let camera_ortho =
        {
            let mut cam = TransformCamera::new();
            cam.data.projection = camera::Projection::Orthographic;
            cam.pan(&vec::Vec3::new(0f64,0f64,50f64));
            cam
        };

        let quad_outline = {
            let mut m = mesh::Mesh::new();
            m.add_quad(1f32, 1f32);

            let outline_mat = resource.material_manager.borrow_mut().request_use_no_proc_tt_instance("material/outline.mat");

            let mere = mesh_render::MeshRender::new_with_mesh_and_mat_res(ResTT::new_with_instance("outline_quad", m), outline_mat);
             TransformMeshRender::with_mesh(mere)
        };

        let quad_all = {
            let mut m = mesh::Mesh::new();
            m.add_quad(1f32, 1f32);

            let all_mat = resource.material_manager.borrow_mut().request_use_no_proc_tt_instance("material/fbo_all.mat");

            let mere = mesh_render::MeshRender::new_with_mesh_and_mat_res(ResTT::new_with_instance("quad_all", m), all_mat);
             TransformMeshRender::with_mesh(mere)
        };

        let grid = {
            let mut m = mesh::Mesh::new();
            create_grid(&mut m, 100i32, 1i32);

            let mere = mesh_render::MeshRender::new_with_mesh(
                m,
                "material/line.mat");
             TransformMeshRender::with_mesh(mere)
        };

        let camera_repere = {
            let mut m = mesh::Mesh::new();
            create_repere(&mut m, 40f64 );

            let mere = mesh_render::MeshRender::new_with_mesh(
                m,
                "material/line.mat");
            TransformMeshRender::with_mesh(mere)
        };

        let line =
        {
            let m = mesh::Mesh::new();
            let mere = mesh_render::MeshRender::new_with_mesh(
                m,
                "material/line.mat");
            TransformMeshRender::with_mesh(mere)
        };

        Render { 
            passes : HashMap::new(),
            camera_ortho : camera_ortho,
            fbo_all : fbo_all,
            fbo_selected : fbo_selected,
            quad_outline : quad_outline,
            quad_all : quad_all, 


            grid : grid,
            camera_repere : camera_repere,

            line : line, 
            resource : resource.clone()
        }
    }

    pub fn init(&mut self)
    {
        //self.fbo_all.write().unwrap().cgl_create();
        let mut fbo_mgr = self.resource.fbo_manager.borrow_mut();
        {
            let fbo_all = fbo_mgr.get_mut_or_panic(self.fbo_all);
            //fbo_all.write().unwrap().cgl_create();
            fbo_all.cgl_create();
        }

        let fbo_sel = fbo_mgr.get_mut_or_panic(self.fbo_selected);
        fbo_sel.cgl_create();
    }

    pub fn resize(&mut self, w : c_int, h : c_int)
    {
        {
            self.quad_outline.transform.scale = 
                vec::Vec3::new(w as f64, h as f64, 1f64);

            self.quad_all.transform.scale = 
                vec::Vec3::new(w as f64, h as f64, 1f64);

            self.camera_ortho.set_resolution(w, h);

            //self.fbo_all.write().unwrap().cgl_resize(w, h);
            //let fbo_all = &self.resource.fbo_manager.borrow().get_from_state(self.fbo_all);
            //fbo_all.write().unwrap().cgl_resize(w, h);
            let mut fbo_mgr = self.resource.fbo_manager.borrow_mut();
            {
                let fbo_all = fbo_mgr.get_mut_or_panic(self.fbo_all);
                fbo_all.cgl_resize(w, h);
                //    self.fbo_selected.write().unwrap().cgl_resize(w, h);
            }

            let fbo_sel = fbo_mgr.get_mut_or_panic(self.fbo_selected);
            fbo_sel.cgl_resize(w,h);
        }

        self.resolution_set(w,h);
    }


    fn resolution_set(&mut self, w : c_int, h : c_int)
    {
        self.quad_outline.set_uniform_data(
            "resolution",
            shader::UniformData::Vec2(vec::Vec2::new(w as f64, h as f64)));

        /*
        self.quad_all.clone().read().unwrap().set_uniform_data(
            "resolution",
            shader::UniformData::Vec2(vec::Vec2::new(w as f64, h as f64)));
            */
    }

    fn clean_passes(&mut self)
    {
        for (_,p) in self.passes.iter_mut()
        {
            //p.objects.clear();
            p.passes.clear();
        }

        {
            let mesh = &mut self.line.mr.mesh.get_instance().unwrap();//.write().unwrap();
            mesh.clear_lines();
        }
    }


    //TODO draw armature
    /*
    fn add_objects_to_passes(
        &mut self,
        camera : &CameraIdMat,
        objects : &[Arc<RwLock<object::Object>>]
        )
    {
        for o in objects.iter() {
            prepare_passes_object(
                o.clone(),
                &mut self.passes,
                &mut self.resource.material_manager.borrow_mut(),
                &mut self.resource.shader_manager.borrow_mut(),
                camera);
            
            let ob =  &*o.read().unwrap();
            match ob.get_component::<armature_animation::ArmatureAnimation>() {
                Some(ref aa) => {
                    let armature = &aa.arm_instance;
                    let color = vec::Vec4::new(1f64,1f64,1f64,1.0f64);

                    let line : &mut object::Object = &mut *self.line.write().unwrap();
                    if let Some(ref mut mr) = line.mesh_render
                    {
                        let mesh = &mut mr.mesh.get_instance().unwrap();

                        let arm_pos = ob.position + ob.orientation.rotate_vec3(&(armature.position*ob.scale));
                        let cur_rot = ob.orientation.as_quat() * armature.rotation;

                        for i in 0..armature.get_bones().len() {
                            let b = armature.get_bone(i);
                            let current_bone_position = armature.position_relative[i];
                            let current_bone_rotation = cur_rot*armature.rotation_relative[i];
                            let p1 = arm_pos + cur_rot.rotate_vec3(&(current_bone_position*ob.scale));
                            let bone_length = (b.tail - b.head)*ob.scale;
                            let diff = current_bone_rotation.rotate_vec3(&bone_length);
                            let p2 = p1 + diff;
                            let s = geometry::Segment::new(p1,p2);
                            mesh.add_line(s, color);
                        }
                    }
                }
                None => {}// println!("{} nooooooo", ob.name)}
            };
        }
    }
    */

    fn prepare_passes_objects_ortho(&mut self, mmr : &[MatrixMeshRender])
    {
        for (_,p) in self.passes.iter_mut()
        {
            //p.objects.clear();
            p.passes.clear();
        }

        let cam = CameraIdMat::from_transform_camera(&self.camera_ortho);
        self.add_mmr(&cam, &mmr);
    }

    fn prepare_passes_objects_per_mmr(
        &mut self,
        camera : &CameraIdMat,
        mmr : &[MatrixMeshRender])
    {
        let load = Arc::new(Mutex::new(0));
        for (_,p) in self.passes.iter_mut()
        {
            p.passes.clear();
        }

        self.add_mmr(camera, mmr);
    }

    fn add_mmr(
        &mut self,
        camera : &CameraIdMat,
        mmr : &[MatrixMeshRender])
    {
        let load = Arc::new(Mutex::new(0));

        for m in mmr {
            let pass = get_pass_from_mesh_render(
                &m.mr,
                &mut self.passes,
                &mut self.resource.material_manager.borrow_mut(),
                &mut self.resource.shader_manager.borrow_mut(),
                camera,
                load.clone()
                );

            if let Some(cam_pass) = pass {
                cam_pass.add_mmr(m.clone());
            }
        }
    }

    pub fn draw(
        &mut self,
        camera : &CameraIdMat,
        objects : &[MatrixMeshRender],
        cameras : &[MatrixMeshRender],
        selected : &[MatrixMeshRender],
        draggers : &[MatrixMeshRender],
        on_finish : &Fn(bool),
        load : Arc<Mutex<usize>>
        ) -> usize
    {
        let mut not_loaded = 0;
        self.prepare_passes_objects_per_mmr(camera, selected);
        {
        let mut fbo_mgr = self.resource.fbo_manager.borrow_mut();
        let fbo_sel = fbo_mgr.get_mut_or_panic(self.fbo_selected);
        fbo_sel.cgl_use();
        }
        for p in self.passes.values()
        {
            let not = p.draw_frame(
                &self.resource,
                load.clone()
                );

            not_loaded += not;
        }
        fbo::Fbo::cgl_use_end();

        self.clean_passes();
        println!("OOOOBBBB :::::::::::::: {}", objects.len());
        self.add_mmr(camera, objects);
        self.add_mmr(camera, cameras);

        {
            let mmr = self.grid.to_mmr();
            self.add_mmr(camera, vec![mmr].as_slice());

            let m = 40f64;
            self.camera_repere.transform.position = 
                vec::Vec3::new(
                    -self.camera_ortho.data.width/2f64 +m, 
                    -self.camera_ortho.data.height/2f64 +m, 
                    -10f64);
            self.camera_repere.transform.orientation = 
                camera.orientation.inverse();

            let cam = CameraIdMat::from_transform_camera(&self.camera_ortho);
            let mmr = self.camera_repere.to_mmr();
            self.add_mmr(&cam, vec![mmr].as_slice());
        }

        {
        //self.fbo_all.read().unwrap().cgl_use();
        //let fbo_all = &self.resource.fbo_manager.borrow().get_from_state(self.fbo_all);
        //fbo_all.write().unwrap().cgl_use();
        
        let mut fbo_mgr = self.resource.fbo_manager.borrow_mut();
        let fbo_all = fbo_mgr.get_mut_or_panic(self.fbo_all);
        fbo_all.cgl_use();
        for p in self.passes.values()
        {
            let not = p.draw_frame(
                &self.resource,
                load.clone()
                );

            not_loaded = not_loaded + not;
        }
        fbo::Fbo::cgl_use_end();
        }

        /*
        for p in self.passes.values()
        {
            p.draw_frame(
                self.mesh_manager.clone(),
                self.material_manager.clone(),
                self.shader_manager.clone(),
                self.texture_manager.clone(),
                self.fbo_manager.clone(),
                );
        }
        */


        let l = vec![self.quad_all.to_mmr()];
        self.prepare_passes_objects_ortho(&l);

        for p in self.passes.values()
        {
            let not = p.draw_frame(
                &self.resource,
                load.clone()
                );

            not_loaded = not_loaded + not;
        }

        let sel_len = selected.len();
        println!("selllll :::::::::::::: {}", sel_len);

        if sel_len > 0 {
            let l = vec![self.quad_outline.to_mmr()];
            self.prepare_passes_objects_ortho(&l);

            for p in self.passes.values()
            {
                let not = p.draw_frame(
                    &self.resource,
                    load.clone()
                    );
            
                not_loaded = not_loaded + not;
            }

            //* TODO dragger
            unsafe { cgl_clear(); }
            //ld.push_back(self.dragger.clone());
            //self.prepare_passes_objects_per(ld);
            //self.prepare_passes_objects_per(draggers);
            println!("dragger :::::::::::::: {}", draggers.len());
            self.prepare_passes_objects_per_mmr(camera, draggers);


            /*
            fn get_camera_resize_w(camera : &camera::Camera, m : &matrix::Matrix4, factor : f64) -> f64
            {
                let cam_mat = camera.object.read().unwrap().get_world_matrix();
                let projection = camera.get_perspective();

                let cam_mat_inv = cam_mat.get_inverse();
                let world_inv = &cam_mat_inv * m;

                let mut tm = &projection * &world_inv;
                tm = tm.transpose();

                let zero = vec::Vec4::new(0f64,0f64,0f64,1f64);
                let vw = &tm * zero;
                let w = vw.w * factor;
                return w;
            }


            
            let scale = get_camera_resize_w(&*self.camera.borrow(),
                &draggers.front().unwrap().read().unwrap().get_matrix(),
                0.05f64);
            //add_box(&mut *self.line.write().unwrap(), selected, scale as f32);
            add_box_only_first_object(&mut *self.line.write().unwrap(), draggers, scale);

            prepare_passes_object(
                self.line.clone(),
                &mut self.passes,
                self.material_manager.clone(),
                self.shader_manager.clone(),
                self.camera.clone());
             */

            let mmr = self.line.to_mmr();
            self.add_mmr(camera, vec![mmr].as_slice());

            for p in self.passes.values()
            {
                let not = p.draw_frame(
                    &self.resource,
                    load.clone()
                    );
                not_loaded = not_loaded + not;
            }
            //*/
        }

        on_finish(false);
        not_loaded
    }
}

fn get_pass_from_mesh_render<'a>(
    mr : &mesh_render::MeshRender,
    passes : &'a mut HashMap<String, Box<RenderPass>>, 
    material_manager : &mut resource::ResourceManager<material::Material>,
    shader_manager : &mut resource::ResourceManager<shader::Shader>,
    camera : &CameraIdMat,
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

fn create_grid(m : &mut mesh::Mesh, num : i32, space : i32)
{
    //TODO make something better then using add_line
    //ie create the vec and then add the buffer

    let color = vec::Vec4::new(1f64,1f64,1f64,0.1f64);
    let xc = vec::Vec4::new(1.0f64,0.247f64,0.188f64,0.4f64);
    let zc = vec::Vec4::new(0f64,0.4745f64,1f64,0.4f64);

    for i in  -num..num {
        let p1 = vec::Vec3::new((i*space) as f64, 0f64, (-space*num) as f64);
        let p2 = vec::Vec3::new((i*space) as f64, 0f64, (space*num) as f64);
        let s = geometry::Segment::new(p1,p2);
        if i == 0 {
            m.add_line(s, zc);
        }
        else {
            m.add_line(s, color);
        }
    }

    for i in  -num..num {
        let p1 = vec::Vec3::new((-space*num) as f64, 0f64, (i*space) as f64);
        let p2 = vec::Vec3::new((space*num) as f64, 0f64, (i*space) as f64);
        let s = geometry::Segment::new(p1,p2);
        if i == 0 {
            m.add_line(s, xc);
        }
        else {
            m.add_line(s, color);
        }
    }
}

fn create_repere(m : &mut mesh::Mesh, len : f64)
{
    let red = vec::Vec4::new(1.0f64,0.247f64,0.188f64,1f64);
    let green = vec::Vec4::new(0.2117f64,0.949f64,0.4156f64,1f64);
    let blue = vec::Vec4::new(0f64,0.4745f64,1f64,1f64);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(len, 0f64, 0f64));
    m.add_line(s, red);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(0f64, len, 0f64));
    m.add_line(s, green);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(0f64, 0f64, len));
    m.add_line(s, blue);
}

pub struct GameRender
{
    passes : HashMap<String, Box<RenderPass>>, //TODO check

    resource: Rc<resource::ResourceGroup>,

        /*
    mesh_manager : Arc<RwLock<resource::ResourceManager<mesh::Mesh>>>,
    shader_manager : Arc<RwLock<resource::ResourceManager<shader::Shader>>>,
    texture_manager : Arc<RwLock<resource::ResourceManager<texture::Texture>>>,
    material_manager : Arc<RwLock<resource::ResourceManager<material::Material>>>,
    fbo_manager : Arc<RwLock<resource::ResourceManager<fbo::Fbo>>>,
    */

    //camera_ortho : Rc<RefCell<camera::Camera>>,
}

impl GameRender {

    //TODO remove dragger and put "view_objects"
    pub fn new(//factory: &mut factory::Factory,
               resource : Rc<resource::ResourceGroup>
               //dragger : Arc<RwLock<object::Object>>,
               ) -> GameRender
    {
        /*
        let camera_ortho = Rc::new(RefCell::new(factory.create_camera()));
        {
            let mut cam = camera_ortho.borrow_mut();
            cam.data.projection = camera::Projection::Orthographic;
            cam.pan(&vec::Vec3::new(0f64,0f64,50f64));
        }
        */

        /*
        let material_manager = Arc::new(RwLock::new(resource::ResourceManager::new()));
        let shader_manager = Arc::new(RwLock::new(resource::ResourceManager::new()));
        let fbo_manager = Arc::new(RwLock::new(resource::ResourceManager::new()));
        */

        let r = GameRender { 
            passes : HashMap::new(),
            //mesh_manager : factory.mesh_manager.clone(),
            /*
            mesh_manager : Arc::new(RwLock::new(resource::ResourceManager::new())),
            shader_manager : shader_manager.clone(),
            texture_manager : Arc::new(RwLock::new(resource::ResourceManager::new())),
            material_manager : material_manager.clone(),
            fbo_manager : fbo_manager,
            */
            resource : resource,
            //camera_ortho : camera_ortho,
        };

        r
    }

    pub fn init(&mut self)
    {
    }

    pub fn resize(&mut self, w : c_int, h : c_int)
    {
        //let mut cam_ortho = self.camera_ortho.borrow_mut();
        //cam_ortho.set_resolution(w, h);
    }

    fn prepare_passes_objects_per_mmr(
        &mut self,
        camera : &CameraIdMat,
        mmr : &[MatrixMeshRender])
    {
        let load = Arc::new(Mutex::new(0));
        for (_,p) in self.passes.iter_mut()
        {
            p.passes.clear();
        }

        self.add_mmr(camera, mmr);
    }

    fn add_mmr(
        &mut self,
        camera : &CameraIdMat,
        mmr : &[MatrixMeshRender])
    {
        let load = Arc::new(Mutex::new(0));

        for m in mmr {
            let pass = get_pass_from_mesh_render(
                &m.mr,
                &mut self.passes,
                &mut self.resource.material_manager.borrow_mut(),
                &mut self.resource.shader_manager.borrow_mut(),
                camera,
                load.clone()
                );

            if let Some(cam_pass) = pass {
                cam_pass.add_mmr(m.clone());
            }
        }
    }


    pub fn draw(
        &mut self,
        camera : &CameraIdMat,
        objects : &[MatrixMeshRender],
        loading : Arc<Mutex<usize>>
        ) -> bool
    {
        self.prepare_passes_objects_per_mmr(camera, objects);

        let mut not_yet_loaded = 0;
        for p in self.passes.values()
        {
            let r = p.draw_frame(&self.resource, loading.clone());
            not_yet_loaded += r;
        }

        not_yet_loaded > 0
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
                //println!("while sending attributes, this mesh does 
                //not have the '{}' buffer, not rendering", name);
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

pub struct CameraIdMat
{
    pub id : uuid::Uuid,
    pub orientation : transform::Orientation,
    pub matrix : matrix::Matrix4
}

impl CameraIdMat {
    
    pub fn from_transform_camera(camera : &TransformCamera) -> CameraIdMat
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
    id : uuid::Uuid,
    data : camera::CameraData,
    transform : transform::Transform
}

impl TransformCamera
{
    fn new() -> TransformCamera
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
