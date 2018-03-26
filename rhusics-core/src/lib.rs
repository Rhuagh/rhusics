//! # Rhusics physics library
//!
//! A physics library.
//! Uses [`cgmath`](https://github.com/brendanzab/cgmath/) for all computation.
//!
//! Features:
//!
//! * Two different broad phase collision detection implementations:
//!   * Brute force
//!   * Sweep and Prune
//! * Narrow phase collision detection using GJK, and optionally EPA for full contact information
//! * Functions for collision detection working on user supplied transform, and
//!    [`CollisionShape`](collide/struct.CollisionShape.html) components.
//!    Can optionally use broad and/or narrow phase detection.
//!    Library supplies a transform implementation [`BodyPose`](struct.BodyPose.html) for
//!    convenience.
//! * Uses single precision as default, can be changed to double precision with the `double`
//!   feature.
//! * Has support for doing spatial sort/collision detection using the collision-rs DBVT.
//! * Support for doing broad phase using the collision-rs DBVT.
//! * Has support for all primitives in collision-rs
//!

#![deny(missing_docs, trivial_casts, unsafe_code, unstable_features, unused_import_braces,
        unused_qualifications)]
#![allow(unknown_lints, type_complexity, borrowed_box)]

extern crate cgmath;
extern crate collision;

#[cfg(feature = "specs")]
extern crate specs;

#[cfg(test)]
#[macro_use]
extern crate approx;

#[cfg(feature = "eders")]
#[macro_use]
extern crate serde;

pub use collide::{basic_collide, tree_collide, Collider, CollisionData, CollisionMode,
                  CollisionShape, CollisionStrategy, Contact, ContactEvent, GetId, Primitive};
pub use collide::broad::{BroadPhase, BruteForce, SweepAndPrune2, SweepAndPrune3};
pub use collide::narrow::NarrowPhase;
pub use physics::{resolve_contact, ApplyAngular, ForceAccumulator, Inertia, Mass, Material,
                  PartialCrossProduct, ResolveData, RigidBody, SingleChangeSet, Velocity, Volume};
pub use physics::simple::{next_frame_integration, next_frame_pose};

pub mod collide2d;
pub mod collide3d;
pub mod physics2d;
pub mod physics3d;

use cgmath::BaseFloat;
use cgmath::prelude::*;
use collision::prelude::*;

mod collide;
mod physics;
#[cfg(feature = "specs")]
mod ecs;

/// Wrapper for data computed for the next frame
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "eders", derive(Serialize, Deserialize))]
pub struct NextFrame<T> {
    /// Wrapped value
    pub value: T,
}

/// Transform component used throughout the library
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "eders", derive(Serialize, Deserialize))]
pub struct BodyPose<P, R> {
    dirty: bool,
    position: P,
    rotation: R,
    inverse_rotation: R,
}

impl<P, R> BodyPose<P, R>
where
    P: EuclideanSpace,
    P::Scalar: BaseFloat,
    R: Rotation<P>,
{
    /// Create a new [`BodyPose`](struct.BodyPose.html) with initial state given by the supplied
    /// position and rotation.
    pub fn new(position: P, rotation: R) -> Self {
        Self {
            dirty: true,
            position,
            inverse_rotation: rotation.invert(),
            rotation,
        }
    }

    /// Set the rotation. Will also compute the inverse rotation. Sets the dirty flag.
    pub fn set_rotation(&mut self, rotation: R) {
        self.rotation = rotation;
        self.inverse_rotation = self.rotation.invert();
        self.dirty = true;
    }

    /// Set the position. Sets the dirty flag.
    pub fn set_position(&mut self, position: P) {
        self.position = position;
        self.dirty = true;
    }

    /// Borrows the position attribute
    pub fn position(&self) -> &P {
        &self.position
    }

    /// Borrows the rotation attribute
    pub fn rotation(&self) -> &R {
        &self.rotation
    }

    /// Clear the dirty flag
    pub fn clear(&mut self) {
        self.dirty = false;
    }
}

impl<P, R> Transform<P> for BodyPose<P, R>
where
    P: EuclideanSpace,
    P::Scalar: BaseFloat,
    R: Rotation<P>,
{
    fn one() -> Self {
        Self::new(P::origin(), R::one())
    }

    fn look_at(eye: P, center: P, up: P::Diff) -> Self {
        let rot = R::look_at(center - eye, up);
        let disp = rot.rotate_vector(P::origin() - eye);
        Self::new(P::from_vec(disp), rot)
    }

    fn transform_vector(&self, vec: P::Diff) -> P::Diff {
        self.rotation.rotate_vector(vec)
    }

    fn transform_point(&self, point: P) -> P {
        self.rotation.rotate_point(point) + self.position.to_vec()
    }

    fn concat(&self, other: &Self) -> Self {
        Self::new(
            self.position + self.rotation.rotate_point(other.position).to_vec(),
            self.rotation * other.rotation,
        )
    }

    fn inverse_transform(&self) -> Option<Self> {
        Some(Self::new(
            self.rotation.rotate_point(self.position) * -P::Scalar::one(),
            self.inverse_rotation,
        ))
    }

    fn inverse_transform_vector(&self, vec: P::Diff) -> Option<P::Diff> {
        Some(self.inverse_rotation.rotate_vector(vec))
    }
}

impl<P, R> TranslationInterpolate<P::Scalar> for BodyPose<P, R>
where
    P: EuclideanSpace,
    P::Scalar: BaseFloat,
    P::Diff: VectorSpace + InnerSpace,
    R: Rotation<P> + Clone,
{
    fn translation_interpolate(&self, other: &Self, amount: P::Scalar) -> Self {
        BodyPose::new(
            P::from_vec(self.position.to_vec().lerp(other.position.to_vec(), amount)),
            other.rotation,
        )
    }
}

impl<P, R> Interpolate<P::Scalar> for BodyPose<P, R>
where
    P: EuclideanSpace,
    P::Scalar: BaseFloat,
    P::Diff: VectorSpace + InnerSpace,
    R: Rotation<P> + Interpolate<P::Scalar>,
{
    fn interpolate(&self, other: &Self, amount: P::Scalar) -> Self {
        BodyPose::new(
            P::from_vec(self.position.to_vec().lerp(other.position.to_vec(), amount)),
            self.rotation.interpolate(&other.rotation, amount),
        )
    }
}
