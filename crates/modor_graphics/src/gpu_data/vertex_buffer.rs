use std::mem;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

pub(crate) trait VertexBuffer<const L: u32>: Sized {
    const ATTRIBUTES: &'static [VertexAttribute];
    const STEP_MODE: VertexStepMode;
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<Self>() as BufferAddress,
        step_mode: Self::STEP_MODE,
        attributes: <Self as VertexBuffer<L>>::ATTRIBUTES,
    };
}
