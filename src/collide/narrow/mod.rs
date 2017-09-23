//! Generic narrow phase collision detection algorithms.
//!
//! Currently only supports GJK/EPA.

pub use self::gjk::{GJK2, GJK3, GJK};

use std::fmt::Debug;

use cgmath::EuclideanSpace;

use collide::{CollisionShape, ContactSet, Primitive};

mod gjk;

/// Base trait implemented by all narrow phase algorithms.
///
/// # Type parameters:
///
/// - `ID`: user supplied ID type for the shapes, will be returned as part of any contact manifolds
/// - `P`: collision primitive type
/// - `T`: model-to-world transform type
pub trait NarrowPhase<ID, P, T>: Debug
where
    P: Primitive,
    <P::Point as EuclideanSpace>::Diff: Debug,
{
    /// Check if two shapes collides, and give contact manifolds for any contacts found
    ///
    /// # Parameters:
    ///
    /// - `left`: tuple with the id of the first shape, the shape itself,
    ///           and the current model-to-world transform for the shape
    /// - `right`: tuple with the id of the second shape, the shape itself,
    ///           and the current model-to-world transform for the shape
    ///
    /// # Returns:
    ///
    /// Optionally returns the contact manifolds for any found collisions.
    ///
    fn collide(
        &mut self,
        left: (ID, &CollisionShape<P, T>, &T),
        right: (ID, &CollisionShape<P, T>, &T),
    ) -> Option<ContactSet<ID, P::Point>>;
}
