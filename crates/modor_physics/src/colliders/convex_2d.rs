use approx::AbsDiffEq;
use smallvec::SmallVec;
use modor_math::Vec2;
use crate::Transform;

#[derive(Default)]
pub(crate) struct Convex2DCollider {
    pub(crate) position: Vec2,
    pub(crate) size: Vec2,
    pub(crate) normals: SmallVec<[Vec2; 2]>,
    pub(crate) points: SmallVec<[Vec2; 4]>,
    pub(crate) segments: SmallVec<[(Vec2, Vec2); 4]>,
}

impl Convex2DCollider {
    pub(crate) fn update(&mut self, transform: &Transform) {
        let matrix = transform.create_matrix();
        let point1 = matrix * Vec2::new(-0.5, 0.5);
        let point2 = matrix * Vec2::new(-0.5, -0.5);
        let point3 = matrix * Vec2::new(0.5, -0.5);
        let point4 = matrix * Vec2::new(0.5, 0.5);
        let (x_axis, y_axis) = match (
            self.size.x.abs_diff_eq(&0., f32::EPSILON),
            self.size.y.abs_diff_eq(&0., f32::EPSILON),
        ) {
            (true, true) => (Vec2::X, Vec2::Y),
            (true, false) => ((point1 - point2).perpendicular_cw(), point1 - point2),
            (false, true) => (point3 - point2, (point3 - point2).perpendicular_cw()),
            (false, false) => (point3 - point2, point1 - point2),
        };
        self.position = transform.position.xy();
        self.size = transform.size.xy();
        self.normals.clear();
        self.normals.extend_from_slice(&[x_axis, y_axis]);
        self.points.clear();
        self.points
            .extend_from_slice(&[point1, point2, point3, point4]);
        self.segments.clear();
        self.segments.extend_from_slice(&[
            (point1, point2),
            (point2, point3),
            (point3, point4),
            (point4, point1),
        ]);
    }
}
