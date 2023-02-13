use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Context;
use quote::quote;
use serde::Deserialize;

#[cfg(test)]
mod tests;

pub fn read_file(file_path: String) -> anyhow::Result<(String, String)> {
    let file_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("no manifest dir")?)
        .join(file_path);
    let file_path_str = format!("{}", file_path.display());

    let contents = std::fs::read_to_string(&file_path)?;

    Ok((file_path_str, contents))
}

pub fn implementation(
    (file_path, contents): (String, String),
    api_name: syn::Path,
    global_namespace: bool,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let manifest: Manifest = toml::from_str(&contents)?;

    #[derive(Debug)]
    enum TreeNodeInner {
        Module(BTreeMap<String, TreeNode>),
        Component(Component),
    }

    #[derive(Debug)]
    struct TreeNode {
        path: Vec<String>,
        inner: TreeNodeInner,
    }
    impl TreeNode {
        fn new(path: Vec<String>, inner: TreeNodeInner) -> Self {
            Self { path, inner }
        }
    }

    let mut root = BTreeMap::new();
    fn insert_into_root(
        root: &mut BTreeMap<String, TreeNode>,
        segments: &[String],
        inner: TreeNodeInner,
    ) -> anyhow::Result<()> {
        let mut manifest_head = root;
        let (leaf_id, modules) = segments.split_last().context("empty segments")?;

        let mut segments_so_far = vec![];
        for segment in modules {
            segments_so_far.push(segment.to_string());

            let new_head = manifest_head
                .entry(segment.to_string())
                .or_insert(TreeNode::new(
                    segments_so_far.clone(),
                    TreeNodeInner::Module(Default::default()),
                ));

            manifest_head = match &mut new_head.inner {
                TreeNodeInner::Module(module) => module,
                _ => anyhow::bail!("found a non-module where a module was expected"),
            };
        }

        manifest_head.insert(
            leaf_id.clone(),
            TreeNode::new(segments.iter().map(|s| s.to_string()).collect(), inner),
        );

        Ok(())
    }
    for (id, component) in manifest.components {
        insert_into_root(
            &mut root,
            &id.split("::").map(|s| s.to_string()).collect::<Vec<_>>(),
            TreeNodeInner::Component(component),
        )?;
    }

    fn expand_tree(
        tree_node: &TreeNode,
        api_name: &syn::Path,
        project_path: &[String],
    ) -> anyhow::Result<proc_macro2::TokenStream> {
        let name = tree_node
            .path
            .last()
            .map(|s| s.as_str())
            .unwrap_or_default();
        match &tree_node.inner {
            TreeNodeInner::Module(module) => {
                let children = module
                    .values()
                    .map(|child| expand_tree(child, api_name, project_path))
                    .collect::<Result<Vec<_>, _>>()?;

                let prelude = quote! {
                    use #api_name::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                };

                Ok(if name.is_empty() {
                    quote! {
                        #prelude
                        #(#children)*
                    }
                } else {
                    let name_ident: syn::Path = syn::parse_str(name)?;
                    quote! {
                        pub mod #name_ident {
                            #prelude
                            #(#children)*
                        }
                    }
                })
            }
            TreeNodeInner::Component(component) => {
                let name_ident: syn::Path = syn::parse_str(name)?;
                let name_uppercase_ident: syn::Path = syn::parse_str(&name.to_ascii_uppercase())?;
                let component_ty = component.type_.to_token_stream(api_name)?;

                let mut doc_comment = format!("**{}**", component.name);

                if !component.description.is_empty() {
                    doc_comment += &format!(": {}", component.description.replace('\n', "\n\n"));
                }

                if !component.attributes.is_empty() {
                    doc_comment += &format!(
                        "\n\n*Attributes*: {}",
                        component.attributes.clone().join(", ")
                    )
                }

                let id = [project_path, &tree_node.path].concat().join("::");

                Ok(quote! {
                    static #name_uppercase_ident: Lazy<Component<#component_ty>> = Lazy::new(|| __internal_get_component(#id));
                    #[doc = #doc_comment]
                    pub fn #name_ident() -> Component<#component_ty> { *#name_uppercase_ident }
                })
            }
        }
    }

    let project_path: Vec<_> = if global_namespace {
        vec![]
    } else {
        manifest
            .project
            .organization
            .iter()
            .chain(std::iter::once(&manifest.project.id))
            .cloned()
            .collect()
    };
    let expanded_tree = expand_tree(
        &TreeNode::new(vec![], TreeNodeInner::Module(root)),
        &api_name,
        &project_path,
    )?;
    Ok(quote!(
        const _PROJECT_MANIFEST: &'static str = include_str!(#file_path);
        #[allow(missing_docs)]
        /// Auto-generated component definitions. These come from `kiwi.toml` in the root of the project.
        pub mod components {
            #expanded_tree
        }
    ))
}

#[derive(Deserialize, Debug)]
struct Manifest {
    project: Project,
    #[serde(default)]
    components: BTreeMap<String, Component>,
}

#[derive(Deserialize, Debug)]
pub struct Project {
    id: String,
    organization: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Component {
    name: String,
    description: String,
    #[serde(rename = "type")]
    type_: ComponentType,
    #[serde(default)]
    attributes: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ComponentType {
    String(String),
    ContainerType {
        #[serde(rename = "type")]
        type_: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        element_type: Option<String>,
    },
}
impl ComponentType {
    fn convert_primitive_type_to_rust_type(
        api_name: &syn::Path,
        ty: &str,
    ) -> Option<proc_macro2::TokenStream> {
        match ty {
            "Empty" => Some(quote! {()}),
            "Bool" => Some(quote! {bool}),
            "EntityId" => Some(quote! {#api_name::global::EntityId}),
            "F32" => Some(quote! {f32}),
            "F64" => Some(quote! {f64}),
            "Mat4" => Some(quote! {#api_name::global::Mat4}),
            "I32" => Some(quote! {i32}),
            "Quat" => Some(quote! {#api_name::global::Quat}),
            "String" => Some(quote! {String}),
            "U32" => Some(quote! {u32}),
            "U64" => Some(quote! {u64}),
            "Vec2" => Some(quote! {#api_name::global::Vec2}),
            "Vec3" => Some(quote! {#api_name::global::Vec3}),
            "Vec4" => Some(quote! {#api_name::global::Vec4}),
            "ObjectRef" => Some(quote! {#api_name::global::ObjectRef}),
            "EntityUid" => Some(quote! {#api_name::global::EntityUid}),
            _ => None,
        }
    }

    fn convert_container_type_to_rust_type(ty: &str) -> Option<proc_macro2::TokenStream> {
        match ty {
            "Vec" => Some(quote! {Vec}),
            "Option" => Some(quote! {Option}),
            _ => None,
        }
    }

    fn to_token_stream(&self, api_name: &syn::Path) -> anyhow::Result<proc_macro2::TokenStream> {
        match self {
            ComponentType::String(ty) => Self::convert_primitive_type_to_rust_type(api_name, ty)
                .context("invalid primitive type"),
            ComponentType::ContainerType {
                type_,
                element_type,
            } => {
                let container_ty = Self::convert_container_type_to_rust_type(type_)
                    .context("invalid container type")?;
                let element_ty = element_type
                    .as_deref()
                    .map(|ty| Self::convert_primitive_type_to_rust_type(api_name, ty))
                    .context("invalid element type")?;
                Ok(if let Some(element_ty) = element_ty {
                    quote! { #container_ty < #element_ty > }
                } else {
                    container_ty
                })
            }
        }
    }
}
