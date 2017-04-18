use factory;
use object;
use mesh;
//use mesh_render;
use resource;
use std::sync::{Arc,RwLock};

pub struct Table
{
    height : f64,
    width : f64,
    length : f64,
}

pub fn create_table_object(factory : &mut factory::Factory, table : Table) -> object::Object
{
    let o = factory.create_object("table");
    /*
    let m = create_table_mesh(table);
    o.mesh_render = Some(create_mesh_render(m));
    */
    o
}

pub fn create_table_mesh(table : Table) -> mesh::Mesh
{
    let mut m = mesh::Mesh::new();
    create_table(&mut m);
    m
}

fn create_table(m : &mut mesh::Mesh)
{
}


