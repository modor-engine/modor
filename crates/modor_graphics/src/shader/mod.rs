use crate::shader::loaded::ShaderLoaded;
use crate::Material;
use derivative::Derivative;
use glob::ShaderGlob;
use log::error;
use modor::{Builder, Context, Glob, GlobRef};
use modor_resources::{Resource, ResourceError, Source};
use std::marker::PhantomData;
use std::ops::Deref;

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
///     [`T::InstanceData`](Material::InstanceData) on Rust side.
///
/// # Bindings
///
/// The code can include the following bindings:
/// - group `0`
///     - binding `0`: camera data
/// - group `1`
///     - binding `0`: material data (`Material` struct corresponds to
///         [`T::Data`](Material::Data) on Rust side)
///     - binding `(i * 2)`: `texture_2d<f32>` value corresponding to texture `i`
///     - binding `(i * 2 + 1)`: `sampler` value corresponding to texture `i`
///
/// # Examples
///
/// See [`Material`].
#[derive(Builder, Derivative)]
#[derivative(Debug(bound = ""))]
pub struct Shader<T> {
    /// Controls how alpha channel should be treated:
    /// - `false`: apply standard alpha blending with non-premultiplied alpha.
    ///     It means models rendered behind a transparent model might be visible.
    /// - `true`: don't apply any color blending, just overwrites the output color.
    ///     It means models rendered behind a transparent model will never be visible.
    ///
    /// Default is `false`.
    #[builder(form(value))]
    pub is_alpha_replaced: bool,
    loaded: ShaderLoaded,
    label: String,
    glob: Glob<ShaderGlob>,
    is_invalid: bool,
    old_is_alpha_replaced: bool,
    phantom_data: PhantomData<T>,
}

impl<T> Resource for Shader<T>
where
    T: 'static + Material,
{
    type Source = ShaderSource;
    type Loaded = ShaderLoaded;

    fn label(&self) -> &str {
        &self.label
    }

    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError> {
        let code =
            String::from_utf8(file_bytes).map_err(|err| ResourceError::Other(format!("{err}")))?;
        ShaderLoaded::new(code)
    }

    fn load(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
        ShaderLoaded::new(match source {
            ShaderSource::String(string) => string.clone(),
        })
    }

    fn update(&mut self, ctx: &mut Context<'_>, loaded: Option<Self::Loaded>) {
        if let Some(loaded) = loaded {
            self.loaded = loaded;
            self.update(ctx);
        } else if self.is_alpha_replaced != self.old_is_alpha_replaced {
            self.update(ctx);
        }
    }
}

impl<T> Shader<T>
where
    T: 'static + Material,
{
    const DEFAULT_IS_ALPHA_REPLACED: bool = false;

    /// Creates a new shader.
    ///
    /// The `label` is used to identity the shader in logs.
    pub fn new(ctx: &mut Context<'_>, label: impl Into<String>) -> Self {
        let label = label.into();
        let loaded = ShaderLoaded::default();
        let glob = ShaderGlob::new::<T>(ctx, &loaded, Self::DEFAULT_IS_ALPHA_REPLACED, &label)
            .expect("internal error: cannot load empty shader");
        Self {
            is_alpha_replaced: Self::DEFAULT_IS_ALPHA_REPLACED,
            glob: Glob::new(ctx, glob),
            loaded,
            label,
            is_invalid: false,
            old_is_alpha_replaced: Self::DEFAULT_IS_ALPHA_REPLACED,
            phantom_data: PhantomData,
        }
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> ShaderGlobRef<T> {
        ShaderGlobRef {
            inner: self.glob.as_ref().clone(),
            phantom: PhantomData,
        }
    }

    /// Whether an error occurred during parsing of the shader code.
    pub fn is_invalid(&self) -> bool {
        self.is_invalid
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        match ShaderGlob::new::<T>(ctx, &self.loaded, self.is_alpha_replaced, self.label()) {
            Ok(glob) => {
                *self.glob.get_mut(ctx) = glob;
                self.is_invalid = false;
            }
            Err(err) => {
                self.is_invalid = true;
                error!("Loading of shader '{}' has failed: {err}", self.label());
            }
        }
        self.old_is_alpha_replaced = self.is_alpha_replaced;
    }
}

/// The global data of a [`Shader`] with material data of type `T`.
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct ShaderGlobRef<T> {
    inner: GlobRef<ShaderGlob>,
    phantom: PhantomData<fn(T)>,
}

impl<T> Deref for ShaderGlobRef<T> {
    type Target = GlobRef<ShaderGlob>;

    fn deref(&self) -> &Self::Target {
        &self.inner
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

impl Source for ShaderSource {
    fn is_async(&self) -> bool {
        false
    }
}

pub(crate) mod glob;
mod loaded;
