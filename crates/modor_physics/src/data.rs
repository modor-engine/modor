use modor_math::{Point3D, Size3D, Vec3D};
use std::ops::Mul;

pub struct Duration(pub std::time::Duration);

impl From<std::time::Duration> for Duration {
    fn from(duration: std::time::Duration) -> Self {
        Self(duration)
    }
}

impl From<Duration> for std::time::Duration {
    fn from(duration: Duration) -> Self {
        duration.0
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct WorldUnit;

#[derive(Default, Clone, Copy, Debug)]
pub struct WorldUnitPerSecond;

#[derive(Default, Clone, Copy, Debug)]
pub struct WorldUnitPerSecondSquared;

#[derive(Default, Clone, Copy, Debug)]
pub struct RelativeWorldUnit;

#[derive(Default, Clone, Copy, Debug)]
pub struct RelativeWorldUnitPerSecond;

#[derive(Default, Clone, Copy, Debug)]
pub struct RelativeWorldUnitPerSecondSquared;

impl Mul<Duration> for Vec3D<WorldUnitPerSecond> {
    type Output = Vec3D<WorldUnit>;

    fn mul(self, rhs: Duration) -> Self::Output {
        let seconds = std::time::Duration::from(rhs).as_secs_f32();
        Vec3D::xyz(self.x * seconds, self.y * seconds, self.z * seconds)
    }
}

impl Mul<Duration> for Vec3D<WorldUnitPerSecondSquared> {
    type Output = Vec3D<WorldUnitPerSecond>;

    fn mul(self, rhs: Duration) -> Self::Output {
        let seconds = std::time::Duration::from(rhs).as_secs_f32();
        Vec3D::xyz(self.x * seconds, self.y * seconds, self.z * seconds)
    }
}

impl Mul<Duration> for Vec3D<RelativeWorldUnitPerSecond> {
    type Output = Vec3D<RelativeWorldUnit>;

    fn mul(self, rhs: Duration) -> Self::Output {
        let seconds = std::time::Duration::from(rhs).as_secs_f32();
        Vec3D::xyz(self.x * seconds, self.y * seconds, self.z * seconds)
    }
}

impl Mul<Duration> for Vec3D<RelativeWorldUnitPerSecondSquared> {
    type Output = Vec3D<RelativeWorldUnitPerSecond>;

    fn mul(self, rhs: Duration) -> Self::Output {
        let seconds = std::time::Duration::from(rhs).as_secs_f32();
        Vec3D::xyz(self.x * seconds, self.y * seconds, self.z * seconds)
    }
}

pub(crate) struct Scale(Size3D<WorldUnit>);

impl From<Size3D<WorldUnit>> for Scale {
    fn from(duration: Size3D<WorldUnit>) -> Self {
        Self(duration)
    }
}

impl Mul<Scale> for Point3D<RelativeWorldUnit> {
    type Output = Vec3D<WorldUnit>;

    fn mul(self, rhs: Scale) -> Self::Output {
        Vec3D::xyz(self.x * rhs.0.x, self.y * rhs.0.y, self.z * rhs.0.z)
    }
}
