use cgmath::{Point2, Vector2};
use cgmath::num_traits::Float;
use cgmath::prelude::*;
use collision::prelude::*;

use super::*;
use Real;
use collide::{CollisionStrategy, Contact};

/// EPA algorithm implementation for 2D. Only to be used in [`GJK`](struct.GJK.html).
#[derive(Debug)]
pub struct EPA2;

impl EPA for EPA2 {
    type Point = Point2<Real>;

    fn process<SL, SR, TL, TR>(
        &self,
        simplex: &mut Vec<SupportPoint<Point2<Real>>>,
        left: &SL,
        left_transform: &TL,
        right: &SR,
        right_transform: &TR,
    ) -> Option<Contact<Point2<Real>>>
    where
        SL: SupportFunction<Point = Self::Point>,
        SR: SupportFunction<Point = Self::Point>,
        TL: Transform<Self::Point>,
        TR: Transform<Self::Point>,
    {
        let mut i = 0;
        if closest_edge(&simplex).is_none() {
            return None;
        }

        loop {
            let e = closest_edge(&simplex);
            let e = e.unwrap();
            let p = SupportPoint::from_minkowski(
                left,
                left_transform,
                right,
                right_transform,
                &e.normal,
            );
            let d = p.v.dot(e.normal);
            if d - e.distance < EPA_TOLERANCE {
                return Some(Contact::new_with_point(
                    CollisionStrategy::FullResolution,
                    e.normal,
                    e.distance,
                    point(&simplex, &e),
                ));
            } else {
                simplex.insert(e.index, p);
            }
            i += 1;
            if i >= MAX_ITERATIONS {
                return Some(Contact::new_with_point(
                    CollisionStrategy::FullResolution,
                    e.normal,
                    e.distance,
                    point(&simplex, &e),
                ));
            }
        }
    }

    fn new() -> Self {
        Self {}
    }
}

/// This function returns the contact point in world space coordinates on shape A.
///
/// Compute the closest point to the origin on the given simplex edge, then use that to interpolate
/// the support points coming from the A shape.
fn point(simplex: &Vec<SupportPoint<Point2<Real>>>, edge: &Edge) -> Point2<Real> {
    let b = &simplex[edge.index];
    let a = if edge.index == 0 {
        &simplex[simplex.len() - 1]
    } else {
        &simplex[edge.index - 1]
    };
    let oa = -a.v;
    let ab = b.v - a.v;
    let t = oa.dot(ab) / ab.magnitude2();
    if t < 0. {
        a.sup_a.clone()
    } else if t < 1. {
        b.sup_a.clone()
    } else {
        a.sup_a + (b.sup_a - a.sup_a) * t
    }
}

#[derive(Debug)]
struct Edge {
    pub normal: Vector2<Real>,
    pub distance: Real,
    pub index: usize,
}

impl Edge {
    pub fn new() -> Self {
        Self {
            normal: Vector2::zero(),
            distance: Real::infinity(),
            index: 0,
        }
    }
}

fn closest_edge(simplex: &Vec<SupportPoint<Point2<Real>>>) -> Option<Edge> {
    if simplex.len() < 3 {
        None
    } else {
        let mut edge = Edge::new();
        for i in 0..simplex.len() {
            let j = if i + 1 == simplex.len() { 0 } else { i + 1 };
            let a = simplex[i].v;
            let b = simplex[j].v;
            let e = b - a;
            let oa = a;
            let n = ::util::triple_product(&e, &oa, &e).normalize();
            let d = n.dot(a);
            if d < edge.distance {
                edge.distance = d;
                edge.normal = n;
                edge.index = j;
            }
        }
        Some(edge)
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{Point2, Rad, Rotation2, Vector2};

    use super::*;
    use collide::narrow::gjk::SupportPoint;

    use Real;
    use collide2d::*;

    #[test]
    fn test_closest_edge_0() {
        assert!(closest_edge(&vec![]).is_none())
    }

    #[test]
    fn test_closest_edge_1() {
        assert!(closest_edge(&vec![sup(10., 10.)]).is_none())
    }

    #[test]
    fn test_closest_edge_2() {
        assert!(closest_edge(&vec![sup(10., 10.), sup(-10., 5.)]).is_none())
    }

    #[test]
    fn test_closest_edge_3() {
        let edge = closest_edge(&vec![sup(10., 10.), sup(-10., 5.), sup(2., -5.)]);
        assert!(edge.is_some());
        let edge = edge.unwrap();
        assert_eq!(2, edge.index);
        assert_approx_eq!(2.5607374, edge.distance);
        assert_approx_eq!(-0.6401844, edge.normal.x);
        assert_approx_eq!(-0.7682213, edge.normal.y);
    }

    #[test]
    fn test_epa_0() {
        let left = Rectangle::new(10., 10.);
        let left_transform = transform(15., 0., 0.);
        let right = Rectangle::new(10., 10.);
        let right_transform = transform(7., 2., 0.);
        assert!(
            EPA2.process(
                &mut vec![],
                &left,
                &left_transform,
                &right,
                &right_transform
            ).is_none()
        );
    }

    #[test]
    fn test_epa_1() {
        let left = Rectangle::new(10., 10.);
        let left_transform = transform(15., 0., 0.);
        let right = Rectangle::new(10., 10.);
        let right_transform = transform(7., 2., 0.);
        let mut simplex = vec![sup(-2., 8.)];
        assert!(
            EPA2.process(
                &mut simplex,
                &left,
                &left_transform,
                &right,
                &right_transform
            ).is_none()
        );
    }

    #[test]
    fn test_epa_2() {
        let left = Rectangle::new(10., 10.);
        let left_transform = transform(15., 0., 0.);
        let right = Rectangle::new(10., 10.);
        let right_transform = transform(7., 2., 0.);
        let mut simplex = vec![sup(-2., 8.), sup(18., -12.)];
        assert!(
            EPA2.process(
                &mut simplex,
                &left,
                &left_transform,
                &right,
                &right_transform
            ).is_none()
        );
    }

    #[test]
    fn test_epa_3() {
        let left = Rectangle::new(10., 10.);
        let left_transform = transform(15., 0., 0.);
        let right = Rectangle::new(10., 10.);
        let right_transform = transform(7., 2., 0.);
        let mut simplex = vec![sup(-2., 8.), sup(18., -12.), sup(-2., -12.)];
        let contact = EPA2.process(
            &mut simplex,
            &left,
            &left_transform,
            &right,
            &right_transform,
        );
        assert!(contact.is_some());
        let contact = contact.unwrap();
        assert_eq!(Vector2::new(-1., 0.), contact.normal);
        assert_eq!(2., contact.penetration_depth);
    }

    fn sup(x: Real, y: Real) -> SupportPoint<Point2<Real>> {
        let mut s = SupportPoint::new();
        s.v = Vector2::new(x, y);
        s
    }

    fn transform(x: Real, y: Real, angle: Real) -> BodyPose2 {
        BodyPose2::new(Point2::new(x, y), Rotation2::from_angle(Rad(angle)))
    }
}
