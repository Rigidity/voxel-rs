use bevy::math::Rect;

use crate::{
    Block, BlockFace, BlockType, FaceRect, ModelId, PackedData, Registry, RenderContext,
    render_block_with_model,
};

pub struct Glass;

impl BlockType for Glass {
    fn unique_name(&self) -> String {
        "glass".to_string()
    }

    fn model_id(&self, registry: &Registry, _data: PackedData) -> ModelId {
        registry.model_id("cube")
    }

    fn register(&self, registry: &mut Registry) {
        let block_id = registry.block_id(&self.unique_name());
        let image =
            image::load_from_memory(include_bytes!("../../textures/blocks/glass.png")).unwrap();

        let texture_index = registry.add_image(image);

        registry.register_texture(
            Block::new(block_id, PackedData::builder().build()),
            texture_index,
        );
    }

    fn face_rect(&self, _face: BlockFace, _data: PackedData) -> Option<FaceRect> {
        Some(FaceRect {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            is_transparent: true,
        })
    }

    fn render(&self, ctx: &mut RenderContext) {
        render_block_with_model(ctx, self.model_id(ctx.registry, ctx.block.data), true);
    }
}
