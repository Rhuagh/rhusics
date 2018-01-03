//! 3D physics ECS

pub use ecs::collide::prelude3d::*;
pub use ecs::physics::{DeltaTime, WithLazyRigidBody, WithRigidBody};
pub use physics::prelude3d::*;

use cgmath::{BaseFloat, Matrix3, Point3, Quaternion, Vector3};
use collision::Aabb3;
use collision::dbvt::TreeValueWrapped;
use collision::primitive::Primitive3;
use specs::{Entity, World};

use ecs::WithRhusics;
use ecs::physics::{ContactResolutionSystem, CurrentFrameUpdateSystem, NextFrameSetupSystem};

/// Current frame integrator system for 2D
pub type CurrentFrameUpdateSystem3<S> =
    CurrentFrameUpdateSystem<Point3<S>, Quaternion<S>, Vector3<S>>;

/// Resolution system for 2D
pub type ContactResolutionSystem3<S> =
    ContactResolutionSystem<Point3<S>, Quaternion<S>, Matrix3<S>, Vector3<S>, Vector3<S>>;

/// Next frame setup system for 2D
pub type NextFrameSetupSystem3<S> =
    NextFrameSetupSystem<Point3<S>, Quaternion<S>, Matrix3<S>, Vector3<S>>;

/// Utility method for registering 3D physics and collision components and resources with
/// [`specs::World`](https://docs.rs/specs/0.9.5/specs/struct.World.html).
///
/// # Parameters
///
/// - `world`: The [world](https://docs.rs/specs/0.9.5/specs/struct.World.html)
/// to register components/resources in.
///
/// # Type parameters
///
/// - `Y`: Collision shape type, see `Collider`
pub fn register_physics<S, Y>(world: &mut World)
where
    S: BaseFloat + Send + Sync + 'static,
    Y: Collider + Default + Send + Sync + 'static,
{
    world.register_physics::<
        Primitive3<S>,
        Aabb3<S>,
        Quaternion<S>,
        TreeValueWrapped<Entity, Aabb3<S>>,
        Y,
        Vector3<S>,
        Vector3<S>,
        Matrix3<S>
    >();
}
