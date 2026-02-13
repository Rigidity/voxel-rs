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

    fn face_data(&self, _face: BlockFace, data: PackedData) -> PackedData {
        data
    }

    fn is_solid(&self) -> bool {
        true
    }

    fn model_name(&self) -> &str {
        "cube"
    }

    fn render(&self, ctx: &mut RenderContext) {
        render_block_with_model(ctx, self.model_name());
    }
}

pub fn render_block_with_model(ctx: &mut RenderContext, model_name: &str) {
    let model_id = ctx.registry.model_id(model_name);
    let visible_faces = calculate_visible_faces(ctx);

    // Front face (+Z)
    if visible_faces.front {
        ctx.add_face(BlockFace::Front, model_id);
    }

    // Back face (-Z)
    if visible_faces.back {
        ctx.add_face(BlockFace::Back, model_id);
    }

    // Left face (-X)
    if visible_faces.left {
        ctx.add_face(BlockFace::Left, model_id);
    }

    // Right face (+X)
    if visible_faces.right {
        ctx.add_face(BlockFace::Right, model_id);
    }

    // Top face (+Y)
    if visible_faces.top {
        ctx.add_face(BlockFace::Top, model_id);
    }

    // Bottom face (-Y)
    if visible_faces.bottom {
        ctx.add_face(BlockFace::Bottom, model_id);
    }
}

pub fn calculate_visible_faces(ctx: &RenderContext) -> VisibleFaces {
    let is_transparent = |neighboring_block: Block| -> bool {
        let neighboring_solid = ctx.registry.block_type(neighboring_block.id).is_solid();
        !neighboring_solid && ctx.registry.block_type(ctx.block.id).is_solid()
    };

    VisibleFaces {
        left: ctx
            .data
            .get_block(ctx.world_pos - IVec3::X)
            .is_none_or(is_transparent),
        right: ctx
            .data
            .get_block(ctx.world_pos + IVec3::X)
            .is_none_or(is_transparent),
        front: ctx
            .data
            .get_block(ctx.world_pos + IVec3::Z)
            .is_none_or(is_transparent),
        back: ctx
            .data
            .get_block(ctx.world_pos - IVec3::Z)
            .is_none_or(is_transparent),
        top: ctx
            .data
            .get_block(ctx.world_pos + IVec3::Y)
            .is_none_or(is_transparent),
        bottom: ctx
            .data
            .get_block(ctx.world_pos - IVec3::Y)
            .is_none_or(is_transparent),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VisibleFaces {
    pub left: bool,
    pub right: bool,
    pub front: bool,
    pub back: bool,
    pub top: bool,
    pub bottom: bool,
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
    fn add_face(&mut self, face: BlockFace, model_id: ModelId) {
        let solid = self.registry.block_type(self.block.id).is_solid();
        let texture_index = self.registry.texture_index(self.block, face);

        let model_index = self.registry.model_offset(model_id);

        let base_index = model_index
            + match face {
                BlockFace::Front => 0,
                BlockFace::Back => 4,
                BlockFace::Left => 8,
                BlockFace::Right => 12,
                BlockFace::Top => 16,
                BlockFace::Bottom => 20,
            };

        let vertex_indices = [base_index, base_index + 1, base_index + 2, base_index + 3];

        let index = self.mesh.index();

        // Calculate AO for each vertex
        let (normal, ao_offsets) = match face {
            BlockFace::Front => (
                IVec3::Z,
                [
                    IVec3::new(1, 1, 1),
                    IVec3::new(-1, 1, 1),
                    IVec3::new(-1, -1, 1),
                    IVec3::new(1, -1, 1),
                ],
            ),
            BlockFace::Back => (
                -IVec3::Z,
                [
                    IVec3::new(-1, 1, -1),
                    IVec3::new(1, 1, -1),
                    IVec3::new(1, -1, -1),
                    IVec3::new(-1, -1, -1),
                ],
            ),
            BlockFace::Left => (
                -IVec3::X,
                [
                    IVec3::new(-1, 1, 1),
                    IVec3::new(-1, 1, -1),
                    IVec3::new(-1, -1, -1),
                    IVec3::new(-1, -1, 1),
                ],
            ),
            BlockFace::Right => (
                IVec3::X,
                [
                    IVec3::new(1, 1, -1),
                    IVec3::new(1, 1, 1),
                    IVec3::new(1, -1, 1),
                    IVec3::new(1, -1, -1),
                ],
            ),
            BlockFace::Top => (
                IVec3::Y,
                [
                    IVec3::new(1, 1, 1),
                    IVec3::new(1, 1, -1),
                    IVec3::new(-1, 1, -1),
                    IVec3::new(-1, 1, 1),
                ],
            ),
            BlockFace::Bottom => (
                -IVec3::Y,
                [
                    IVec3::new(1, -1, -1),
                    IVec3::new(1, -1, 1),
                    IVec3::new(-1, -1, 1),
                    IVec3::new(-1, -1, -1),
                ],
            ),
        };

        let aos: [u32; 4] = [
            self.calculate_ao(ao_offsets[0], normal),
            self.calculate_ao(ao_offsets[1], normal),
            self.calculate_ao(ao_offsets[2], normal),
            self.calculate_ao(ao_offsets[3], normal),
        ];

        // Add vertices
        for i in 0..4 {
            self.mesh.vertices.push(ChunkVertex::new(
                self.local_pos,
                vertex_indices[i],
                aos[i],
                texture_index,
            ));
        }

        // Add indices with proper winding based on AO
        if aos[0] + aos[2] < aos[1] + aos[3] {
            self.mesh.indices.extend_from_slice(&[
                index,
                index + 1,
                index + 3,
                index + 1,
                index + 2,
                index + 3,
            ]);
            // Add back face for non-solid blocks
            if !solid {
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
            // Add back face for non-solid blocks
            if !solid {
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

    fn calculate_ao(&self, offset: IVec3, normal: IVec3) -> u32 {
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

        let side1 = self
            .data
            .get_block(side1_pos)
            .is_some_and(|block| self.registry.block_type(block.id).is_solid());
        let side2 = self
            .data
            .get_block(side2_pos)
            .is_some_and(|block| self.registry.block_type(block.id).is_solid());
        let corner = self
            .data
            .get_block(corner_pos)
            .is_some_and(|block| self.registry.block_type(block.id).is_solid());

        let occlusion = if side1 && side2 {
            3
        } else {
            (side1 as u32) + (side2 as u32) + (corner as u32)
        };

        3 - occlusion
    }
}
