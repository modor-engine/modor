// Resolved issues:
//    - Context<'_> should simplify be App (and indicate in doc that update() panics if call from a node)
//        - this simplifies a bit the tests
//    - Visit creates too much complexity and magic
//        - Should be flat singletons (Visit/Node/RootNode become Singleton + FromApp)
//        - Update should be as implicit as possible (e.g. when Model2D position is changed, "immediately" update InstanceGroups)
//    - The engine API seems too verbose for UI
//    - Glob adds complexity and flexibility
//        - Need to sync data between the node and the glob
//        - Glob is not very practical to refer a node in another node -> finally, still used

#[derive(FromApp)] // a type implements FromApp if it implements Default
struct MyGame {
    model: Glob<Model2D>,
    text: Text2D,
}

impl Singleton for MyGame {
    fn update(&mut self, app: &mut App) {
        let old_position = self.model.position(); // good to be able to easily test
        self.model
            .updater()
            .for_position(app, |position| position += Vec2::ONE * 0.01)
            .size(Vec2::ONE)
            .material(app.get_mut::<Resources>().material.to_ref())
            .apply(app);
        self.text.desc().position(Vec2::ONE).apply(app); // internally: self.model.desc().position(...)
    }
}

#[derive(FromApp)]
struct Resources {
    material: MaterialGlob<DefaultMaterial2D>,
}

impl Singleton for Resources {
    fn init(&mut self, app: &mut App) {
        self.material
            .updater()
            .color(Color::Green)
            .is_ellipse(false)
            .apply();
    }
}

#[derive(Updater)]
pub struct Model2D {
    /// See [Model2D::position] for more details.
    #[updater(setter)]
    #[updater(closure)]
    position: Vec2,
    #[updater(setter)]
    #[updater(closure)]
    size: Vec2,
    #[updater(setter)]
    material: GlobRef<Material>,
    #[updater(setter)]
    #[updater(closure)] // error: cannot use closure with updater virtual field
    offset: UpdaterField<Vec2>,
}

impl Model2D {
    fn position(&self) -> Vec2 {
        self.position
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn material(&self) -> &GlobRef<Mat<T>> {
        &self.material
    }
}

impl FromApp for Model2D {
    fn from_app(app: &mut App) -> Self {
        Self {
            position: Vec2::ZERO,
            size: Vec2::ONE,
            material: app.get_mut::<GraphicsResources>().default_material.to_ref(),
        }
    }
}

impl Global for Model2D {
    fn init(&mut self, app: &mut App) -> Self {
        self.position = 0.2;
    }

    fn update(&mut self, app: &mut App) {
        self.position += 0.01 * Vec2::ONE;
    }
}

// automatically implemented with GlobalUpdater derive macro
impl Updater for Model2D {
    type Updater<'a> = Model2DUpdater<'a>;

    fn updater(glob: &Glob<Self>) -> Self::Updater<'_> {
        Model2DUpdater {
            glob,
            position: None,
            size: None,
            material: None,
        }
    }
}

#[must_use] // very important
pub struct Model2DUpdater<'a> {
    glob: &'a Glob<Model2D>,
    position: Option<Vec2>,
    size: Option<Vec2>,
    material: Option<GlobRef<Material>>,
    offset: Option<Vec2>,
}

impl Model2DUpdater<'_> {
    /// Defines the new `position`.
    ///
    /// See [Model2D::position] for more details.
    pub fn position(mut self, position: Vec2) -> Self {
        self.position = Some(position);
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    pub fn material(mut self, material: GlobRef<Material>) -> Self {
        self.material = Some(material);
        self
    }

    /// Defines the new `position` from the current one.
    ///
    /// See [Model2D::position] for more details.
    pub fn for_position<O>(mut self, app: &App, f: impl FnOnce(&mut Vec2) -> O) -> Self {
        f(self
            .position
            .get_or_insert_with(|| self.glob.get(app).position.clone()));
        self
    }

    pub fn for_size<O>(mut self, app: &App, f: impl FnOnce(&mut Vec2) -> O) -> Self {
        f(self
            .position
            .get_or_insert_with(|| self.glob.get(app).size.clone()));
        self
    }

    pub fn position_offset(mut self, offset: Vec2) -> Self {
        self.offset = Some(offset);
        self
    }
}

impl Model2DUpdater<'_> {
    pub fn apply(self, app: &mut App) {
        app.take::<Globals<Model2D>>(|app, models| {
            let model = models.get_mut(self.glob);
            let mut is_updated = false;
            if let Some(position) = self.position {
                is_updated |= position != model.position;
                model.position = position;
            }
            if let Some(size) = self.size {
                is_updated |= size != model.size;
                model.size = size;
            }
            if is_updated {
                app.get_mut::<InstanceGroups>().update(model);
            }
        });
    }
}
