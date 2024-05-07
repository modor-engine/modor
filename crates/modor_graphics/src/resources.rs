use crate::mesh::Mesh;
use crate::{Camera2D, DefaultMaterial2D, Mat, Window};
use modor::{Context, Node, RootNode, Visit};

#[non_exhaustive]
#[derive(Debug, Node, Visit)]
pub struct GraphicsResources {
    pub window_camera: Camera2D,
    pub white_material: Mat<DefaultMaterial2D>,
    pub(crate) rectangle_mesh: Mesh,
}

impl RootNode for GraphicsResources {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let window_target = ctx.root::<Window>().get(ctx).target.glob().clone();
        Self {
            window_camera: Camera2D::new(ctx, "main", vec![window_target]),
            white_material: Mat::new(ctx, "white(modor_graphics)", DefaultMaterial2D::default()),
            rectangle_mesh: Mesh::rectangle(ctx),
        }
    }
}
