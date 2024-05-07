use modor_resources::ResourceError;
use regex::Regex;
use std::str::FromStr;
use wgpu::{VertexAttribute, VertexFormat};

#[derive(Debug, PartialEq, Eq)]
pub struct ShaderLoaded {
    pub(crate) code: String,
    pub(crate) texture_count: u32,
    pub(crate) instance_vertex_attributes: Vec<VertexAttribute>,
}

impl ShaderLoaded {
    pub(crate) fn new(code: String) -> Result<Self, ResourceError> {
        Ok(Self {
            texture_count: Self::extract_texture_count(&code),
            instance_vertex_attributes: Self::extract_material_instance_struct(&code)
                .map_or_else(|| Ok(vec![]), |s| Self::extract_vertex_attributes(&s))
                .map_err(ResourceError::Other)?,
            code,
        })
    }

    fn extract_texture_count(code: &str) -> u32 {
        let binding_count = Regex::new(r"@group\(1\)\s*@binding\(([0-9]+)\)")
            .expect("internal error: invalid texture count regex")
            .captures_iter(code)
            .filter_map(|c| u32::from_str(&c[1]).ok())
            .max()
            .unwrap_or(0);
        (binding_count + 1).div_euclid(2)
    }

    fn extract_material_instance_struct(code: &str) -> Option<String> {
        Regex::new(r"(?s)struct\s+MaterialInstance\s*\{[^}]*}")
            .expect("internal error: invalid material instance struct regex")
            .captures(code)
            .map(|capture| capture[0].to_string())
    }

    fn extract_vertex_attributes(struct_code: &str) -> Result<Vec<VertexAttribute>, String> {
        let mut offset = 0;
        Regex::new(r"@location\(([0-9]+)\)\s*\w+\s*:\s*([\w<>]+)")
            .expect("internal error: invalid material instance layout regex")
            .captures_iter(struct_code)
            .filter_map(|capture| u32::from_str(&capture[1]).ok().map(|l| (capture, l)))
            .map(|(capture, shader_location)| {
                Self::format(&capture[2]).map(|format| {
                    let attribute = VertexAttribute {
                        format,
                        offset,
                        shader_location,
                    };
                    offset += format.size();
                    attribute
                })
            })
            .collect()
    }

    fn format(type_: &str) -> Result<VertexFormat, String> {
        match type_ {
            "f32" => Ok(VertexFormat::Float32),
            "i32" => Ok(VertexFormat::Sint32),
            "u32" => Ok(VertexFormat::Uint32),
            "vec2<f32>" => Ok(VertexFormat::Float32x2),
            "vec2<i32>" => Ok(VertexFormat::Sint32x2),
            "vec2<u32>" => Ok(VertexFormat::Uint32x2),
            "vec3<f32>" => Ok(VertexFormat::Float32x3),
            "vec3<i32>" => Ok(VertexFormat::Sint32x3),
            "vec3<u32>" => Ok(VertexFormat::Uint32x3),
            "vec4<f32>" => Ok(VertexFormat::Float32x4),
            "vec4<i32>" => Ok(VertexFormat::Sint32x4),
            "vec4<u32>" => Ok(VertexFormat::Uint32x4),
            _ => Err(format!(
                "WGSL type `{type_}` not supported in MaterialInstance"
            )),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod shader_loaded_tests {
    use crate::shader::loaded::ShaderLoaded;
    use modor_resources::ResourceError;
    use wgpu::{VertexAttribute, VertexFormat};

    #[modor::test]
    fn load_code_with_no_material() {
        let code = "
        @group(0)
        @binding(0)
        var<uniform> camera: Camera;
        ";
        let shader = ShaderLoaded::new(code.into()).unwrap();
        assert_eq!(shader.code, code);
        assert_eq!(shader.texture_count, 0);
    }

    #[modor::test]
    fn load_code_with_no_texture() {
        let code = "
        @group(0)
        @binding(0)
        var<uniform> camera: Camera;

        @group(1)
        @binding(0)
        var<uniform> material: Material;
        ";
        let shader = ShaderLoaded::new(code.into()).unwrap();
        assert_eq!(shader.code, code);
        assert_eq!(shader.texture_count, 0);
    }

    #[modor::test]
    fn load_code_with_one_texture() {
        let code = "
        @group(0)
        @binding(0)
        var<uniform> camera: Camera;

        @group(1)
        @binding(0)
        var<uniform> material: Material;

        @group(1)
        @binding(1)
        var texture: texture_2d<f32>;

        @group(1)
        @binding(2)
        var texture_sampler: sampler;
        ";
        let shader = ShaderLoaded::new(code.into()).unwrap();
        assert_eq!(shader.code, code);
        assert_eq!(shader.texture_count, 1);
    }

    #[modor::test]
    fn load_code_with_many_textures() {
        let code = "
        @group(0)
        @binding(0)
        var<uniform> camera: Camera;

        @group(1)
        @binding(0)
        var<uniform> material: Material;

        @group(1)
        @binding(1)
        var texture: texture_2d<f32>;

        @group(1)
        @binding(2)
        var texture_sampler: sampler;

        @group(1)
        @binding(3)
        var texture2: texture_2d<f32>;

        @group(1)
        @binding(4)
        var texture_sampler2: sampler;
        ";
        let shader = ShaderLoaded::new(code.into()).unwrap();
        assert_eq!(shader.code, code);
        assert_eq!(shader.texture_count, 2);
    }

    #[modor::test]
    fn load_code_without_material_instance_struct() {
        let code = "
        ";
        let shader = ShaderLoaded::new(code.into()).unwrap();
        assert_eq!(shader.instance_vertex_attributes, vec![]);
    }

    #[modor::test]
    fn load_code_with_empty_material_instance_struct() {
        let code = "
        struct MaterialInstance {
        }
        ";
        let shader = ShaderLoaded::new(code.into()).unwrap();
        assert_eq!(shader.instance_vertex_attributes, vec![]);
    }

    #[modor::test]
    fn load_code_with_one_material_instance_location() {
        let code = "
        struct MaterialInstance {
            @location(6)
            variable: u32
        }
        ";
        let shader = ShaderLoaded::new(code.into()).unwrap();
        assert_eq!(
            shader.instance_vertex_attributes,
            vec![VertexAttribute {
                format: VertexFormat::Uint32,
                offset: 0,
                shader_location: 6,
            }]
        );
    }

    #[modor::test]
    fn load_code_with_many_material_instance_location() {
        let code = "
        struct MaterialInstance {
            @location(6)
            variable: u32,
            @location(7)
            variable: vec2<u32>,
            @location(8)
            variable: vec3<u32>,
            @location(9)
            variable: vec4<u32>,
            @location(10)
            variable: i32,
            @location(11)
            variable: vec2<i32>,
            @location(12)
            variable: vec3<i32>,
            @location(13)
            variable: vec4<i32>,
            @location(14)
            variable: f32,
            @location(15)
            variable: vec2<f32>,
            @location(16)
            variable: vec3<f32>,
            @location(17)
            variable: vec4<f32>,
        }
        ";
        let shader = ShaderLoaded::new(code.into()).unwrap();
        assert_eq!(
            shader.instance_vertex_attributes,
            vec![
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: 0,
                    shader_location: 6,
                },
                VertexAttribute {
                    format: VertexFormat::Uint32x2,
                    offset: 4,
                    shader_location: 7,
                },
                VertexAttribute {
                    format: VertexFormat::Uint32x3,
                    offset: 12,
                    shader_location: 8,
                },
                VertexAttribute {
                    format: VertexFormat::Uint32x4,
                    offset: 24,
                    shader_location: 9,
                },
                VertexAttribute {
                    format: VertexFormat::Sint32,
                    offset: 40,
                    shader_location: 10,
                },
                VertexAttribute {
                    format: VertexFormat::Sint32x2,
                    offset: 44,
                    shader_location: 11,
                },
                VertexAttribute {
                    format: VertexFormat::Sint32x3,
                    offset: 52,
                    shader_location: 12,
                },
                VertexAttribute {
                    format: VertexFormat::Sint32x4,
                    offset: 64,
                    shader_location: 13,
                },
                VertexAttribute {
                    format: VertexFormat::Float32,
                    offset: 80,
                    shader_location: 14,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 84,
                    shader_location: 15,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 92,
                    shader_location: 16,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 104,
                    shader_location: 17,
                },
            ]
        );
    }

    #[modor::test]
    fn load_code_with_unsupported_material_instance_location() {
        let code = "
        struct MaterialInstance {
            @location(6)
            variable: i8
        }
        ";
        let shader = ShaderLoaded::new(code.into());
        assert_eq!(
            shader,
            Err(ResourceError::Other(
                "WGSL type `i8` not supported in MaterialInstance".into()
            ))
        );
    }
}
