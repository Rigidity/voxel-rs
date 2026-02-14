use bevy::{math::USizeVec3, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    Aabb, BlockId, ChunkMeshBuilder, ChunkVertex, ModelId, PackedData, Registry, RelevantChunks,
};

pub trait BlockType: 'static + Send + Sync {
    fn unique_name(&self) -> String;

    fn get_aabb(&self, _data: PackedData) -> Option<Aabb> {
        Some(Aabb::new(Vec3::ZERO, Vec3::splat(1.0)))
    }

    fn register(&self, registry: &mut Registry);

    fn model_id(&self, registry: &Registry, _data: PackedData) -> ModelId {
        registry.model_id("cube")
    }

    fn render(&self, ctx: &mut RenderContext) {
        render_block_with_model(ctx, self.model_id(ctx.registry, ctx.block.data), false);
    }

    fn face_rect(&self, _face: BlockFace, _data: PackedData) -> Option<FaceRect> {
        Some(FaceRect {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            is_transparent: false,
        })
    }

    fn occludes_vertex_shading(&self, face: BlockFace, data: PackedData) -> bool {
        self.face_rect(face, data)
            .is_some_and(|face_rect| !face_rect.is_transparent)
    }
}

pub struct FaceRect {
    pub rect: Rect,
    pub is_transparent: bool,
}

pub fn render_block_with_model(ctx: &mut RenderContext, model_id: ModelId, double_sided: bool) {
    let texture_index = ctx.texture_index(ctx.block);

    for face in BlockFace::ALL {
        ctx.add_model_face(model_id, face, texture_index, double_sided);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockFace {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

impl BlockFace {
    pub const ALL: [BlockFace; 6] = [
        BlockFace::Front,
        BlockFace::Back,
        BlockFace::Left,
        BlockFace::Right,
        BlockFace::Top,
        BlockFace::Bottom,
    ];

    pub fn normal(self) -> IVec3 {
        match self {
            BlockFace::Top => IVec3::Y,
            BlockFace::Bottom => -IVec3::Y,
            BlockFace::Left => -IVec3::X,
            BlockFace::Right => IVec3::X,
            BlockFace::Front => IVec3::Z,
            BlockFace::Back => -IVec3::Z,
        }
    }

    pub fn opposite(self) -> BlockFace {
        match self {
            BlockFace::Front => BlockFace::Back,
            BlockFace::Back => BlockFace::Front,
            BlockFace::Left => BlockFace::Right,
            BlockFace::Right => BlockFace::Left,
            BlockFace::Top => BlockFace::Bottom,
            BlockFace::Bottom => BlockFace::Top,
        }
    }

    pub fn model_vertex_start(self) -> u32 {
        match self {
            BlockFace::Front => 0,
            BlockFace::Back => 4,
            BlockFace::Left => 8,
            BlockFace::Right => 12,
            BlockFace::Top => 16,
            BlockFace::Bottom => 20,
        }
    }

    pub fn shading_offsets(self) -> [IVec3; 4] {
        match self {
            BlockFace::Front => [
                IVec3::new(1, 1, 1),
                IVec3::new(-1, 1, 1),
                IVec3::new(-1, -1, 1),
                IVec3::new(1, -1, 1),
            ],
            BlockFace::Back => [
                IVec3::new(-1, 1, -1),
                IVec3::new(1, 1, -1),
                IVec3::new(1, -1, -1),
                IVec3::new(-1, -1, -1),
            ],
            BlockFace::Left => [
                IVec3::new(-1, 1, 1),
                IVec3::new(-1, 1, -1),
                IVec3::new(-1, -1, -1),
                IVec3::new(-1, -1, 1),
            ],
            BlockFace::Right => [
                IVec3::new(1, 1, -1),
                IVec3::new(1, 1, 1),
                IVec3::new(1, -1, 1),
                IVec3::new(1, -1, -1),
            ],
            BlockFace::Top => [
                IVec3::new(1, 1, 1),
                IVec3::new(1, 1, -1),
                IVec3::new(-1, 1, -1),
                IVec3::new(-1, 1, 1),
            ],
            BlockFace::Bottom => [
                IVec3::new(1, -1, -1),
                IVec3::new(1, -1, 1),
                IVec3::new(-1, -1, 1),
                IVec3::new(-1, -1, -1),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlockSurface {
    pub cull_face: Option<BlockFace>,
    pub vertex_indices: [u32; 4],
    pub normal: IVec3,
    pub shading_offsets: [IVec3; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub data: PackedData,
}

impl Block {
    pub fn new(id: BlockId, data: PackedData) -> Self {
        Self { id, data }
    }
}

pub struct RenderContext<'a> {
    pub data: &'a RelevantChunks,
    pub registry: &'a Registry,
    pub mesh: &'a mut ChunkMeshBuilder,
    pub block: Block,
    pub local_pos: USizeVec3,
    pub world_pos: IVec3,
}

impl RenderContext<'_> {
    pub fn texture_index(&self, block: Block) -> u32 {
        self.registry.texture_index(block)
    }

    pub fn texture_index_for_data(&self, data: PackedData) -> u32 {
        self.texture_index(Block::new(self.block.id, data))
    }

    pub fn add_model_face(
        &mut self,
        model_id: ModelId,
        face: BlockFace,
        texture_index: u32,
        double_sided: bool,
    ) {
        let model_index = self.registry.model_offset(model_id);
        let start = model_index + face.model_vertex_start();
        let surface = BlockSurface {
            cull_face: Some(face),
            vertex_indices: [start, start + 1, start + 2, start + 3],
            normal: face.normal(),
            shading_offsets: face.shading_offsets(),
        };

        let is_transparent = self
            .registry
            .block_type(self.block.id)
            .face_rect(face, self.block.data)
            .map(|rect| rect.is_transparent)
            .unwrap_or(false);

        self.add_surface(surface, texture_index, is_transparent, double_sided);
    }

    pub fn is_face_visible(&self, face: BlockFace) -> bool {
        self.data
            .get_block(self.world_pos + face.normal())
            .is_none_or(|neighboring_block| {
                !self.is_face_obscured(self.block, neighboring_block, face)
            })
    }

    pub fn add_surface(
        &mut self,
        surface: BlockSurface,
        texture_index: u32,
        is_transparent: bool,
        double_sided: bool,
    ) {
        if let Some(cull_face) = surface.cull_face
            && !self.is_face_visible(cull_face)
        {
            return;
        }

        let index = self.mesh.index();

        let shading: [u32; 4] = [
            self.sample_vertex_shading(surface.shading_offsets[0], surface.normal),
            self.sample_vertex_shading(surface.shading_offsets[1], surface.normal),
            self.sample_vertex_shading(surface.shading_offsets[2], surface.normal),
            self.sample_vertex_shading(surface.shading_offsets[3], surface.normal),
        ];

        for (i, shading) in shading.into_iter().enumerate() {
            self.mesh.vertices.push(ChunkVertex::new(
                self.local_pos,
                surface.vertex_indices[i],
                shading,
                texture_index,
                is_transparent,
            ));
        }

        if shading[0] + shading[2] < shading[1] + shading[3] {
            self.mesh.indices.extend_from_slice(&[
                index,
                index + 1,
                index + 3,
                index + 1,
                index + 2,
                index + 3,
            ]);
            if double_sided {
                self.mesh.indices.extend_from_slice(&[
                    index + 3,
                    index + 1,
                    index,
                    index + 3,
                    index + 2,
                    index + 1,
                ]);
            }
        } else {
            self.mesh.indices.extend_from_slice(&[
                index,
                index + 1,
                index + 2,
                index + 2,
                index + 3,
                index,
            ]);
            if double_sided {
                self.mesh.indices.extend_from_slice(&[
                    index,
                    index + 3,
                    index + 2,
                    index + 2,
                    index + 1,
                    index,
                ]);
            }
        }
    }

    fn sample_vertex_shading(&self, offset: IVec3, normal: IVec3) -> u32 {
        let (axis1, axis2) = if normal.x.abs() == 1 {
            (IVec3::Y, IVec3::Z)
        } else if normal.y.abs() == 1 {
            (IVec3::X, IVec3::Z)
        } else {
            (IVec3::X, IVec3::Y)
        };

        let side1_dir = offset.dot(axis1).signum();
        let side2_dir = offset.dot(axis2).signum();

        let side1_pos = self.world_pos + normal + axis1 * side1_dir;
        let side2_pos = self.world_pos + normal + axis2 * side2_dir;
        let corner_pos = self.world_pos + normal + axis1 * side1_dir + axis2 * side2_dir;

        let get_neighbor_face_toward_block = |neighbor_pos: IVec3| -> BlockFace {
            let diff = neighbor_pos - self.world_pos;
            if diff.x > 0 {
                BlockFace::Left
            } else if diff.x < 0 {
                BlockFace::Right
            } else if diff.y > 0 {
                BlockFace::Bottom
            } else if diff.y < 0 {
                BlockFace::Top
            } else if diff.z > 0 {
                BlockFace::Back
            } else {
                BlockFace::Front
            }
        };

        let side1 = self.data.get_block(side1_pos).is_some_and(|block| {
            let face = get_neighbor_face_toward_block(side1_pos);
            self.registry
                .block_type(block.id)
                .occludes_vertex_shading(face, block.data)
        });
        let side2 = self.data.get_block(side2_pos).is_some_and(|block| {
            let face = get_neighbor_face_toward_block(side2_pos);
            self.registry
                .block_type(block.id)
                .occludes_vertex_shading(face, block.data)
        });
        let corner = self.data.get_block(corner_pos).is_some_and(|block| {
            let face = get_neighbor_face_toward_block(corner_pos);
            self.registry
                .block_type(block.id)
                .occludes_vertex_shading(face, block.data)
        });

        let occlusion = if side1 && side2 {
            3
        } else {
            (side1 as u32) + (side2 as u32) + (corner as u32)
        };

        3 - occlusion
    }

    fn is_face_obscured(
        &self,
        render_block: Block,
        neighboring_block: Block,
        render_face: BlockFace,
    ) -> bool {
        let neighboring_face = render_face.opposite();

        let render_rect = self
            .registry
            .block_type(render_block.id)
            .face_rect(render_face, render_block.data);

        let neighboring_rect = self
            .registry
            .block_type(neighboring_block.id)
            .face_rect(neighboring_face, neighboring_block.data);

        if let (Some(render_rect), Some(neighboring_rect)) = (render_rect, neighboring_rect) {
            if render_rect.is_transparent != neighboring_rect.is_transparent {
                return false;
            }

            is_face_obscured(render_rect.rect, neighboring_rect.rect)
        } else {
            false
        }
    }
}

fn is_face_obscured(render_rect: Rect, neighboring_rect: Rect) -> bool {
    render_rect.min.x >= neighboring_rect.min.x
        && render_rect.max.x <= neighboring_rect.max.x
        && render_rect.min.y >= neighboring_rect.min.y
        && render_rect.max.y <= neighboring_rect.max.y
}
