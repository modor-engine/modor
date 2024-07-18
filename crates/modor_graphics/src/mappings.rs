use crate::{Camera2DGlob, MaterialGlob};
use derivative::Derivative;
use fxhash::FxHashMap;
use modor::{Context, GlobRef, Globals, Node, RootNode, Visit};

#[derive(Default, RootNode, Visit)]
pub struct Model2DMappings {
    mappings: FxHashMap<SourceModelProps, Vec<DestinationModelProps>>,
    camera_mappings: FxHashMap<usize, Vec<SourceModelProps>>,
    material_mappings: FxHashMap<usize, Vec<SourceModelProps>>,
}

impl Node for Model2DMappings {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        for (index, _) in ctx.get_mut::<Globals<Camera2DGlob>>().deleted_items() {
            for mappings in self.camera_mappings.remove(index).iter().flatten() {
                self.mappings.remove(mappings);
            }
        }
        for (index, _) in ctx.get_mut::<Globals<MaterialGlob>>().deleted_items() {
            for mappings in self.material_mappings.remove(index).iter().flatten() {
                self.mappings.remove(mappings);
            }
        }
    }
}

impl Model2DMappings {
    pub fn register(
        &mut self,
        source_camera: &GlobRef<Camera2DGlob>,
        source_material: &GlobRef<MaterialGlob>,
        dest_camera: &GlobRef<Camera2DGlob>,
        dest_material: &GlobRef<MaterialGlob>,
        ctx: &Context<'_>,
    ) {
        let source = SourceModelProps::new(source_camera, source_material);
        let dest = DestinationModelProps::new(dest_camera, dest_material, ctx);
        self.add_camera(source.camera, source);
        self.add_camera(dest.camera, source);
        self.add_material(source.material, source);
        self.add_material(dest.material, source);
        self.mappings.entry(source).or_default().push(dest);
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceModelProps, DestinationModelProps)> + '_ {
        self.mappings
            .iter()
            .flat_map(|(k, v)| v.iter().map(|v| (*k, *v)))
    }

    pub fn destinations(
        &self,
        source: SourceModelProps,
    ) -> impl Iterator<Item = DestinationModelProps> + '_ {
        self.mappings.get(&source).into_iter().flatten().copied()
    }

    fn add_camera(&mut self, camera: usize, source: SourceModelProps) {
        self.camera_mappings.entry(camera).or_default().push(source);
    }

    fn add_material(&mut self, material: usize, source: SourceModelProps) {
        self.material_mappings
            .entry(material)
            .or_default()
            .push(source);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct SourceModelProps {
    pub camera: usize,
    pub material: usize,
}

impl SourceModelProps {
    pub fn new(camera: &GlobRef<Camera2DGlob>, material: &GlobRef<MaterialGlob>) -> Self {
        Self {
            camera: camera.index(),
            material: material.index(),
        }
    }
}

#[derive(Derivative, Debug, Clone, Copy)]
#[derivative(PartialEq, Eq, Hash)]
pub struct DestinationModelProps {
    pub camera: usize,
    pub material: usize,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub(crate) instance_data_size: usize,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub(crate) generate_instance_data: fn(&mut Context<'_>, usize) -> Vec<u8>,
}

impl DestinationModelProps {
    fn new(
        camera: &GlobRef<Camera2DGlob>,
        material: &GlobRef<MaterialGlob>,
        ctx: &Context<'_>,
    ) -> Self {
        Self {
            camera: camera.index(),
            material: material.index(),
            instance_data_size: material.get(ctx).instance_data_size,
            generate_instance_data: |_, _| todo!(),
        }
    }
}
