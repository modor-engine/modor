use crate::shader::loaded::ShaderLoaded;
use crate::{Material, ShaderGlobInner};
use derivative::Derivative;
use getset::CopyGetters;
use log::error;
use modor::{App, FromApp, Glob, GlobRef, Update, Updater};
use modor_resources::{Res, ResSource, ResUpdater, Resource, ResourceError, Source};
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;

pub(crate) mod glob;
mod loaded;

/// A [`Shader`] glob.
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct ShaderGlob<T: Material> {
    inner: Glob<Res<Shader>>,
    phantom: PhantomData<fn(T)>,
}

impl<T> FromApp for ShaderGlob<T>
where
    T: Material,
{
    fn from_app(app: &mut App) -> Self {
        Self {
            inner: Glob::<Res<Shader>>::from_app_with(app, |res, app| {
                res.get_mut(app).instance_size = mem::size_of::<T::InstanceData>();
            }),
            phantom: PhantomData,
        }
    }
}

impl<T> Deref for ShaderGlob<T>
where
    T: Material,
{
    type Target = Glob<Res<Shader>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> ShaderGlob<T>
where
    T: Material,
{
    /// Returns static reference to the glob.
    pub fn to_ref(&self) -> ShaderGlobRef<T> {
        ShaderGlobRef {
            inner: self.inner.to_ref(),
            phantom: PhantomData,
        }
    }
}

/// A [`Shader`] glob reference.
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = ""),
    Clone(bound = "")
)]
pub struct ShaderGlobRef<T> {
    inner: GlobRef<Res<Shader>>,
    phantom: PhantomData<T>,
}

impl<T> Deref for ShaderGlobRef<T>
where
    T: Material,
{
    type Target = GlobRef<Res<Shader>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A shader that defines a rendering logic.
///
/// # Supported languages
///
/// This component only supports code in [WGSL](https://www.w3.org/TR/WGSL/) format.
///
/// # Input locations
///
/// The code can include the following locations:
/// - location `0`: vertex position.
/// - location `1`: texture position for the vertex.
/// - location `2`: column 1 of the instance transform matrix.
/// - location `3`: column 2 of the instance transform matrix.
/// - location `4`: column 3 of the instance transform matrix.
/// - location `5`: column 4 of the instance transform matrix.
/// - location `6` or more: material data per instance. These locations must be defined
///     in a struct named `MaterialInstance` which corresponds to
///     [`Material::InstanceData`] on Rust side.
///
/// # Bindings
///
/// The code can include the following bindings:
/// - group `0`
///     - binding `0`: camera data
/// - group `1`
///     - binding `0`: material data (`Material` struct corresponds to
///         [`Material::Data`] on Rust side)
///     - binding `(i * 2)`: `texture_2d<f32>` value corresponding to texture `i`
///     - binding `(i * 2 + 1)`: `sampler` value corresponding to texture `i`
///
/// # Examples
///
/// See [`Material`].
#[derive(Debug, FromApp, Updater, CopyGetters)]
pub struct Shader {
    /// Controls how alpha channel should be treated:
    /// - `false`: apply standard alpha blending with non-premultiplied alpha.
    ///     It means models rendered behind a transparent model might be visible.
    /// - `true`: don't apply any color blending, just overwrites the output color.
    ///     It means models rendered behind a transparent model will never be visible.
    ///
    /// Default is `false`.
    #[getset(get_copy = "pub")]
    #[updater(field, for_field)]
    is_alpha_replaced: bool,
    #[updater(inner_type, field)]
    res: PhantomData<ResUpdater<Shader>>,
    instance_size: usize,
    source: ResSource<Self>,
    loaded: ShaderLoaded,
    pub(crate) glob: ShaderGlobInner,
    is_invalid: bool,
}

impl Resource for Shader {
    type Source = ShaderSource;
    type Loaded = ShaderLoaded;

    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError> {
        let code =
            String::from_utf8(file_bytes).map_err(|err| ResourceError::Other(format!("{err}")))?;
        ShaderLoaded::new(code)
    }

    fn load_from_source(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
        ShaderLoaded::new(match source {
            ShaderSource::String(string) => string.clone(),
        })
    }

    fn on_load(&mut self, app: &mut App, loaded: Self::Loaded, source: &ResSource<Self>) {
        self.loaded = loaded;
        self.source = source.clone();
        self.update(app);
    }
}

impl Shader {
    /// Whether an error occurred during parsing of the shader code.
    pub fn is_invalid(&self) -> bool {
        self.is_invalid
    }

    fn update(&mut self, app: &mut App) {
        match ShaderGlobInner::new(
            app,
            &self.loaded,
            self.is_alpha_replaced,
            self.instance_size,
        ) {
            Ok(glob) => {
                self.glob = glob;
                self.is_invalid = false;
            }
            Err(err) => {
                self.is_invalid = true;
                error!(
                    "Loading of shader from `{:?}` has failed: {err}",
                    self.source
                );
            }
        }
    }
}

impl ShaderUpdater<'_> {
    /// Runs the update.
    pub fn apply(mut self, app: &mut App, glob: &Glob<Res<Shader>>) {
        glob.take(app, |shader, app| {
            if Update::apply_checked(&mut self.is_alpha_replaced, &mut shader.is_alpha_replaced) {
                shader.update(app);
            }
        });
        if let Some(res) = self.res.take_value(|| unreachable!()) {
            res.apply(app, glob);
        }
    }
}

/// The source of a [`Shader`].
///
/// # Examples
///
/// See [`Shader`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ShaderSource {
    /// Shader code as a string.
    String(String),
}

impl Default for ShaderSource {
    fn default() -> Self {
        Self::String(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/empty.wgsl")).into())
    }
}

impl Source for ShaderSource {
    fn is_async(&self) -> bool {
        false
    }
}
