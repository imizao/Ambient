use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{Component, Concept, Enum, Identifier, ItemPathBuf, Message, Version};

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct Manifest {
    #[serde(default)]
    pub project: Project,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub components: IndexMap<ItemPathBuf, Component>,
    #[serde(default)]
    pub concepts: IndexMap<ItemPathBuf, Concept>,
    #[serde(default)]
    pub messages: IndexMap<ItemPathBuf, Message>,
    #[serde(default)]
    pub enums: IndexMap<Identifier, Enum>,
}
impl Manifest {
    pub fn parse(manifest: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(manifest)
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Default, Serialize)]
pub struct Project {
    pub id: Identifier,
    pub name: Option<String>,
    pub version: Option<Version>,
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub organization: Option<Identifier>,
    #[serde(default)]
    pub includes: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Default, Serialize)]
pub struct Build {
    #[serde(default)]
    pub rust: BuildRust,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct BuildRust {
    #[serde(rename = "feature-multibuild")]
    pub feature_multibuild: Vec<String>,
}
impl Default for BuildRust {
    fn default() -> Self {
        Self {
            feature_multibuild: vec!["client".to_string(), "server".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use crate::{
        Build, BuildRust, Component, ComponentType, Concept, ContainerType, Enum, EnumMember,
        Identifier, ItemPathBuf, Manifest, Project, Version, VersionSuffix,
    };

    fn i(s: &str) -> Identifier {
        Identifier::new(s).unwrap()
    }

    #[test]
    fn can_parse_tictactoe_toml() {
        const TOML: &str = r#"
        [project]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"

        [components]
        cell = { type = "i32", name = "Cell", description = "The ID of the cell this player is in", attributes = ["store"] }

        [concepts.cell]
        name = "Cell"
        description = "A cell object"
        [concepts.cell.components]
        cell = 0
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                project: Project {
                    id: i("tictactoe"),
                    name: Some("Tic Tac Toe".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    description: None,
                    authors: vec![],
                    organization: None,
                    includes: Default::default(),
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([(
                    ItemPathBuf::new("cell").unwrap(),
                    Component {
                        name: Some("Cell".to_string()),
                        description: Some("The ID of the cell this player is in".to_string()),
                        type_: ComponentType::Item(i("i32").into()),
                        attributes: vec![i("store").into()],
                        default: None,
                    }
                    .into()
                )]),
                concepts: IndexMap::from_iter([(
                    ItemPathBuf::new("cell").unwrap(),
                    Concept {
                        name: Some("Cell".to_string()),
                        description: Some("A cell object".to_string()),
                        extends: vec![],
                        components: IndexMap::from_iter([(
                            ItemPathBuf::new("cell").unwrap(),
                            toml::Value::Integer(0)
                        )])
                    }
                    .into()
                )]),
                messages: IndexMap::new(),
                enums: IndexMap::new(),
            })
        )
    }

    #[test]
    fn can_parse_rust_build_settings() {
        const TOML: &str = r#"
        [project]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"

        [build.rust]
        feature-multibuild = ["client"]
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                project: Project {
                    id: i("tictactoe"),
                    name: Some("Tic Tac Toe".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    description: None,
                    authors: vec![],
                    organization: None,
                    includes: Default::default(),
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string()]
                    }
                },
                components: IndexMap::new(),
                concepts: IndexMap::new(),
                messages: IndexMap::new(),
                enums: IndexMap::new(),
            })
        )
    }

    #[test]
    fn can_parse_concepts_with_documented_namespace_from_manifest() {
        use toml::Value;

        const TOML: &str = r#"
        [project]
        id = "my-project"
        name = "My Project"
        version = "0.0.1"

        [components]
        "core::transform::rotation" = { type = "quat", name = "Rotation", description = "" }
        "core::transform::scale" = { type = "vec3", name = "Scale", description = "" }
        "core::transform::spherical-billboard" = { type = "empty", name = "Spherical billboard", description = "" }
        "core::transform::translation" = { type = "vec3", name = "Translation", description = "" }

        [concepts."ns::transformable"]
        name = "Transformable"
        description = "Can be translated, rotated and scaled."

        [concepts."ns::transformable".components]
        # This is intentionally out of order to ensure that order is preserved
        "core::transform::translation" = [0, 0, 0]
        "core::transform::scale" = [1, 1, 1]
        "core::transform::rotation" = [0, 0, 0, 1]
        "#;

        let manifest = Manifest::parse(TOML).unwrap();
        assert_eq!(
            manifest,
            Manifest {
                project: Project {
                    id: i("my-project"),
                    name: Some("My Project".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    description: None,
                    authors: vec![],
                    organization: None,
                    includes: Default::default(),
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([
                    (
                        ItemPathBuf::new("core::transform::rotation").unwrap(),
                        Component {
                            name: Some("Rotation".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("quat").into()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        ItemPathBuf::new("core::transform::scale").unwrap(),
                        Component {
                            name: Some("Scale".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("vec3").into()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        ItemPathBuf::new("core::transform::spherical-billboard").unwrap(),
                        Component {
                            name: Some("Spherical billboard".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("empty").into()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        ItemPathBuf::new("core::transform::translation").unwrap(),
                        Component {
                            name: Some("Translation".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("vec3").into()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                ]),
                concepts: IndexMap::from_iter([(
                    ItemPathBuf::new("ns::transformable").unwrap(),
                    Concept {
                        name: Some("Transformable".to_string()),
                        description: Some("Can be translated, rotated and scaled.".to_string()),
                        extends: vec![],
                        components: IndexMap::from_iter([
                            (
                                ItemPathBuf::new("core::transform::translation").unwrap(),
                                Value::Array(vec![
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(0)
                                ])
                            ),
                            (
                                ItemPathBuf::new("core::transform::scale").unwrap(),
                                Value::Array(vec![
                                    Value::Integer(1),
                                    Value::Integer(1),
                                    Value::Integer(1)
                                ])
                            ),
                            (
                                ItemPathBuf::new("core::transform::rotation").unwrap(),
                                Value::Array(vec![
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(1)
                                ])
                            ),
                        ])
                    }
                    .into()
                )]),
                messages: IndexMap::new(),
                enums: IndexMap::new(),
            }
        );

        assert_eq!(
            manifest
                .concepts
                .first()
                .unwrap()
                .1
                .components
                .keys()
                .collect::<Vec<_>>(),
            vec![
                &ItemPathBuf::new("core::transform::translation").unwrap(),
                &ItemPathBuf::new("core::transform::scale").unwrap(),
                &ItemPathBuf::new("core::transform::rotation").unwrap(),
            ]
        );
    }

    #[test]
    fn can_parse_enums() {
        const TOML: &str = r#"
        [project]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"

        [enums]
        cell-state = [
            { name = "free", description = "The cell is free" },
            { name = "taken", description = "The cell is taken" }
        ]
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                project: Project {
                    id: i("tictactoe"),
                    name: Some("Tic Tac Toe".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    description: None,
                    authors: vec![],
                    organization: None,
                    includes: Default::default(),
                },
                build: Build::default(),
                components: IndexMap::new(),
                concepts: IndexMap::new(),
                messages: IndexMap::new(),
                enums: IndexMap::from_iter([(
                    i("cell-state"),
                    Enum(vec![
                        EnumMember {
                            name: i("free"),
                            description: Some("The cell is free".to_string()),
                        },
                        EnumMember {
                            name: i("taken"),
                            description: Some("The cell is taken".to_string()),
                        }
                    ])
                    .into()
                )]),
            })
        )
    }

    #[test]
    fn can_parse_container_types() {
        const TOML: &str = r#"
        [project]
        id = "test"
        name = "Test"
        version = "0.0.1"

        [components]
        test = { type = "i32", name = "Test", description = "Test" }
        vec-test = { type = { container_type = "vec", element_type = "i32" }, name = "Test", description = "Test" }
        option-test = { type = { container_type = "option", element_type = "i32" }, name = "Test", description = "Test" }

        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                project: Project {
                    id: i("test"),
                    name: Some("Test".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    description: None,
                    authors: vec![],
                    organization: None,
                    includes: Default::default(),
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([
                    (
                        ItemPathBuf::new("test").unwrap(),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Item(i("i32").into()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        ItemPathBuf::new("vec-test").unwrap(),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Contained {
                                type_: ContainerType::Vec,
                                element_type: i("i32").into()
                            },
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        ItemPathBuf::new("option-test").unwrap(),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Contained {
                                type_: ContainerType::Option,
                                element_type: i("i32").into()
                            },
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    )
                ]),
                concepts: IndexMap::new(),
                messages: IndexMap::new(),
                enums: IndexMap::new(),
            })
        )
    }
}
