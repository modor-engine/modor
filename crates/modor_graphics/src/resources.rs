use crate::mesh::Mesh;
use crate::{Camera2D, Window};
use modor::{Context, Node, RootNode, Visit};

#[non_exhaustive]
#[derive(Debug, Node, Visit)]
pub struct GraphicsResources {
    pub window_camera: Camera2D,
    pub(crate) rectangle_mesh: Mesh,
}

impl RootNode for GraphicsResources {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let window_target = ctx.get_mut::<Window>().target.glob().clone();
        Self {
            window_camera: Camera2D::new(ctx, "window(modor_graphics)", vec![window_target]),
            rectangle_mesh: Mesh::rectangle(ctx),
        }
    }
}
