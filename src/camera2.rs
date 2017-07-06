use std::f64::consts;
use std::default::Default;
use serde_derive;

use vec;
use vec::{Vec3};
use matrix;
use geometry;
use transform::{Transform,Orientation};
use uuid;
use camera;

//extern crate serde_json;

#[derive(Clone,Serialize,Deserialize, Debug)]
pub enum Projection
{
    Perspective,
    Orthographic
}

impl Projection {
    fn from_old_camera_data(p : &camera::Projection) -> Projection
    {
        match *p {
            camera::Projection::Perspective => Projection::Perspective,
            camera::Projection::Orthographic => Projection::Orthographic,
        }
    }
}

#[derive(Clone,Serialize,Deserialize,Debug)]
pub struct Camera
{
    fovy : f64,
    pub fovy_base : f64,
    pub near : f64,
    pub far : f64,
    pub aspect : f64,
    pub width : f64,
    pub height : f64,
    pub height_base : i32,
    //pub yaw : f64,
    //pub pitch : f64,
    //pub roll : f64,

    //pub clear_color : vec::Vec4,
    pub projection : Projection,
}

impl Default for Camera
{
    fn default() -> Camera
    {
        Camera {
            fovy : consts::PI/8.0f64,
            fovy_base : consts::PI/8.0f64,
            near : 1f64,
            far : 10000f64,
            aspect : 1.6f64,
            width : 800f64,
            height : 500f64,
            height_base : 500i32,

            //clear_color : vec::Vec4::zero(),
            projection : Projection::Perspective,
        }
    }
}

impl Camera
{
    pub fn get_perspective(&self) -> matrix::Matrix4
    {
        match self.projection {
            Projection::Perspective =>
                matrix::Matrix4::perspective(
                    self.fovy,
                    self.aspect,
                    self.near,
                    self.far),
            Projection::Orthographic => 
                matrix::Matrix4::orthographic(
                    (self.width / 2f64) as u32,
                    (self.height / 2f64) as u32,
                    self.near,
                    self.far)
        }
    }

    pub fn ray_from_screen(
        &self,
        camera_transform : &Transform,
        x : f64,
        y : f64,
        length: f64) -> geometry::Ray
    {
        let near = self.near;
        let camz = camera_transform.orientation.rotate_vec3(&Vec3::forward());
        let up = camera_transform.orientation.rotate_vec3(&Vec3::up());
        let h = (camz^up).normalized();
        let vl = (self.fovy/2f64).tan() * near;

        let width = self.width;
        let height = self.height;
        let aspect : f64 = width / height;
        let vh = vl * aspect;

        let up = up * vl;
        let h = h * vh;

        let x : f64 = x - (width /2.0f64);
        let y : f64 = y - (height /2.0f64);

        let x : f64 = x / (width /2.0f64);
        let y : f64 = y / (height /2.0f64);


        let pos = camera_transform.position + (camz * near) + ( (h * x) + (up * -y));
        let dir = pos - camera_transform.position;
        let dir = dir.normalized();
        let dir = dir * length;

        geometry::Ray {
            start : pos,
            direction : dir
        }
    }

    pub fn set_resolution(&mut self, w : i32, h : i32)
    {
        if w as f64 != self.width || h as f64 != self.height {
            self.width = w as f64;
            self.height = h as f64;
            self.update_projection();
            //cam.update_orthographic(c);
        }
    }

    pub fn update_projection(&mut self)
    {
        self.aspect = self.width/ self.height;
        self.fovy = self.fovy_base * self.height/ (self.height_base as f64);
        //mat4_set_perspective(c->projection, c->fovy, c->aspect , c->near, c->far);
    }

    pub fn get_frustum_planes_rect(
        &self,
        camera_transform : &Transform,
        left : f64, top : f64, width : f64, height : f64) -> [geometry::Plane;6]
    {
        let ori = &camera_transform.orientation;
        let pos = &camera_transform.position;

        let direction = ori.rotate_vec3(&vec::Vec3::new(0f64,0f64,-1f64));
        let right = ori.rotate_vec3(&vec::Vec3::new(1f64,0f64,0f64));
        let up = ori.rotate_vec3(&vec::Vec3::new(0f64,1f64,0f64));

        //plane order:
        //near, far, up, down, right, left
        let mut p = [geometry::Plane::xz();6];

        //near plane
        p[0].point = *pos + (direction * self.near);
        p[0].normal = direction;

        //far plane
        p[1].point = *pos + (direction * self.far);
        p[1].normal = direction * -1f64;

        //up plane
        let hh = (self.fovy/2f64).tan()* self.near;
        let top = top * hh / (self.height/2.0f64);
        let height = height * hh / (self.height/2.0f64);

        let th = hh - top;
        let upd = (direction * self.near) + (up * th);

        p[2].point = *pos;
        let nn = (right ^ upd).normalized();
        p[2].normal = nn * -1f64;

        //down plane
        let bh = hh - (top + height);
        p[3].point = *pos;
        let downd = (direction * self.near) + (up * bh);
        let nn = (right ^ downd).normalized();
        //p[3].normal = vec3_mul(nn, -1);
        p[3].normal = nn;


        //right plane
        let hw = hh * self.aspect;
        let left = left * hw / (self.width/2.0f64);
        let width = width * hw / (self.width/2.0f64);
        
        let rw = -hw + (left + width);
        p[4].point = *pos;
        let rightd = (direction * self.near) + (right* rw);
        let nn = (up ^ rightd).normalized();
        //p[4].normal = vec3_mul(nn, -1);
        p[4].normal = nn;

        //left plane
        let lw = -hw + left;
        p[5].point = *pos;
        let leftd = (direction* self.near) + (right* lw);
        let nn = (up ^ leftd).normalized();
        p[5].normal = nn * -1f64;

        /*
           printf(" leftd : %f, %f, %f \n", leftd.x, leftd.y, leftd.z);
           printf(" up : %f, %f, %f \n", up.x, up.y, up.z);
           printf(" up plane normal : %f, %f, %f \n", nn.x, nn.y, nn.z);
           */
        return p;
    }

    pub fn world_to_screen(
        &self,
        camera_world_matrix : &matrix::Matrix4,
        p : vec::Vec3) -> vec::Vec2
    {
        let cam_inv = camera_world_matrix.get_inverse();
        let projection = self.get_perspective();

        let tm = &projection * &cam_inv;

        let p4 = vec::Vec4::new(p.x, p.y, p.z, 1f64);
        let sp = &tm * p4;

        let n  = vec::Vec3::new(sp.x/sp.w, sp.y/sp.w, sp.z/sp.w);

        let screen  = vec::Vec2::new(
            (n.x+1.0f64)* self.width/2.0f64,
            -(n.y-1.0f64)* self.height/2.0f64);

        //printf("screen : %f, %f \n", screen.x, screen.y);

        return screen;
    }

    //TODO remove
    pub fn from_old_camera_data(data : &camera::CameraData) -> Camera
    {
        Camera {
            fovy : data.fovy,
            fovy_base : data.fovy_base,
            near : data.near,
            far : data.far,
            aspect : data.aspect,
            width : data.width,
            height : data.height,
            height_base : data.height_base,

            //clear_color : vec::Vec4::zero(),
            projection : Projection::from_old_camera_data(&data.projection)
        }
    }

}

pub struct CameraTransform<'a>
{
    pub camera : &'a Camera,
    pub transform : &'a Transform
}

impl<'a> CameraTransform<'a>
{
    pub fn new(transform : &'a Transform, cam : &'a Camera) -> CameraTransform<'a>
    {
        CameraTransform {
            camera : cam,
            transform : transform
        }
    }

    pub fn ray_from_screen(
        &self,
        x : f64,
        y : f64,
        length: f64) -> geometry::Ray
    {
        self.camera.ray_from_screen(self.transform, x, y, length)
    }

    pub fn world_to_screen(
        &self,
        p : vec::Vec3) -> vec::Vec2
    {
        self.camera.world_to_screen(self.transform.get_computed_local_matrix(), p)
    }

    pub fn get_frustum_planes_rect(
        &self,
        left : f64, top : f64, width : f64, height : f64) -> [geometry::Plane;6]
    {
        self.camera.get_frustum_planes_rect(self.transform, left, top, width, height)
    }

}
