//! This module contains all "leaves" that can be created underneath [`RootProp`](crate::RootProp)
//! subtrees for a [`SchemaVariant`](crate::SchemaVariant). In this domain, a "leaf" is considered
//! to be a subtree or single primitive [`Prop`](crate::Prop) of an immediate child underneath
//! "/root".

mod code_generation;

pub use code_generation::CodeGeneratedPayload;

// ------------------------------------------------------------------------------------------------
// TODO(nick): re-use or delete the below once we add the next "thing" to the prop tree...
// which will probably be qualifications or validations. We may want it generic, but we may not.
// I generally lean towards anti-D.R.Y., but if there's mass re-use, let's do something like the
// below maybe? Keep in mind: we use map entries now rather than new objects, so the implementation
// below will also have to be converted for that paradigm.
// ------------------------------------------------------------------------------------------------
//
// use async_trait::async_trait;
//
// #[async_trait]
// trait Leaf {
//     async fn new(ctx: &DalContext, tree_prop_id: PropId) -> SchemaVariantResult<Box<Self>>;
//
//     async fn finish(
//         &self,
//         ctx: &DalContext,
//         domain_prop_id: PropId,
//         tree_attribute_prototype_id: AttributePrototypeId,
//         func_argument_id: FuncArgumentId,
//     ) -> SchemaVariantResult<()>;
// }
//
// #[derive(Copy, Clone)]
// pub enum LeafKind {
//     CodeGeneration,
// }
//
// #[derive(Debug)]
// pub enum FinishedLeaf {
//     CodeGeneration(CodeGenerationLeaf),
// }
//
// impl SchemaVariant {
//     pub async fn add_leaf(
//         ctx: &DalContext,
//         func_id: FuncId,
//         func_argument_id: FuncArgumentId,
//         schema_variant_id: SchemaVariantId,
//         _leaf_kind: LeafKind,
//     ) -> SchemaVariantResult<FinishedLeaf> {
//         if schema_variant_id.is_none() {
//             return Err(SchemaVariantError::InvalidSchemaVariant);
//         }
//
//         // Collect the root prop for the schema variant as we will need it to setup new props
//         // and intelligence.
//         let root_prop = Self::root_prop(ctx, schema_variant_id).await?;
//
//         // The new prop is named after the func name since func names must be unique for a given
//         // tenancy and visibility. If that changes, then this may break.
//         let func = Func::get_by_id(ctx, &func_id)
//             .await?
//             .ok_or(FuncError::NotFound(func_id))?;
//         let mut tree_prop = Prop::new(ctx, func.name(), PropKind::Object, None).await?;
//         tree_prop.set_hidden(ctx, true).await?;
//         let tree_prop_id = *tree_prop.id();
//
//         // FIXME(nick): re-add once a second variant is added.
//         // let parent_prop_id = match leaf_kind {
//         //     CodeGeneration => root_prop.code_prop_id,
//         // };
//         let parent_prop_id = root_prop.code_prop_id;
//         tree_prop.set_parent_prop(ctx, parent_prop_id).await?;
//
//         // Now that the tree is ready, let's create its children.
//         // FIXME(nick): re-add once a second variant is added.
//         // let leaf = match leaf_kind {
//         //     CodeGeneration => *CodeGenerationLeaf::new(ctx, tree_prop_id).await?,
//         // };
//         let leaf = *CodeGenerationLeaf::new(ctx, tree_prop_id).await?;
//
//         // Finalize the schema variant (which will likely be done again).
//         SchemaVariant::finalize_for_id(ctx, schema_variant_id).await?;
//
//         // Modify the prototype to use the function.
//         let tree_read_context = AttributeReadContext::default_with_prop(tree_prop_id);
//         let tree_attribute_value = AttributeValue::find_for_context(ctx, tree_read_context)
//             .await?
//             .ok_or(AttributeValueError::NotFoundForReadContext(
//                 tree_read_context,
//             ))?;
//         let mut tree_attribute_prototype = tree_attribute_value
//             .attribute_prototype(ctx)
//             .await?
//             .ok_or(AttributeValueError::MissingAttributePrototype)?;
//         tree_attribute_prototype.set_func_id(ctx, func_id).await?;
//
//         // Finish and return the leaf.
//         leaf.finish(
//             ctx,
//             root_prop.domain_prop_id,
//             *tree_attribute_prototype.id(),
//             func_argument_id,
//         )
//         .await?;
//         Ok(FinishedLeaf::CodeGeneration(leaf))
//     }
// }
//
// #[async_trait]
// impl Leaf for CodeGenerationLeaf {
//     async fn new(ctx: &DalContext, tree_prop_id: PropId) -> SchemaVariantResult<Box<Self>> {
//         let code_map_item_prop = SchemaVariant::find_code_item_prop()
//
//         let first_inserted_album_attribute_value_id = AttributeValue::insert_for_context(
//             ctx,
//             insert_context,
//             *map_attribute_value.id(),
//             Some(serde_json::json![{}]),
//             Some("first".to_string()),
//         )
//             .await
//             .expect("could not insert for context");
//
//         Ok(Box::new(Self {
//             tree_prop_id,
//             code_prop_id: *code_prop.id(),
//             format_prop_id: *child_format_prop.id(),
//         }))
//     }
//
//     async fn finish(
//         &self,
//         ctx: &DalContext,
//         domain_prop_id: PropId,
//         tree_attribute_prototype_id: AttributePrototypeId,
//         func_argument_id: FuncArgumentId,
//     ) -> SchemaVariantResult<()> {
//         let domain_implicit_internal_provider =
//             InternalProvider::find_for_prop(ctx, domain_prop_id)
//                 .await?
//                 .ok_or(InternalProviderError::NotFoundForProp(domain_prop_id))?;
//         AttributePrototypeArgument::new_for_intra_component(
//             ctx,
//             tree_attribute_prototype_id,
//             func_argument_id,
//             *domain_implicit_internal_provider.id(),
//         )
//             .await?;
//         Ok(())
//     }
// }
