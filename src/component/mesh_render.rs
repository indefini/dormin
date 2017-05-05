use input;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{RwLock, Arc};

use component::{Component, CompData};
use object::Object;
use transform;
use mesh;
use resource;
use resource::ResTT;
use material;

use property::{PropertyRead, PropertyGet, PropertyWrite, WriteValue};
use std::any::Any;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct MeshRender
{
    pub mesh : String,
    pub material : String,
}

impl MeshRender
{
    pub fn new(mesh : &str, material : &str) -> MeshRender
    {
        MeshRender {
            mesh : mesh.to_owned(),
            material : material.to_owned(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MeshRenderer
{
    pub mesh : ResTT<mesh::Mesh>,
    pub material : ResTT<material::Material>,
}

impl Component for MeshRenderer
{
    /*
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(
                box MeshRenderer
                {
                    mesh : self.mesh.clone(),
                    material : self.material.clone(),
                    mesh_instance : None,
                        //match self.mesh_instance {
                        //None => None,
                        //Some(m) => Some(m.clone())
                    //},
                    material_instance : None,
                        //match self.material_instance {
                        //None => None,
                        //Some(m) => Some(m.clone())
                    //},
                }))
    }
    */

    fn update(
        &mut self,
        ob : &mut Object,
        dt : f64,
        input : &input::Input,
        resource : &resource::ResourceGroup
        )
    {
    }

    fn get_name(&self) -> String {
        "mesh_render".to_owned()
    }
}

impl MeshRenderer{

    pub fn get_or_create_mesh_instance(& mut self) -> & mut mesh::Mesh
    {
        self.mesh.get_or_create_instance()
    }

    pub fn get_mesh(&self) -> ResTT<mesh::Mesh>// Arc<RwLock<mesh::Mesh>>
    {
        self.mesh.clone()
    }

    pub fn new(ob : &Object, resource : &resource::ResourceGroup) -> MeshRenderer
    {
        let mesh_render = {
            match ob.get_comp_data::<MeshRender>(){
                Some(m) => m.clone(),
                None => panic!("no mesh data")
            }
        };

        MeshRenderer::with_mesh_render(&mesh_render, resource)
    }

    pub fn with_names(mesh : &str, material : &str, resource : &resource::ResourceGroup) -> MeshRenderer
    {
        MeshRenderer {
            mesh : resource.mesh_manager.borrow_mut().get_handle_instant(mesh),
            material : resource.material_manager.borrow_mut().get_handle_instant(material),
        }
    }

    pub fn with_names_only(mesh : &str, material : &str) -> MeshRenderer
    {
        MeshRenderer {
            mesh : resource::ResTT::new(mesh),
            material : resource::ResTT::new(material)
        }
    }


    pub fn with_mesh_render(mesh_render : &MeshRender, resource : &resource::ResourceGroup) -> MeshRenderer
    {
        MeshRenderer::with_names(mesh_render.mesh.as_str(), mesh_render.material.as_str(), resource)
    }

    pub fn new_with_mesh_res(
        //mesh : Arc<RwLock<mesh::Mesh>>,
        mesh : ResTT<mesh::Mesh>,
        material : &str,
        resource : &resource::ResourceGroup) -> MeshRenderer
    {
        MeshRenderer {
            mesh : mesh,
            material : resource.material_manager.borrow_mut().get_handle_instant(material),
        }
    }

    pub fn new_with_mesh(
        mesh : mesh::Mesh,
        material : &str,
        resource : &resource::ResourceGroup) -> MeshRenderer
    {
        MeshRenderer {
            //TODO
            mesh : ResTT::new_with_instance("none", mesh),
            material : resource.material_manager.borrow_mut().get_handle_instant(material),
        }
    }

    pub fn new_with_mat(
        mesh : &str, 
        material : material::Material,
        resource : &resource::ResourceGroup) -> MeshRenderer
    {
        MeshRenderer {
            mesh : resource.mesh_manager.borrow_mut().get_handle_instant(mesh),
            material : ResTT::new_with_instance("no_name", material),
        }
    }

    pub fn new_with_mat_res(
        mesh : &str, 
        //material : Arc<RwLock<material::Material>>,
        material : ResTT<material::Material>,
        resource : &resource::ResourceGroup) -> MeshRenderer
    {
        MeshRenderer {
            mesh : resource.mesh_manager.borrow_mut().get_handle_instant(mesh),
            material : material,
        }
    }


    pub fn new_with_mesh_and_mat_res(
        mesh : ResTT<mesh::Mesh>,
        material : ResTT<material::Material>) 
        -> MeshRenderer
    {
        MeshRenderer {
            mesh : mesh,
            material : material,
        }
    }

    pub fn new_with_mesh_and_mat(
        mesh : mesh::Mesh,
        material : material::Material) 
        -> MeshRenderer
    {
        MeshRenderer {
            mesh : ResTT::new_with_instance("none", mesh),
            material : ResTT::new_with_instance("none", material),
        }
    }
}

pub fn new(ob : &Object, resource : &resource::ResourceGroup) -> Box<Component>
{
    box MeshRenderer::new(ob, resource)
}

property_set_impl!(MeshRender,[mesh,material]);
property_get_impl!(MeshRender,[mesh,material]);

