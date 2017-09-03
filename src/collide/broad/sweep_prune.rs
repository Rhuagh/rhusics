use std::clone::Clone;
use std::cmp::Ordering;
use std::fmt::Debug;

use self::variance::Variance;
use Real;
use collide::broad::*;

#[derive(Debug)]
pub struct SweepAndPrune<V> {
    sweep_axis: usize,
    variance: V,
}

impl<V> SweepAndPrune<V>
where
    V: Variance,
{
    pub fn new() -> Self {
        Self::new_impl(0)
    }

    pub fn new_impl(sweep_axis: usize) -> Self {
        Self {
            sweep_axis,
            variance: V::new(),
        }
    }
}

impl<ID, A, V> BroadPhase<ID, A> for SweepAndPrune<V>
where
    ID: Clone + Debug,
    A: Aabb<Scalar = Real> + Discrete<A> + Debug,
    A::Point: EuclideanSpace,
    A::Diff: VectorSpace + ElementWise,
    V: Variance<Point = A::Point>,
{
    fn compute(&mut self, shapes: &mut Vec<BroadCollisionInfo<ID, A>>) -> Vec<(ID, ID)> {
        let mut pairs = Vec::<(ID, ID)>::default();
        if shapes.len() <= 1 {
            return pairs;
        }

        debug!("Starting sweep and prune");
        debug!("Sweep axis is {}", self.sweep_axis);
        shapes.sort_by(|a, b| if a.bound.min()[self.sweep_axis] !=
            b.bound.min()[self.sweep_axis]
        {
            a.bound.min()[self.sweep_axis]
                .partial_cmp(&b.bound.min()[self.sweep_axis])
                .unwrap_or(Ordering::Equal)
        } else {
            a.bound.max()[self.sweep_axis]
                .partial_cmp(&b.bound.max()[self.sweep_axis])
                .unwrap_or(Ordering::Equal)
        });
        debug!("Sorted vector {:?}", shapes);

        let mut active_index = 0;

        self.variance.clear();
        self.variance.add_to_sum(
            &shapes[active_index].bound.min(),
            &shapes[active_index].bound.max(),
        );

        // FIXME: very large shapes will cause this algorithm to be O(n^2), needs to be fixed.
        debug!("starting checks");
        for index in 1..shapes.len() {
            debug!("before advance, active: {}, index: {}", active_index, index);
            // advance active_index until it could be intersecting

            while shapes[active_index].bound.max()[self.sweep_axis] <
                shapes[index].bound.min()[self.sweep_axis] &&
                active_index < index
            {
                active_index += 1;
            }
            debug!("after advance, active: {}, index: {}", active_index, index);
            if index > active_index {
                for left_index in active_index..index {
                    if shapes[left_index].bound.intersects(&shapes[index].bound) {
                        pairs.push((shapes[left_index].id.clone(), shapes[index].id.clone()));
                    }
                }
            }
            self.variance.add_to_sum(
                &shapes[index].bound.min(),
                &shapes[index].bound.max(),
            );
        }

        let (axis, _) = self.variance.compute_axis(shapes.len() as Real);
        self.sweep_axis = axis;

        pairs
    }
}

pub mod variance {
    use cgmath::{Vector2, Point2, Point3, Vector3};
    use cgmath::prelude::*;

    use Real;

    pub trait Variance {
        type Point: EuclideanSpace<Scalar = Real>;

        fn new() -> Self;
        fn clear(&mut self);
        fn add_to_sum(&mut self, min: &Self::Point, max: &Self::Point);
        fn compute_axis(&self, n: Real) -> (usize, Real);
    }

    pub struct Variance2D {
        csum: Vector2<Real>,
        csumsq: Vector2<Real>,
    }

    impl Variance for Variance2D {
        type Point = Point2<Real>;

        fn new() -> Self {
            Self {
                csum: Vector2::zero(),
                csumsq: Vector2::zero(),
            }
        }

        fn clear(&mut self) {
            self.csum = Vector2::zero();
            self.csumsq = Vector2::zero();
        }

        #[inline]
        fn add_to_sum(&mut self, min: &Point2<Real>, max: &Point2<Real>) {
            let min_vec = min.to_vec();
            let max_vec = max.to_vec();
            let sum = min_vec.add_element_wise(max_vec);
            let c = sum / 2.;
            self.csum.add_element_wise(c);
            self.csumsq.add_element_wise(c.mul_element_wise(c));
        }

        #[inline]
        fn compute_axis(&self, n: Real) -> (usize, Real) {
            let square_n = self.csum.mul_element_wise(self.csum) / n;
            let variance = self.csumsq.sub_element_wise(square_n);
            let mut sweep_axis = 0;
            let mut sweep_variance = variance[0];
            for i in 1..2 {
                let v = variance[i];
                if v > sweep_variance {
                    sweep_axis = i;
                    sweep_variance = v;
                }
            }
            (sweep_axis, sweep_variance)
        }
    }

    pub struct Variance3D {
        csum: Vector3<Real>,
        csumsq: Vector3<Real>,
    }

    impl Variance for Variance3D {
        type Point = Point3<Real>;

        fn new() -> Self {
            Self {
                csum: Vector3::zero(),
                csumsq: Vector3::zero(),
            }
        }

        fn clear(&mut self) {
            self.csum = Vector3::zero();
            self.csumsq = Vector3::zero();
        }

        #[inline]
        fn add_to_sum(&mut self, min: &Point3<Real>, max: &Point3<Real>) {
            let min_vec = min.to_vec();
            let max_vec = max.to_vec();
            let sum = min_vec.add_element_wise(max_vec);
            let c = sum / 2.;
            self.csum.add_element_wise(c);
            self.csumsq.add_element_wise(c.mul_element_wise(c));
        }

        #[inline]
        fn compute_axis(&self, n: Real) -> (usize, Real) {
            let square_n = self.csum.mul_element_wise(self.csum) / n;
            let variance = self.csumsq.sub_element_wise(square_n);
            let mut sweep_axis = 0;
            let mut sweep_variance = variance[0];
            for i in 1..3 {
                let v = variance[i];
                if v > sweep_variance {
                    sweep_axis = i;
                    sweep_variance = v;
                }
            }
            (sweep_axis, sweep_variance)
        }
    }
}

#[cfg(test)]
mod tests {
    use cgmath::Point2;
    use collision::Aabb2;

    use super::*;
    use Real;
    use collide2d::{BroadCollisionInfo2D, SweepAndPrune2D};

    #[test]
    fn no_intersection_for_miss() {
        let left = coll(1, 8., 8., 10., 11.);

        let right = coll(2, 12., 13., 18., 18.);

        let mut sweep = SweepAndPrune2D::new();
        let potentials = sweep.compute(&mut vec![left, right]);
        assert_eq!(0, potentials.len());
    }

    #[test]
    fn no_intersection_for_miss_unsorted() {
        let left = coll(1, 8., 8., 10., 11.);

        let right = coll(2, 12., 13., 18., 18.);

        let mut sweep = SweepAndPrune2D::new();
        let potentials = sweep.compute(&mut vec![right, left]);
        assert_eq!(0, potentials.len());
    }

    #[test]
    fn intersection_for_hit() {
        let left = coll(1, 8., 8., 10., 11.);

        let right = coll(2, 9., 10., 18., 18.);

        let mut sweep = SweepAndPrune2D::new();
        let potentials = sweep.compute(&mut vec![left, right]);
        assert_eq!(1, potentials.len());
        assert_eq!((1, 2), potentials[0]);
    }

    #[test]
    fn intersection_for_hit_unsorted() {
        let left = coll(1, 8., 8., 10., 11.);

        let right = coll(222, 9., 10., 18., 18.);

        let mut sweep = SweepAndPrune2D::new();
        let potentials = sweep.compute(&mut vec![right, left]);
        assert_eq!(1, potentials.len());
        assert_eq!((1, 222), potentials[0]);
    }

    // util
    fn coll(
        id: u32,
        min_x: Real,
        min_y: Real,
        max_x: Real,
        max_y: Real,
    ) -> BroadCollisionInfo2D<u32> {
        BroadCollisionInfo2D::new(id, bound(min_x, min_y, max_x, max_y))
    }

    fn bound(min_x: Real, min_y: Real, max_x: Real, max_y: Real) -> Aabb2<Real> {
        Aabb2::new(Point2::new(min_x, min_y), Point2::new(max_x, max_y))
    }
}