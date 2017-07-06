use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{RwLock, Arc};


use component::{Component, Components, CompData};
use object::Object;
use transform;
use armature;
use mesh;
use resource;
use vec;
use input;
use mesh_render;

#[derive(Copy,Clone)]
pub enum State
{
    Idle,
    Play,
    Pause,
    Stop
}

#[derive(Clone)]
pub struct ArmatureAnimation
{
    state : State,
    //armature : armature::Armature,
    //armature : Arc<RwLock<armature::Armature>>,
    armature : resource::ResTT<armature::Armature>,
    pub arm_instance : armature::ArmatureInstance,
    mesh : Option<resource::ResTT<mesh::Mesh>>,
    action : Option<String>,
    time : f64

    //TODO mesh component + dependencies
    //mesh_base : Option<resource::ResTT<MeshRenderComponent>>,
    //mesh_renderer : Rc<component::meshrender::MeshRenderer>,
}

impl ArmatureAnimation
{
    fn update(
        &mut self,
        dt : f64,
        mr : &mut mesh_render::MeshRender,
        resource : &resource::ResourceGroup
        )
    {
        let action = if let Some(ref a) = self.action {
            a
        }
        else {
            //println!("update armature anim : no action");
            return
        };

        self.time = self.time + dt;
        if self.time > 50f64/30f64 {
            self.time = 0f64;
        }

        let armature_manager = &mut *resource.armature_manager.borrow_mut();
        let arm_base = self.armature.get_ref(armature_manager).unwrap();

        self.arm_instance.set_pose(arm_base, action.as_str(), self.time);

        let base_mesh = mr.get_mesh();
        //let base = base_mesh.read().unwrap();
        let mm = &mut *resource.mesh_manager.borrow_mut();
        let base = base_mesh.get_ref(mm).unwrap();
        let mut mi = mr.get_or_create_mesh_instance();
        update_mesh_with_armature(&base, mi, &self.arm_instance);

        //let normal_pose = 

        //TODO get the current animation pose with the action name and the time.
        // get the bones translation and rotation DIFFERENCE with the original pose.
        // ...
        //get the original mesh and apply weights 

    }

}

impl Component for ArmatureAnimation
{
    /*
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(box))
    }
    */

    /*
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(
                box ArmatureAnimation
                {
                    state : self.state,
                    armature : self.armature.clone(),
                    mesh : self.mesh.clone(),
                    arm_instance : self.arm_instance.clone(),
                    action : self.action.clone(),
                    time : self.time

                }))
    }
    */

    /*
    fn update(
        &mut self,
        ob : &mut Object,
        dt : f64,
        input : &input::Input,
        resource : &resource::ResourceGroup
        )
    {
        if let Some(ref mut mr) = ob.mesh_render {
            self.update(dt, mr, resource);
        }
    }
    */

    fn get_name(&self) -> String {
        "armature_animation".to_owned()
    }
}

/*
pub fn new(ob : &Object, resource : &resource::ResourceGroup) -> Box<Components>
{
    println!("armature anim new---->>>>");
    let arm = {
        match ob.get_comp_data::<armature::ArmaturePath>(){
            Some(a) => a.clone(),
            None => panic!("no armature data")
        }
    };

    let armature_manager = &mut *resource.armature_manager.borrow_mut();
    let armature = armature_manager.request_use_no_proc_tt(arm.name.as_ref());
    let instance = {
        let arm_base = armature.get_ref(armature_manager).unwrap();
        arm_base.create_instance()
    };

    let arm_anim = ArmatureAnimation {
        state : State::Idle,
        armature : armature,
        arm_instance : instance,
        mesh : None,
        action : None,//Some(String::from("roll")),//None,
        time : 0f64
    };

    box Components::ArmatureAnimation(arm_anim)
}
*/

//TODO
fn update_mesh_with_armature(
    base : &mesh::Mesh,
    mesh : &mut mesh::Mesh,
    arm : &armature::ArmatureInstance)
{
    let mut i = 0;
    for v in &base.weights {
        //TODO get vertex and normal
        let vertex_pos = if let Some(b) = base.buffer_f32_get("position") {
            vec::Vec3::new(
                b.data[i*3] as f64,
                b.data[i*3+ 1] as f64,
                b.data[i*3+ 2] as f64)
        }
        else {
            println!("no buffer position in base");
            return;
        };
        let vertex_nor = if let Some(b) = base.buffer_f32_get("normal") {
            vec::Vec3::new(
                b.data[i*3] as f64,
                b.data[i*3+ 1] as f64,
                b.data[i*3+ 2] as f64)
        }
        else {
            println!("no buffer normal in base");
            return;
        };

        //TODO rotation
        let mut translation = vec::Vec3::zero();
        let mut rotation = vec::Quat::identity();
        for w in v.iter() {
            //TODO TODO
            // slime used to try to find the bone with name
            let bone = arm.get_bone(w.index as usize);
            let pos_relative = arm.position_relative[w.index as usize];
            let rot_relative = arm.rotation_relative[w.index as usize];

            if w.weight == 0f32 {
                continue;
            }

            let vpos_from_bone = vertex_pos - bone.head_from_arm;

            let bone_tr_diff = (pos_relative - bone.head_from_arm) * w.weight +
                (rot_relative.rotate_vec3(&vpos_from_bone)-vpos_from_bone)*w.weight;
            let bone_rt_diff = vec::quat_slerp(
                vec::Quat::identity(),
                rot_relative,
                w.weight as f64);

            translation = translation + bone_tr_diff;
            rotation = rotation * bone_rt_diff;
        }

        let newpos = vertex_pos + translation;
        let newnor = rotation.rotate_vec3(&vertex_nor);

        mesh.set_dirty();

        if let Some(b) = mesh.buffer_f32_get_mut("position") {
            b.data[i*3] = newpos.x as f32;
            b.data[i*3+ 1] = newpos.y as f32;
            b.data[i*3+ 2] = newpos.z as f32;
        }
        else {
            println!("no buffer position");
        };

        if let Some(b) = mesh.buffer_f32_get_mut("normal") {
            b.data[i*3] = newnor.x as f32;
            b.data[i*3+ 1] = newnor.y as f32;
            b.data[i*3+ 2] = newnor.z as f32;
        }
        else {
            println!("no buffer normal");
            return;
        };

        i = i+1;
    }





}
