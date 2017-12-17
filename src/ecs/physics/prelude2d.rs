//! 2D physics ECS

pub use ecs::collide::prelude2d::*;
pub use ecs::physics::DeltaTime;
pub use physics::prelude2d::*;

use cgmath::{Basis2, Point2};
use shrev::EventChannel;
use specs::World;

use {NextFrame, Real};
use ecs::physics::LinearContactSolverSystem;

/// Linear contact resolve system for 2D
pub type LinearContactSolverSystem2 = LinearContactSolverSystem<Point2<Real>, Basis2<Real>>;

/// Register required components and resources in world
pub fn world_physics_register<Y>(world: &mut World)
where
    Y: Default + Send + Sync + 'static,
{
    world.add_resource(DeltaTime { delta_seconds: 0. });
    world.register::<Mass>();
    world.register::<Velocity2>();
    world.register::<NextFrame<Velocity2>>();
    world.add_resource(EventChannel::<ContactEvent2>::new());
    world_register::<BodyPose2, Y>(world);
}

/// Register required components and resources in world
pub fn world_physics_register_with_spatial<Y>(world: &mut World)
where
    Y: Default + Send + Sync + 'static,
{
    world.add_resource(DeltaTime { delta_seconds: 0. });
    world.register::<Mass>();
    world.register::<Velocity2>();
    world.register::<NextFrame<Velocity2>>();
    world.add_resource(EventChannel::<ContactEvent2>::new());
    world_register_with_spatial::<BodyPose2, Y>(world);
}
