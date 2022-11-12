//void rotate_vector_by_quaternion(const Vector3& v, const Quaternion& q, Vector3& vprime)
//{
//    // Extract the vector part of the quaternion
//Vector3 u(q.x, q.y, q.z);
//
//    // Extract the scalar part of the quaternion
//float s = q.w;
//
//    // Do the math
//vprime = 2.0f * dot(u, v) * u
//+ (s*s - dot(u, u)) * v
//+ 2.0f * s * cross(u, v);
//}

use bevy::{
    math::{Quat, Vec3},
    prelude::Transform,
};

pub fn rotate_vec_by_quat<T: Into<Vec3>, U: Into<Quat>>(i_vec: T, i_quat: U) -> Vec3 {
    let vec: Vec3 = i_vec.into();
    let quat: Quat = i_quat.into();
    let vec_q = Vec3::new(quat.x, quat.y, quat.z);
    let s = quat.w;

    2. * vec_q.dot(vec) * vec_q + (s * s - vec_q.dot(vec_q)) * vec + 2. * s * vec_q.cross(vec)
}

pub fn translate_with_local_reference_frame(transform: Transform, translation: Vec3) -> Transform {
    transform.with_translation(
        transform.local_x() * translation.x
            + transform.local_y() * translation.y
            + transform.local_z() * translation.z,
    )
}
