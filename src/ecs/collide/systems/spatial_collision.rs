use std::fmt::Debug;

use cgmath::prelude::*;
use collision::dbvt::{DiscreteVisitor, DynamicBoundingVolumeTree, TreeValue};
use collision::prelude::*;
use shrev::EventChannel;
use specs::{Component, Entities, Entity, FetchMut, Join, ReadStorage, System};

use {NextFrame, Real};
use collide::{CollisionShape, CollisionStrategy, ContactEvent, Primitive};
use collide::broad::{BroadPhase, HasBound};
use collide::narrow::NarrowPhase;
use ecs::collide::resources::{Contacts, GetEntity};

/// Collision detection [system](https://docs.rs/specs/0.9.5/specs/trait.System.html) for use with
/// [`specs`](https://docs.rs/specs/0.9.5/specs/).
///
/// Will perform spatial sorting of the collision world.
///
/// Has support for both broad phase and narrow phase collision detection. Will only do narrow phase
/// if both broad and narrow phase is activated. If no broad phase is set, it will use a DBVT based
/// broad phase that has complexity O(m log^2 n), where m is the number of shapes that have a dirty
/// pose.
///
/// Can handle any transform component type, as long as the type implements
/// [`Transform`](https://docs.rs/cgmath/0.15.0/cgmath/trait.Transform.html), and as long as the
/// storage is wrapped in
/// [`FlaggedStorage`](https://docs.rs/specs/0.9.5/specs/struct.FlaggedStorage.html).
///
pub struct SpatialCollisionSystem<P, T, D, Y = ()>
where
    P: Primitive,
    P::Aabb: Aabb<Scalar = Real> + Clone + Debug,
{
    narrow: Option<Box<NarrowPhase<P, T, Y>>>,
    broad: Option<Box<BroadPhase<D>>>,
}

impl<P, T, D, Y> SpatialCollisionSystem<P, T, D, Y>
where
    P: Primitive + Send + Sync + 'static,
    <P::Point as EuclideanSpace>::Diff: Debug,
    P::Aabb: Clone
        + Debug
        + Send
        + Sync
        + 'static
        + Aabb<Scalar = Real>
        + Union<P::Aabb, Output = P::Aabb>
        + Contains<P::Aabb>
        + SurfaceArea<Scalar = Real>,
    T: Transform<P::Point> + Component,
    D: HasBound<Bound = P::Aabb>,
{
    /// Create a new collision detection system, with no broad or narrow phase activated.
    pub fn new() -> Self {
        Self {
            narrow: None,
            broad: None,
        }
    }

    /// Specify what narrow phase algorithm to use
    pub fn with_narrow_phase<N: NarrowPhase<P, T, Y> + 'static>(mut self, narrow: N) -> Self {
        self.narrow = Some(Box::new(narrow));
        self
    }

    /// Specify what broad phase algorithm to use
    pub fn with_broad_phase<B: BroadPhase<D> + 'static>(mut self, broad: B) -> Self {
        self.broad = Some(Box::new(broad));
        self
    }
}

fn discrete_visitor<P, D>(bound: &P::Aabb) -> DiscreteVisitor<P::Aabb, D>
where
    P: Primitive,
    P::Aabb: Aabb<Scalar = Real> + Debug + Discrete<P::Aabb>,
    P::Point: Debug,
    <P::Point as EuclideanSpace>::Diff: Debug,
    D: TreeValue<Bound = P::Aabb>,
{
    DiscreteVisitor::<P::Aabb, D>::new(bound)
}

impl<'a, P, T, Y, D> System<'a> for SpatialCollisionSystem<P, T, (usize, D), Y>
where
    P: Primitive + Send + Sync + 'static,
    P::Aabb: Clone
        + Debug
        + Send
        + Sync
        + 'static
        + Aabb<Scalar = Real>
        + Discrete<P::Aabb>
        + Contains<P::Aabb>
        + SurfaceArea<Scalar = Real>,
    <P::Point as EuclideanSpace>::Diff: Debug + Send + Sync + 'static,
    P::Point: Debug + Send + Sync + 'static,
    T: Component + Clone + Debug + Transform<P::Point> + Send + Sync + 'static,
    Y: Default + Send + Sync + 'static,
    for<'b: 'a> &'b T::Storage: Join<Type = &'b T>,
    D: Send + Sync + 'static + TreeValue<Bound = P::Aabb> + HasBound<Bound = P::Aabb> + GetEntity,
{
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, T>,
        ReadStorage<'a, NextFrame<T>>,
        ReadStorage<'a, CollisionShape<P, T, Y>>,
        Option<FetchMut<'a, Contacts<P::Point>>>,
        Option<FetchMut<'a, EventChannel<ContactEvent<Entity, P::Point>>>>,
        FetchMut<'a, DynamicBoundingVolumeTree<D>>,
    );

    fn run(&mut self, system_data: Self::SystemData) {
        let (entities, poses, next_poses, shapes, mut contacts, mut event_channel, mut tree) =
            system_data;

        if let Some(ref mut c) = contacts {
            c.clear();
        }

        let potentials = if let Some(ref mut broad) = self.broad {
            // Overridden broad phase, use that
            let potentials = broad.find_potentials(tree.values_mut());
            tree.reindex_values();
            potentials
                .iter()
                .map(|&(ref l, ref r)| {
                    (
                        tree.values()[*l].1.entity().clone(),
                        tree.values()[*r].1.entity().clone(),
                    )
                })
                .collect()
        } else {
            // Fallback to DBVT based broad phase
            let mut potentials = Vec::default();
            // find changed values, do intersection tests against tree for each
            // uses FlaggedStorage
            for (entity, _, shape) in (&*entities, (&poses).open().1, &shapes).join() {
                for (v, _) in tree.query(&mut discrete_visitor::<P, D>(shape.bound())) {
                    let e = v.entity();
                    if entity != e {
                        let n = if entity < e {
                            (entity, e.clone())
                        } else {
                            (e.clone(), entity)
                        };
                        if let Err(pos) = potentials.binary_search(&n) {
                            potentials.insert(pos, n);
                        }
                    }
                }
            }
            // find changed next frame values, do intersection tests against tree for each
            // uses FlaggedStorage
            for (entity, _, shape) in (&*entities, (&next_poses).open().1, &shapes).join() {
                for (v, _) in tree.query(&mut discrete_visitor::<P, D>(shape.bound())) {
                    let e = v.entity();
                    if entity != e {
                        let n = if entity < e {
                            (entity, e.clone())
                        } else {
                            (e.clone(), entity)
                        };
                        if let Err(pos) = potentials.binary_search(&n) {
                            potentials.insert(pos, n);
                        }
                    }
                }
            }
            potentials
        };

        match self.narrow {
            Some(ref narrow) => for (left_entity, right_entity) in potentials {
                let left_shape = shapes.get(left_entity).unwrap();
                let right_shape = shapes.get(right_entity).unwrap();
                let left_pose = poses.get(left_entity).unwrap();
                let right_pose = poses.get(right_entity).unwrap();
                let left_next_pose = next_poses.get(left_entity).as_ref().map(|p| &p.value);
                let right_next_pose = next_poses.get(right_entity).as_ref().map(|p| &p.value);
                match narrow.collide_continuous(
                    left_shape,
                    left_pose,
                    left_next_pose,
                    right_shape,
                    right_pose,
                    right_next_pose,
                ) {
                    Some(contact) => {
                        let event =
                            ContactEvent::new((left_entity.clone(), right_entity.clone()), contact);
                        if let Some(ref mut events) = event_channel {
                            events.single_write(event);
                        } else if let Some(ref mut c) = contacts {
                            c.push(event);
                        }
                    }
                    None => (),
                };
            },
            None => {
                // if we only have a broad phase, we generate contacts for aabb
                // intersections
                // right now, we only report the collision, no normal/depth calculation
                for (left_entity, right_entity) in potentials {
                    let event = ContactEvent::new_single(
                        CollisionStrategy::CollisionOnly,
                        (left_entity, right_entity),
                    );
                    if let Some(ref mut events) = event_channel {
                        events.single_write(event);
                    } else if let Some(ref mut c) = contacts {
                        c.push(event);
                    }
                }
            }
        }
    }
}
