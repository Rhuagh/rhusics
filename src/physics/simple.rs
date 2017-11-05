//! Simple force integration and impulse solver

use std::ops::Mul;

use cgmath::{EuclideanSpace, InnerSpace, Rotation, VectorSpace, Zero};

use super::{ApplyAngular, ForceAccumulator, Inertia, Mass, Velocity};
use {BodyPose, NextFrame, Real};

/// Do force integration for next frame
pub fn next_frame_integration<'a, D, P, A, I, R>(data: D, dt: Real)
where
    D: Iterator<
        Item = (
            &'a mut NextFrame<Velocity<P::Diff, A>>,
            &'a NextFrame<BodyPose<P, R>>,
            &'a mut ForceAccumulator<P::Diff, A>,
            &'a Mass<I>,
        ),
    >,
    P: EuclideanSpace<Scalar = Real> + 'a,
    P::Diff: VectorSpace<Scalar = Real> + InnerSpace + 'a,
    I: Inertia<Orientation = R> + Mul<A, Output = A> + 'a,
    A: Mul<Real, Output = A> + Clone + Copy + Zero + 'a,
    R: Rotation<P> + ApplyAngular<A> + 'a,
{
    // Do force integration
    for (next_velocity, next_pose, force, mass) in data {
        let a = force.consume_force() * mass.inverse_mass();
        let new_velocity = *next_velocity.value.linear() + a * dt;
        next_velocity.value.set_linear(new_velocity);
        let a = mass.world_inverse_inertia(next_pose.value.rotation()) * force.consume_torque();
        let new_velocity = *next_velocity.value.angular() + a * dt;
        next_velocity.value.set_angular(new_velocity);
    }
}

/// Compute next frame pose
pub fn next_frame_pose<'a, D, P, A, R>(data: D, dt: Real)
where
    D: Iterator<
        Item = (
            &'a NextFrame<Velocity<P::Diff, A>>,
            &'a BodyPose<P, R>,
            &'a mut NextFrame<BodyPose<P, R>>,
        ),
    >,
    P: EuclideanSpace<Scalar = Real> + 'a,
    P::Diff: VectorSpace<Scalar = Real> + InnerSpace + 'a,
    A: Mul<Real, Output = A> + Clone + Copy + Zero + 'a,
    R: Rotation<P> + ApplyAngular<A> + 'a,
{
    for (next_velocity, pose, next_pose) in data {
        next_pose.value = next_velocity.value.apply(pose, dt)
    }
}
