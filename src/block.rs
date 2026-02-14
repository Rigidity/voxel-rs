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
}

pub struct FaceRect {
    pub rect: Rect,
    pub is_transparent: bool,
}

pub fn render_block_with_model(ctx: &mut RenderContext, model_id: ModelId, double_sided: bool) {
    let visible_faces = calculate_visible_faces(ctx);

    // Front face (+Z)
    if visible_faces.front {
        ctx.add_face(BlockFace::Front, model_id, double_sided);
    }

    // Back face (-Z)
    if visible_faces.back {
        ctx.add_face(BlockFace::Back, model_id, double_sided);
    }

    // Left face (-X)
    if visible_faces.left {
        ctx.add_face(BlockFace::Left, model_id, double_sided);
    }

    // Right face (+X)
    if visible_faces.right {
        ctx.add_face(BlockFace::Right, model_id, double_sided);
    }

    // Top face (+Y)
    if visible_faces.top {
        ctx.add_face(BlockFace::Top, model_id, double_sided);
    }

    // Bottom face (-Y)
    if visible_faces.bottom {
        ctx.add_face(BlockFace::Bottom, model_id, double_sided);
    }
}

pub fn calculate_visible_faces(ctx: &RenderContext) -> VisibleFaces {
    VisibleFaces {
        left: ctx
            .data
            .get_block(ctx.world_pos - IVec3::X)
            .is_none_or(|neighboring_block| {
                !ctx.is_face_obscured(ctx.block, neighboring_block, BlockFace::Left)
            }),
        right: ctx
            .data
            .get_block(ctx.world_pos + IVec3::X)
            .is_none_or(|neighboring_block| {
                !ctx.is_face_obscured(ctx.block, neighboring_block, BlockFace::Right)
            }),
        front: ctx
            .data
            .get_block(ctx.world_pos + IVec3::Z)
            .is_none_or(|neighboring_block| {
                !ctx.is_face_obscured(ctx.block, neighboring_block, BlockFace::Front)
            }),
        back: ctx
            .data
            .get_block(ctx.world_pos - IVec3::Z)
            .is_none_or(|neighboring_block| {
                !ctx.is_face_obscured(ctx.block, neighboring_block, BlockFace::Back)
            }),
        top: ctx
            .data
            .get_block(ctx.world_pos + IVec3::Y)
            .is_none_or(|neighboring_block| {
                !ctx.is_face_obscured(ctx.block, neighboring_block, BlockFace::Top)
            }),
        bottom: ctx
            .data
            .get_block(ctx.world_pos - IVec3::Y)
            .is_none_or(|neighboring_block| {
                !ctx.is_face_obscured(ctx.block, neighboring_block, BlockFace::Bottom)
            }),
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
    fn add_face(&mut self, face: BlockFace, model_id: ModelId, double_sided: bool) {
        let texture_index = self.registry.texture_index(self.block, face);

        // Check if this face is transparent
        let is_transparent = self
            .registry
            .block_type(self.block.id)
            .face_rect(face, self.block.data)
            .map(|rect| rect.is_transparent)
            .unwrap_or(false);

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
                is_transparent,
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
            // Add back face for transparent blocks
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
            // Add back face for transparent blocks
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

        let get_ao_face = |neighbor_pos: IVec3| -> BlockFace {
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
            let face = get_ao_face(side1_pos);
            self.registry
                .block_type(block.id)
                .face_rect(face, block.data)
                .is_some()
        });
        let side2 = self.data.get_block(side2_pos).is_some_and(|block| {
            let face = get_ao_face(side2_pos);
            self.registry
                .block_type(block.id)
                .face_rect(face, block.data)
                .is_some()
        });
        let corner = self.data.get_block(corner_pos).is_some_and(|block| {
            let face = get_ao_face(corner_pos);
            self.registry
                .block_type(block.id)
                .face_rect(face, block.data)
                .is_some()
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
        let neighboring_face = match render_face {
            BlockFace::Front => BlockFace::Back,
            BlockFace::Back => BlockFace::Front,
            BlockFace::Left => BlockFace::Right,
            BlockFace::Right => BlockFace::Left,
            BlockFace::Top => BlockFace::Bottom,
            BlockFace::Bottom => BlockFace::Top,
        };

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
