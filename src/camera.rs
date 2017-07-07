use std::sync::{RwLock, Arc};
use std::f64::consts;
use std::default::Default;
use serde;

use vec;
use vec::{Vec3};
use matrix;
use geometry;
use transform::Orientation;
use uuid;
use render::CameraIdMat;
use resource;
use transform;

#[derive(Clone, Serialize, Deserialize)]
pub enum Projection
{
    Perspective,
    Orthographic
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CameraData
{
    pub fovy : f64,
    pub fovy_base : f64,
    pub near : f64,
    pub far : f64,
    pub aspect : f64,
    pub width : f64,
    pub height : f64,
    pub height_base : i32,
    pub yaw : f64,
    pub pitch : f64,
    pub roll : f64,

    pub clear_color : vec::Vec4,
    pub projection : Projection,

    pub origin : vec::Vec3,
    pub local_offset : vec::Vec3,
    pub center : vec::Vec3,

    //pub euler : vec::Vec3
}

impl Default for CameraData
{
    fn default() -> CameraData
    {
        CameraData {
            fovy : consts::PI/8.0f64,
            fovy_base : consts::PI/8.0f64,
            near : 1f64,
            far : 10000f64,
            aspect : 1.6f64,
            width : 800f64,
            height : 500f64,
            height_base : 500i32,
            yaw : 0f64,
            pitch : 0f64,
            roll : 0f64,

            origin : vec::Vec3::zero(),
            local_offset : vec::Vec3::zero(),
            center : vec::Vec3::zero(),

            clear_color : vec::Vec4::zero(),

            projection : Projection::Perspective,

            //euler : vec::Vec3::zero(),
        }
    }
}


