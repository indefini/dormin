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

#[derive(Clone, Serialize, Deserialize)]
pub struct MeshRender
{
    pub mesh : ResTT<mesh::Mesh>,
    pub material : ResTT<material::Material>,
}

impl Default for MeshRender {
    fn default() -> MeshRender {
        MeshRender  {
            mesh : ResTT::new("no_mesh"),
            material : ResTT::new("no_mat"),
        }
    }
}

impl Component for MeshRender
{
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

impl MeshRender{

    pub fn get_or_create_mesh_instance(& mut self) -> & mut mesh::Mesh
    {
        self.mesh.get_or_create_instance()
    }

    pub fn get_mesh(&self) -> ResTT<mesh::Mesh>
    {
        self.mesh.clone()
    }

    pub fn with_names(mesh : &str, material : &str, resource : &resource::ResourceGroup) -> MeshRender
    {
        MeshRender {
            mesh : resource.mesh_manager.borrow_mut().get_handle_instant(mesh),
            material : resource.material_manager.borrow_mut().get_handle_instant(material),
        }
    }

    pub fn with_names_only(mesh : &str, material : &str) -> MeshRender
    {
        MeshRender {
            mesh : resource::ResTT::new(mesh),
            material : resource::ResTT::new(material)
        }
    }

    pub fn new_with_mesh_res(
        mesh : ResTT<mesh::Mesh>,
        material : &str,
        resource : &resource::ResourceGroup) -> MeshRender
    {
        MeshRender {
            mesh : mesh,
            material : resource.material_manager.borrow_mut().get_handle_instant(material),
        }
    }

    pub fn new_with_mesh(
        mesh : mesh::Mesh,
        material : &str,
        resource : &resource::ResourceGroup) -> MeshRender
    {
        MeshRender {
            //TODO
            mesh : ResTT::new_with_instance("none", mesh),
            material : resource.material_manager.borrow_mut().get_handle_instant(material),
        }
    }

    pub fn new_with_mat(
        mesh : &str, 
        material : material::Material,
        resource : &resource::ResourceGroup) -> MeshRender
    {
        MeshRender {
            mesh : resource.mesh_manager.borrow_mut().get_handle_instant(mesh),
            material : ResTT::new_with_instance("no_name", material),
        }
    }

    pub fn new_with_mat_res(
        mesh : &str, 
        //material : Arc<RwLock<material::Material>>,
        material : ResTT<material::Material>,
        resource : &resource::ResourceGroup) -> MeshRender
    {
        MeshRender {
            mesh : resource.mesh_manager.borrow_mut().get_handle_instant(mesh),
            material : material,
        }
    }


    pub fn new_with_mesh_and_mat_res(
        mesh : ResTT<mesh::Mesh>,
        material : ResTT<material::Material>) 
        -> MeshRender
    {
        MeshRender {
            mesh : mesh,
            material : material,
        }
    }

    pub fn new_with_mesh_and_mat(
        mesh : mesh::Mesh,
        material : material::Material) 
        -> MeshRender
    {
        MeshRender {
            mesh : ResTT::new_with_instance("none", mesh),
            material : ResTT::new_with_instance("none", material),
        }
    }
}

property_set_impl!(MeshRender,[mesh,material]);
property_get_impl!(MeshRender,[mesh,material]);

