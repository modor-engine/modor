use crate::shader::loaded::ShaderLoaded;
use crate::Material;
use derivative::Derivative;
use glob::ShaderGlob;
use log::error;
use modor::{Context, Glob, GlobRef};
use modor_resources::{Resource, ResourceError, Source};
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct Shader<T> {
    pub is_alpha_replaced: bool,
    loaded: ShaderLoaded,
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

    fn create(ctx: &mut Context<'_>, label: &str) -> Self {
        let loaded = ShaderLoaded::default();
        let glob = ShaderGlob::new::<T>(ctx, &loaded, Self::DEFAULT_IS_ALPHA_REPLACED, label)
            .expect("internal error: cannot load empty shader");
        Self {
            is_alpha_replaced: Self::DEFAULT_IS_ALPHA_REPLACED,
            glob: Glob::new(ctx, glob),
            loaded,
            is_invalid: false,
            old_is_alpha_replaced: Self::DEFAULT_IS_ALPHA_REPLACED,
            phantom_data: PhantomData,
        }
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

    fn update(&mut self, ctx: &mut Context<'_>, loaded: Option<Self::Loaded>, label: &str) {
        if let Some(loaded) = loaded {
            self.loaded = loaded;
            self.update(ctx, label);
        } else if self.is_alpha_replaced != self.old_is_alpha_replaced {
            self.update(ctx, label);
        }
    }
}

impl<T> Shader<T>
where
    T: 'static + Material,
{
    const DEFAULT_IS_ALPHA_REPLACED: bool = false;

    /// Returns a reference to global data.
    pub fn glob(&self) -> ShaderGlobRef<T> {
        ShaderGlobRef {
            inner: self.glob.as_ref().clone(),
            phantom: PhantomData,
        }
    }

    pub fn is_invalid(&self) -> bool {
        self.is_invalid
    }

    fn update(&mut self, ctx: &mut Context<'_>, label: &str) {
        match ShaderGlob::new::<T>(ctx, &self.loaded, self.is_alpha_replaced, label) {
            Ok(glob) => {
                *self.glob.get_mut(ctx) = glob;
                self.is_invalid = false;
            }
            Err(err) => {
                self.is_invalid = true;
                error!("Loading of shader '{label}' has failed: {err}");
            }
        }
        self.old_is_alpha_replaced = self.is_alpha_replaced;
    }
}

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

#[derive(Debug, Clone)]
pub enum ShaderSource {
    String(String),
}

impl Source for ShaderSource {
    fn is_async(&self) -> bool {
        false
    }
}

pub(crate) mod glob;
mod loaded;
