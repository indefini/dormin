use std::ptr;
use libc::{c_char};
use std::ffi::CString;
use std::ops::{Mul};
use std::fmt;
use std::default::Default;
use std::cell::Cell;

use vec;
use matrix;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Orientation
{
    AngleXYZ(vec::Vec3),
    Quat(vec::Quat)
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::AngleXYZ(vec::Vec3::default())
    }
}

impl Orientation
{
    pub fn to_angle_xyz(&mut self) {
        match *self {
            Orientation::Quat(q) => *self = Orientation::AngleXYZ(q.to_euler_deg()),
            _ => {}
        }
    }

    pub fn to_quat(&mut self) {
        match *self {
            Orientation::AngleXYZ(a) => *self = Orientation::Quat(vec::Quat::new_angles_deg(&a)),
            _ => {}
        }
    }

    pub fn as_quat(&self) -> vec::Quat
    {
        match *self {
            Orientation::AngleXYZ(a) => vec::Quat::new_angles_deg(&a),
            Orientation::Quat(q) => q
        }
    }

    pub fn new_with_quat(q : &vec::Quat) -> Orientation
    {
        Orientation::Quat(*q)
    }

    pub fn new_with_angle_xyz(v : &vec::Vec3) -> Orientation
    {
        Orientation::AngleXYZ(*v)
    }

    pub fn new_quat() -> Orientation
    {
        Orientation::Quat(vec::Quat::identity())
    }

    pub fn rotate_vec3(&self, v : &vec::Vec3) -> vec::Vec3
    {
        self.as_quat().rotate_vec3(v)
    }

    pub fn inverse(&self) -> Orientation
    {
        match *self {
            Orientation::AngleXYZ(_) => {
                //TODO
                let q = self.as_quat().inverse();
                let mut o = Orientation::new_with_quat(&q);
                o.to_angle_xyz();
                o
            },
            Orientation::Quat(q) => Orientation::Quat(q.inverse())
        }
    }

    pub fn get_angle_xyz(& self) -> vec::Vec3
    {
        match *self {
            Orientation::Quat(q) => q.to_euler_deg(),
            Orientation::AngleXYZ(a) => {a}
        }
    }

    pub fn get_quat(& self) -> vec::Quat
    {
        match *self {
            Orientation::Quat(q) => q,
            Orientation::AngleXYZ(a) => vec::Quat::new_angles_deg(&a),
        }
    }

    pub fn set_and_keep_type(&mut self, ori : Orientation)
    {
        match *self {
            Orientation::AngleXYZ(_) => 
                *self = Orientation::AngleXYZ(ori.get_angle_xyz()),
            Orientation::Quat(_) => 
                *self = Orientation::Quat(ori.get_quat())
        }
    }

    pub fn set_with_quat(&mut self, ori : vec::Quat)
    {
        match *self {
            Orientation::AngleXYZ(_) => 
                *self = Orientation::AngleXYZ(ori.to_euler_deg()),
            Orientation::Quat(_) => 
                *self = Orientation::Quat(ori)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Transform {
    pub position : vec::Vec3, 
    pub orientation : Orientation,
    pub scale : vec::Vec3, 
    #[serde(skip_serializing, skip_deserializing)]
    dirty : bool,
    #[serde(skip_serializing, skip_deserializing)]
    local_matrix : matrix::Matrix4
}

impl Transform
{
    pub fn new() -> Transform
    {
        Transform {
            position : vec::Vec3::zero(),
            orientation : Orientation::Quat(vec::Quat::identity()),
            scale : vec::Vec3::one(),
            dirty : false,
            local_matrix : matrix::Matrix4::identity()
        }
    }

    pub fn from_position_orientation_scale(
        pos : vec::Vec3,
        ori : Orientation,
        scale : vec::Vec3) -> Transform
    {
        Transform {
            position : pos,
            orientation : ori,
            scale : scale,
            dirty : true,
            local_matrix : matrix::Matrix4::identity()
        }
    }

    pub fn get_or_compute_local_matrix(&mut self) -> &matrix::Matrix4
    {
        self.compute_local_matrix();
        &self.local_matrix
    }

    pub fn get_computed_local_matrix(&self) -> &matrix::Matrix4
    {
        &self.local_matrix
    }

    pub fn compute_local_matrix(&mut self)
    {
        if self.dirty {
            //TODO optim possible?
            self.local_matrix = compute_matrix_from_position_rotation_scale(
                &self.position,
                &self.orientation.as_quat(),
                &self.scale);

            self.dirty = false;
        }
    }

    //TODO for debug
    pub fn set_as_dirty(&mut self)
    {
        self.dirty = true;
    }

    pub fn get_pos_quat(&self) -> (vec::Vec3, vec::Quat)
    {
        (self.position, self.orientation.as_quat())
    }
}

fn compute_matrix_from_position_rotation_scale(
    position : &vec::Vec3,
    orientation : &vec::Quat,
    scale : &vec::Vec3) -> matrix::Matrix4
{
    let mt = matrix::Matrix4::translation(position);
    let mq = matrix::Matrix4::rotation(orientation);
    let ms = matrix::Matrix4::scale(scale);

    &(&mt * &mq) * &ms
}


impl Mul<Orientation> for Orientation {
    type Output = Orientation;
    fn mul(self, other: Orientation) -> Orientation {
        let p = self.as_quat() * other.as_quat();
        //Orientation::Quat(self.as_quat() * other.as_quat())
        match self {
            Orientation::AngleXYZ(_) => 
                Orientation::AngleXYZ(p.to_euler_deg()),
            Orientation::Quat(_) => 
                Orientation::Quat(p)
        }
    }
}

impl Mul<vec::Quat> for Orientation {
    type Output = Orientation;
    fn mul(self, other: vec::Quat) -> Orientation {
        let p = self.as_quat() * other;
        println!("I made a multiplication and : {:?} ", p);
        match self {
            Orientation::AngleXYZ(_) => 
                Orientation::AngleXYZ(p.to_euler_deg()),
            Orientation::Quat(_) => 
                Orientation::Quat(p)
        }
    }
}

impl fmt::Debug for Orientation
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        match *self {
            Orientation::AngleXYZ(a) => 
                //write!(fmt, "({}, {}, {})", a.x, a.y, a.z)
                write!(fmt, "Angles : {:?}", a),
            Orientation::Quat(q) => 
                //Orientation::Quat(p)
                write!(fmt, "Quat : {:?}", q)
        }
    }
}

