use std::ptr;
use libc::{c_char};
use std::ffi::CString;
use std::ops::{Mul};
use std::fmt;
use std::default::Default;
use std::cell::Cell;

use vec;
use matrix;

#[derive(RustcDecodable, RustcEncodable, Serialize, Deserialize, Clone, Copy)]
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
            scale : vec::Vec3::zero(),
            dirty : false,
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
            let mt = matrix::Matrix4::translation(self.position);
            let mq = matrix::Matrix4::rotation(self.orientation.as_quat());
            let ms = matrix::Matrix4::scale(self.scale);

            self.local_matrix = &(&mt * &mq) * &ms;
            self.dirty = false;
        }
    }

    //TODO for debug
    pub fn set_as_dirty(&mut self)
    {
        self.dirty = true;
    }
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

