use std::fmt::Debug;

use cgmath::prelude::*;
use collision::{Aabb, Primitive};
use specs::{Component, DenseVecStorage, Entity, FlaggedStorage};

use {BodyPose, NextFrame, Real};
use collide::CollisionShape;
use collide::util::ContainerShapeWrapper;

impl<P, R> Component for BodyPose<P, R>
where
    P: EuclideanSpace<Scalar = Real> + Send + Sync + 'static,
    R: Rotation<P> + Send + Sync + 'static,
{
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl<T> Component for NextFrame<T>
where
    T: Send + Sync + 'static,
{
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

/// Retrieve the entity for the given object
pub trait GetEntity {
    /// Return the entity
    fn entity(&self) -> Entity;
}

impl<P, T, Y> Component for CollisionShape<P, T, Y>
where
    T: Send + Sync + 'static,
    Y: Send + Sync + 'static,
    P: Primitive + Send + Sync + 'static,
    P::Aabb: Send + Sync + 'static,
{
    type Storage = DenseVecStorage<CollisionShape<P, T, Y>>;
}

impl<'a, P, T, Y> From<(Entity, &'a CollisionShape<P, T, Y>)> for ContainerShapeWrapper<Entity, P>
where
    P: Primitive,
    P::Aabb: Aabb<Scalar = Real>,
    <P::Point as EuclideanSpace>::Diff: Debug,
    T: Transform<P::Point>,
    Y: Default,
{
    fn from((entity, ref shape): (Entity, &CollisionShape<P, T, Y>)) -> Self {
        Self::new(entity, shape.bound())
    }
}

impl<P> GetEntity for ContainerShapeWrapper<Entity, P>
where
    P: Primitive,
    P::Aabb: Aabb<Scalar = Real>,
    <P::Point as EuclideanSpace>::Diff: Debug,
{
    fn entity(&self) -> Entity {
        self.id
    }
}
