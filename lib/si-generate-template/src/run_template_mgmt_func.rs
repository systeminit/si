use std::collections::{
    HashMap,
    HashSet,
};

use askama::Template;

#[derive(Template, Debug, Clone)]
#[template(path = "run_template_mgmt_func.ts")]
pub struct RunTemplateMgmtFunc {
    components: Vec<RunTemplateComponent>,
}

impl RunTemplateMgmtFunc {
    pub fn new(components: Vec<RunTemplateComponent>) -> RunTemplateMgmtFunc {
        RunTemplateMgmtFunc { components }
    }

    pub fn sort_components_by_dependencies(&mut self) {
        // Create a map from variable_name to index for quick lookup
        let mut name_to_index: HashMap<String, usize> = HashMap::new();
        for (i, component) in self.components.iter().enumerate() {
            name_to_index.insert(component.variable_name.clone(), i);
        }

        // Build dependency graph: component -> set of components it depends on
        let mut dependencies: HashMap<usize, HashSet<usize>> = HashMap::new();

        for (i, component) in self.components.iter().enumerate() {
            let mut deps = HashSet::new();

            // Check each attribute for subscriptions with variables
            for attr in &component.attributes {
                if let AttributeSource::Subscription(sub) = &attr.value {
                    if let Some(variable_name) = &sub.variable {
                        // This component depends on the component with variable_name
                        if let Some(&dep_index) = name_to_index.get(variable_name) {
                            deps.insert(dep_index);
                        }
                    }
                }
            }

            dependencies.insert(i, deps);
        }

        // Perform topological sort using Kahn's algorithm
        let mut in_degree = vec![0; self.components.len()];

        // Calculate in-degrees
        for (&component, deps) in &dependencies {
            in_degree[component] = deps.len();
        }

        // Find nodes with no incoming edges, sorted by variable_name for stability
        let mut queue = std::collections::BinaryHeap::new();
        for (i, &degree) in in_degree.iter().enumerate() {
            if degree == 0 {
                // Use reverse ordering because BinaryHeap is a max-heap, but we want min-heap behavior
                queue.push(std::cmp::Reverse((
                    self.components[i].variable_name.clone(),
                    i,
                )));
            }
        }

        let mut sorted_indices = Vec::new();

        while let Some(std::cmp::Reverse((_, current))) = queue.pop() {
            sorted_indices.push(current);

            // For each component that depends on current
            let mut newly_ready = Vec::new();
            for (&dependent, deps) in &dependencies {
                if deps.contains(&current) {
                    in_degree[dependent] -= 1;
                    if in_degree[dependent] == 0 {
                        newly_ready
                            .push((self.components[dependent].variable_name.clone(), dependent));
                    }
                }
            }

            // Add newly ready components to queue, sorted by variable_name
            for (name, idx) in newly_ready {
                queue.push(std::cmp::Reverse((name, idx)));
            }
        }

        // Reorder components based on sorted indices
        let mut sorted_components = Vec::with_capacity(self.components.len());
        for &index in &sorted_indices {
            sorted_components.push(self.components[index].clone());
        }

        // Handle any remaining components (shouldn't happen with valid DAG)
        for (i, component) in self.components.iter().enumerate() {
            if !sorted_indices.contains(&i) {
                sorted_components.push(component.clone());
            }
        }

        self.components = sorted_components;
    }
}

#[derive(Debug, Clone)]
pub struct RunTemplateComponent {
    variable_name: String,
    kind: String,
    name: String,
    pub attributes: Vec<RunTemplateAttribute>,
}

impl RunTemplateComponent {
    pub fn new(
        variable_name: impl Into<String>,
        kind: impl Into<String>,
        name: impl Into<String>,
        attributes: Vec<RunTemplateAttribute>,
    ) -> RunTemplateComponent {
        RunTemplateComponent {
            variable_name: variable_name.into(),
            kind: kind.into(),
            name: name.into(),
            attributes,
        }
    }

    pub fn attributes_pruned_and_sorted(&self) -> Vec<&RunTemplateAttribute> {
        let mut sorted_attrs: Vec<&RunTemplateAttribute> = self.attributes.iter().collect();

        // First, sort by dest_path
        sorted_attrs.sort_by(|a, b| a.dest_path.cmp(&b.dest_path));

        // Find subscription paths to use for pruning
        let mut subscription_paths = Vec::new();
        for attr in &sorted_attrs {
            if matches!(attr.value, AttributeSource::Subscription(_)) {
                subscription_paths.push(&attr.dest_path);
            }
        }

        // Prune attributes whose dest_path is a descendant of any subscription path
        let mut pruned_attrs = Vec::new();
        for attr in sorted_attrs {
            let should_prune = subscription_paths.iter().any(|sub_path| {
                // Check if this attribute's path is a descendant of the subscription path
                // A path is a descendant if it starts with the subscription path followed by a '/'
                attr.dest_path != **sub_path && attr.dest_path.starts_with(&format!("{sub_path}/"))
            });

            if !should_prune {
                pruned_attrs.push(attr);
            }
        }

        pruned_attrs
    }
}

#[derive(Debug, Clone)]
pub struct SubscriptionValue {
    pub component: String,
    pub path: String,
    pub func: Option<String>,
    pub variable: Option<String>,
}

impl SubscriptionValue {
    pub fn new(
        component: impl Into<String>,
        path: impl Into<String>,
        func: Option<String>,
        variable: Option<String>,
    ) -> SubscriptionValue {
        SubscriptionValue {
            component: component.into(),
            path: path.into(),
            func,
            variable,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AttributeSource {
    Value(serde_json::Value),
    Subscription(SubscriptionValue),
    InputSource,
}

impl AttributeSource {
    pub fn value(value: impl Into<serde_json::Value>) -> AttributeSource {
        AttributeSource::Value(value.into())
    }

    pub fn subscription(
        component: impl Into<String>,
        path: impl Into<String>,
        func: Option<String>,
        variable: Option<String>,
    ) -> AttributeSource {
        AttributeSource::Subscription(SubscriptionValue::new(component, path, func, variable))
    }

    pub fn input_source() -> AttributeSource {
        AttributeSource::InputSource
    }
}

#[derive(Debug, Clone)]
pub struct RunTemplateAttribute {
    pub dest_path: String,
    pub value: AttributeSource,
}

impl RunTemplateAttribute {
    pub fn new(dest_path: impl Into<String>, value: AttributeSource) -> RunTemplateAttribute {
        RunTemplateAttribute {
            dest_path: dest_path.into(),
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_component() {
        let mut components = Vec::new();
        let mut attributes = Vec::new();

        // Test an Attribute with a direct value
        attributes.push(RunTemplateAttribute::new(
            "/domain/CidrBlock",
            AttributeSource::value("10.0.1.0/16"),
        ));

        // Test an Attribute with a complex direct value
        attributes.push(RunTemplateAttribute::new(
            "/domain/Foo/0",
            AttributeSource::value(serde_json::json!(["one", "two", { "three": "four" }])),
        ));

        // Test a subscription
        attributes.push(RunTemplateAttribute::new(
            "/domain/Poop",
            AttributeSource::subscription("foolio", "/domain/Poop", None, None),
        ));

        // Test a subscription with a variable name
        attributes.push(RunTemplateAttribute::new(
            "/domain/Canoe",
            AttributeSource::subscription(
                "foolio",
                "/domain/Canoe",
                None,
                Some("arbalestComponent".to_string()),
            ),
        ));

        // Test a subscription with a func id for /domain/Tags
        attributes.push(RunTemplateAttribute::new(
            "/domain/Tags",
            AttributeSource::subscription(
                "tagComponent",
                "/domain/Tags",
                Some("func_12345".to_string()),
                None,
            ),
        ));

        let component = RunTemplateComponent::new(
            "vpcFriend",
            "AWS::EC2::VPC",
            "vpc friend",
            attributes.clone(),
        );
        components.push(component);
        let component2 = RunTemplateComponent::new(
            "subnetFriend",
            "AWS::EC2::Subnet",
            "subnet friend",
            attributes.clone(),
        );
        components.push(component2);

        let mgmt_func_tmpl = RunTemplateMgmtFunc::new(components);
        let output = mgmt_func_tmpl
            .render()
            .expect("failed to render management function");

        assert!(
            output.contains(r#""/si/name": namePrefix + "vpc friend","#),
            "should set si/name"
        );
        assert!(
            output.contains(r#""/domain/CidrBlock": "10.0.1.0/16","#),
            "should set domain/CidrBlock"
        );
        assert!(
            output.contains(r#""/domain/Foo/0": ["#),
            "should set domain/Foo/0"
        );
        assert!(output.contains(r#""$source": {"#), "should set a source",);

        assert!(
            output.contains(r#"template.getComponentName(arbalestComponent)"#),
            "should have a component reference by name"
        );

        assert!(
            output.contains(r#""func_12345""#),
            "should have a func id in the subscription source"
        );

        // NOTE: This is helpful for debugging! Leave it here to help out future you.
        // <3 Adam
        //
        //println!("{}", output);
        //panic!("helpful to debug");
    }

    #[test]
    fn sorts_components_by_dependencies_simple() {
        let mut components = Vec::new();

        // Create poopComponent (no dependencies)
        let poop_attrs = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("poop_value"),
        )];
        components.push(RunTemplateComponent::new(
            "poopComponent",
            "AWS::Test",
            "poop",
            poop_attrs,
        ));

        // Create fooComponent that depends on poopComponent
        let foo_attrs = vec![RunTemplateAttribute::new(
            "/domain/source",
            AttributeSource::subscription(
                "comp",
                "/domain/value",
                None,
                Some("poopComponent".to_string()),
            ),
        )];
        components.push(RunTemplateComponent::new(
            "fooComponent",
            "AWS::Test",
            "foo",
            foo_attrs,
        ));

        let mut mgmt_func = RunTemplateMgmtFunc::new(components);
        mgmt_func.sort_components_by_dependencies();

        // poopComponent should come before fooComponent
        assert_eq!(mgmt_func.components[0].variable_name, "poopComponent");
        assert_eq!(mgmt_func.components[1].variable_name, "fooComponent");
    }

    #[test]
    fn sorts_components_by_dependencies_chain() {
        let mut components = Vec::new();

        // Create components in reverse dependency order
        // cComponent depends on bComponent
        let c_attrs = vec![RunTemplateAttribute::new(
            "/domain/source",
            AttributeSource::subscription(
                "comp",
                "/domain/value",
                None,
                Some("bComponent".to_string()),
            ),
        )];
        components.push(RunTemplateComponent::new(
            "cComponent",
            "AWS::Test",
            "c",
            c_attrs,
        ));

        // bComponent depends on aComponent
        let b_attrs = vec![RunTemplateAttribute::new(
            "/domain/source",
            AttributeSource::subscription(
                "comp",
                "/domain/value",
                None,
                Some("aComponent".to_string()),
            ),
        )];
        components.push(RunTemplateComponent::new(
            "bComponent",
            "AWS::Test",
            "b",
            b_attrs,
        ));

        // aComponent has no dependencies
        let a_attrs = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("a_value"),
        )];
        components.push(RunTemplateComponent::new(
            "aComponent",
            "AWS::Test",
            "a",
            a_attrs,
        ));

        let mut mgmt_func = RunTemplateMgmtFunc::new(components);
        mgmt_func.sort_components_by_dependencies();

        // Should be in order: aComponent, bComponent, cComponent
        assert_eq!(mgmt_func.components[0].variable_name, "aComponent");
        assert_eq!(mgmt_func.components[1].variable_name, "bComponent");
        assert_eq!(mgmt_func.components[2].variable_name, "cComponent");
    }

    #[test]
    fn sorts_components_with_no_dependencies() {
        let mut components = Vec::new();

        // Create components with no dependencies
        let attrs1 = vec![RunTemplateAttribute::new(
            "/domain/value1",
            AttributeSource::value("value1"),
        )];
        components.push(RunTemplateComponent::new(
            "comp1",
            "AWS::Test",
            "component1",
            attrs1,
        ));

        let attrs2 = vec![RunTemplateAttribute::new(
            "/domain/value2",
            AttributeSource::value("value2"),
        )];
        components.push(RunTemplateComponent::new(
            "comp2",
            "AWS::Test",
            "component2",
            attrs2,
        ));

        let mut mgmt_func = RunTemplateMgmtFunc::new(components);
        mgmt_func.sort_components_by_dependencies();

        // Both components should still be present
        assert_eq!(mgmt_func.components.len(), 2);
        let names: HashSet<String> = mgmt_func
            .components
            .iter()
            .map(|c| c.variable_name.clone())
            .collect();
        assert!(names.contains("comp1"));
        assert!(names.contains("comp2"));
    }

    #[test]
    fn sorts_components_with_multiple_dependencies() {
        let mut components = Vec::new();

        // Create dComponent that depends on both bComponent and cComponent
        let d_attrs = vec![
            RunTemplateAttribute::new(
                "/domain/source1",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("bComponent".to_string()),
                ),
            ),
            RunTemplateAttribute::new(
                "/domain/source2",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("cComponent".to_string()),
                ),
            ),
        ];
        components.push(RunTemplateComponent::new(
            "dComponent",
            "AWS::Test",
            "d",
            d_attrs,
        ));

        // Create independent components
        let b_attrs = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("b_value"),
        )];
        components.push(RunTemplateComponent::new(
            "bComponent",
            "AWS::Test",
            "b",
            b_attrs,
        ));

        let c_attrs = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("c_value"),
        )];
        components.push(RunTemplateComponent::new(
            "cComponent",
            "AWS::Test",
            "c",
            c_attrs,
        ));

        let mut mgmt_func = RunTemplateMgmtFunc::new(components);
        mgmt_func.sort_components_by_dependencies();

        // dComponent should be last, bComponent and cComponent should be before it
        assert_eq!(mgmt_func.components[2].variable_name, "dComponent");

        let first_two: HashSet<String> = mgmt_func.components[0..2]
            .iter()
            .map(|c| c.variable_name.clone())
            .collect();
        assert!(first_two.contains("bComponent"));
        assert!(first_two.contains("cComponent"));
    }

    #[test]
    fn sorts_complex_dependency_graph() {
        let mut components = Vec::new();

        // Create a complex dependency graph with 10 components using names that would sort
        // differently alphabetically than their dependency order requires:
        //
        // Dependency graph:
        // zebra (no deps)
        // yak (no deps)
        // xray -> zebra
        // wolf -> yak
        // violet -> xray, wolf
        // uniform -> zebra, yak
        // tango -> violet
        // sierra -> uniform, tango
        // romeo -> sierra
        // quebec -> romeo, xray
        //
        // Expected stable topological order (respecting dependencies + alphabetical tiebreaking):
        // Level 0 (no deps): yak, zebra (alphabetically: yak < zebra)
        // Level 1: wolf (depends on yak) - comes immediately after yak
        // Level 1: zebra (no deps) - comes after wolf alphabetically among remaining
        // Level 1: uniform (depends on yak,zebra) - must wait for both yak and zebra
        // Level 1: xray (depends on zebra) - must wait for zebra
        // Level 2: violet (depends on wolf,xray) - must wait for both
        // Level 3: tango (depends on violet)
        // Level 4: sierra (depends on uniform,tango) - must wait for both
        // Level 5: romeo (depends on sierra)
        // Level 6: quebec (depends on romeo,xray) - must wait for both
        //
        // Final expected order: [yak, wolf, zebra, uniform, xray, violet, tango, sierra, romeo, quebec]

        // zebra (no dependencies)
        let zebra_attrs = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("zebra_value"),
        )];
        components.push(RunTemplateComponent::new(
            "zebra",
            "AWS::Test",
            "zebra_component",
            zebra_attrs,
        ));

        // yak (no dependencies)
        let yak_attrs = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("yak_value"),
        )];
        components.push(RunTemplateComponent::new(
            "yak",
            "AWS::Test",
            "yak_component",
            yak_attrs,
        ));

        // xray depends on zebra
        let xray_attrs = vec![RunTemplateAttribute::new(
            "/domain/source",
            AttributeSource::subscription("comp", "/domain/value", None, Some("zebra".to_string())),
        )];
        components.push(RunTemplateComponent::new(
            "xray",
            "AWS::Test",
            "xray_component",
            xray_attrs,
        ));

        // wolf depends on yak
        let wolf_attrs = vec![RunTemplateAttribute::new(
            "/domain/source",
            AttributeSource::subscription("comp", "/domain/value", None, Some("yak".to_string())),
        )];
        components.push(RunTemplateComponent::new(
            "wolf",
            "AWS::Test",
            "wolf_component",
            wolf_attrs,
        ));

        // violet depends on xray and wolf
        let violet_attrs = vec![
            RunTemplateAttribute::new(
                "/domain/source1",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("xray".to_string()),
                ),
            ),
            RunTemplateAttribute::new(
                "/domain/source2",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("wolf".to_string()),
                ),
            ),
        ];
        components.push(RunTemplateComponent::new(
            "violet",
            "AWS::Test",
            "violet_component",
            violet_attrs,
        ));

        // uniform depends on zebra and yak
        let uniform_attrs = vec![
            RunTemplateAttribute::new(
                "/domain/source1",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("zebra".to_string()),
                ),
            ),
            RunTemplateAttribute::new(
                "/domain/source2",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("yak".to_string()),
                ),
            ),
        ];
        components.push(RunTemplateComponent::new(
            "uniform",
            "AWS::Test",
            "uniform_component",
            uniform_attrs,
        ));

        // tango depends on violet
        let tango_attrs = vec![RunTemplateAttribute::new(
            "/domain/source",
            AttributeSource::subscription(
                "comp",
                "/domain/value",
                None,
                Some("violet".to_string()),
            ),
        )];
        components.push(RunTemplateComponent::new(
            "tango",
            "AWS::Test",
            "tango_component",
            tango_attrs,
        ));

        // sierra depends on uniform and tango
        let sierra_attrs = vec![
            RunTemplateAttribute::new(
                "/domain/source1",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("uniform".to_string()),
                ),
            ),
            RunTemplateAttribute::new(
                "/domain/source2",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("tango".to_string()),
                ),
            ),
        ];
        components.push(RunTemplateComponent::new(
            "sierra",
            "AWS::Test",
            "sierra_component",
            sierra_attrs,
        ));

        // romeo depends on sierra
        let romeo_attrs = vec![RunTemplateAttribute::new(
            "/domain/source",
            AttributeSource::subscription(
                "comp",
                "/domain/value",
                None,
                Some("sierra".to_string()),
            ),
        )];
        components.push(RunTemplateComponent::new(
            "romeo",
            "AWS::Test",
            "romeo_component",
            romeo_attrs,
        ));

        // quebec depends on romeo and xray
        let quebec_attrs = vec![
            RunTemplateAttribute::new(
                "/domain/source1",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("romeo".to_string()),
                ),
            ),
            RunTemplateAttribute::new(
                "/domain/source2",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("xray".to_string()),
                ),
            ),
        ];
        components.push(RunTemplateComponent::new(
            "quebec",
            "AWS::Test",
            "quebec_component",
            quebec_attrs,
        ));

        let mut mgmt_func = RunTemplateMgmtFunc::new(components);
        mgmt_func.sort_components_by_dependencies();

        // Create a position map for easy lookup
        let positions: HashMap<String, usize> = mgmt_func
            .components
            .iter()
            .enumerate()
            .map(|(i, comp)| (comp.variable_name.clone(), i))
            .collect();

        // Verify all dependency constraints are satisfied
        // zebra and yak should come before their dependents
        assert!(positions["zebra"] < positions["xray"]);
        assert!(positions["yak"] < positions["wolf"]);

        // xray should come before violet and quebec
        assert!(positions["xray"] < positions["violet"]);
        assert!(positions["xray"] < positions["quebec"]);

        // wolf should come before violet
        assert!(positions["wolf"] < positions["violet"]);

        // violet should come before tango
        assert!(positions["violet"] < positions["tango"]);

        // uniform should come before sierra
        assert!(positions["uniform"] < positions["sierra"]);

        // tango should come before sierra
        assert!(positions["tango"] < positions["sierra"]);

        // sierra should come before romeo
        assert!(positions["sierra"] < positions["romeo"]);

        // romeo should come before quebec
        assert!(positions["romeo"] < positions["quebec"]);

        // Additional constraints from multi-dependencies
        assert!(positions["zebra"] < positions["uniform"]);
        assert!(positions["yak"] < positions["uniform"]);

        // All 10 components should be present
        assert_eq!(mgmt_func.components.len(), 10);

        // Print the final order for debugging
        //let final_order: Vec<String> = mgmt_func
        //    .components
        //    .iter()
        //    .map(|c| c.variable_name.clone())
        //    .collect();
        //println!("Final component order: {final_order:?}");
    }

    #[test]
    fn sorts_with_stable_ordering() {
        let mut components = Vec::new();

        // Create components with no dependencies in reverse alphabetical order
        // After stable sort, they should be in alphabetical order
        let attrs_z = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("z_value"),
        )];
        components.push(RunTemplateComponent::new(
            "zComponent",
            "AWS::Test",
            "z",
            attrs_z,
        ));

        let attrs_y = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("y_value"),
        )];
        components.push(RunTemplateComponent::new(
            "yComponent",
            "AWS::Test",
            "y",
            attrs_y,
        ));

        let attrs_x = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("x_value"),
        )];
        components.push(RunTemplateComponent::new(
            "xComponent",
            "AWS::Test",
            "x",
            attrs_x,
        ));

        // Add one more component that depends on all three to test stable ordering with dependencies
        let attrs_final = vec![
            RunTemplateAttribute::new(
                "/domain/source1",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("xComponent".to_string()),
                ),
            ),
            RunTemplateAttribute::new(
                "/domain/source2",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("yComponent".to_string()),
                ),
            ),
            RunTemplateAttribute::new(
                "/domain/source3",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("zComponent".to_string()),
                ),
            ),
        ];
        components.push(RunTemplateComponent::new(
            "finalComponent",
            "AWS::Test",
            "final",
            attrs_final,
        ));

        let mut mgmt_func = RunTemplateMgmtFunc::new(components);
        mgmt_func.sort_components_by_dependencies();

        // The first three should be in alphabetical order (stable sort)
        assert_eq!(mgmt_func.components[0].variable_name, "xComponent");
        assert_eq!(mgmt_func.components[1].variable_name, "yComponent");
        assert_eq!(mgmt_func.components[2].variable_name, "zComponent");
        assert_eq!(mgmt_func.components[3].variable_name, "finalComponent");

        // Test multiple runs produce same result (stability)
        let first_run: Vec<String> = mgmt_func
            .components
            .iter()
            .map(|c| c.variable_name.clone())
            .collect();

        // Run sort again with same components in different initial order
        let mut components2 = Vec::new();
        let attrs_x2 = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("x_value"),
        )];
        components2.push(RunTemplateComponent::new(
            "xComponent",
            "AWS::Test",
            "x",
            attrs_x2,
        ));

        let attrs_final2 = vec![
            RunTemplateAttribute::new(
                "/domain/source1",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("xComponent".to_string()),
                ),
            ),
            RunTemplateAttribute::new(
                "/domain/source2",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("yComponent".to_string()),
                ),
            ),
            RunTemplateAttribute::new(
                "/domain/source3",
                AttributeSource::subscription(
                    "comp",
                    "/domain/value",
                    None,
                    Some("zComponent".to_string()),
                ),
            ),
        ];
        components2.push(RunTemplateComponent::new(
            "finalComponent",
            "AWS::Test",
            "final",
            attrs_final2,
        ));

        let attrs_z2 = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("z_value"),
        )];
        components2.push(RunTemplateComponent::new(
            "zComponent",
            "AWS::Test",
            "z",
            attrs_z2,
        ));

        let attrs_y2 = vec![RunTemplateAttribute::new(
            "/domain/value",
            AttributeSource::value("y_value"),
        )];
        components2.push(RunTemplateComponent::new(
            "yComponent",
            "AWS::Test",
            "y",
            attrs_y2,
        ));

        let mut mgmt_func2 = RunTemplateMgmtFunc::new(components2);
        mgmt_func2.sort_components_by_dependencies();

        let second_run: Vec<String> = mgmt_func2
            .components
            .iter()
            .map(|c| c.variable_name.clone())
            .collect();

        // Both runs should produce identical results
        assert_eq!(first_run, second_run);
    }

    #[test]
    fn test_attributes_pruned_and_sorted() {
        let attributes = vec![
            // Add attributes in unsorted order
            RunTemplateAttribute::new(
                "/domain/Tags/0/f/lobster",
                AttributeSource::value("should_be_pruned"),
            ),
            RunTemplateAttribute::new("/domain/CidrBlock", AttributeSource::value("10.0.0.0/16")),
            RunTemplateAttribute::new(
                "/domain/Tags",
                AttributeSource::subscription(
                    "comp",
                    "/domain/Tags",
                    None,
                    Some("tagComponent".to_string()),
                ),
            ),
            RunTemplateAttribute::new("/domain/Tags/0", AttributeSource::value("should_be_pruned")),
            RunTemplateAttribute::new("/domain/Name", AttributeSource::value("test_name")),
            RunTemplateAttribute::new(
                "/domain/Other/Value",
                AttributeSource::subscription("comp", "/domain/Other/Value", None, None),
            ),
            RunTemplateAttribute::new(
                "/domain/Other/Value/Sub",
                AttributeSource::value("should_be_pruned_other"),
            ),
        ];

        let component =
            RunTemplateComponent::new("testComponent", "AWS::Test", "test component", attributes);

        let result = component.attributes_pruned_and_sorted();

        // Check that the result is sorted by dest_path
        let paths: Vec<String> = result.iter().map(|attr| attr.dest_path.clone()).collect();
        let mut expected_paths = paths.clone();
        expected_paths.sort();
        assert_eq!(
            paths, expected_paths,
            "Attributes should be sorted by dest_path"
        );

        // Check that descendant paths of subscriptions are pruned
        let result_paths: Vec<&str> = result.iter().map(|attr| attr.dest_path.as_str()).collect();

        // Should contain these paths
        assert!(
            result_paths.contains(&"/domain/CidrBlock"),
            "Should contain /domain/CidrBlock"
        );
        assert!(
            result_paths.contains(&"/domain/Name"),
            "Should contain /domain/Name"
        );
        assert!(
            result_paths.contains(&"/domain/Tags"),
            "Should contain /domain/Tags (subscription)"
        );
        assert!(
            result_paths.contains(&"/domain/Other/Value"),
            "Should contain /domain/Other/Value (subscription)"
        );

        // Should NOT contain these paths (they are descendants of subscription paths)
        assert!(
            !result_paths.contains(&"/domain/Tags/0"),
            "Should NOT contain /domain/Tags/0 (descendant of /domain/Tags)"
        );
        assert!(
            !result_paths.contains(&"/domain/Tags/0/f/lobster"),
            "Should NOT contain /domain/Tags/0/f/lobster (descendant of /domain/Tags)"
        );
        assert!(
            !result_paths.contains(&"/domain/Other/Value/Sub"),
            "Should NOT contain /domain/Other/Value/Sub (descendant of /domain/Other/Value)"
        );

        // Verify the exact count
        assert_eq!(
            result.len(),
            4,
            "Should have exactly 4 attributes after pruning"
        );
    }

    #[test]
    fn test_attributes_pruned_and_sorted_no_subscriptions() {
        let attributes = vec![
            // Add attributes with no subscriptions, in unsorted order
            RunTemplateAttribute::new("/domain/ZZZ", AttributeSource::value("last")),
            RunTemplateAttribute::new("/domain/AAA", AttributeSource::value("first")),
            RunTemplateAttribute::new("/domain/MMM", AttributeSource::value("middle")),
        ];

        let component =
            RunTemplateComponent::new("testComponent", "AWS::Test", "test component", attributes);

        let result = component.attributes_pruned_and_sorted();

        // Should be sorted and none should be pruned
        assert_eq!(result.len(), 3, "All attributes should be preserved");
        assert_eq!(result[0].dest_path, "/domain/AAA");
        assert_eq!(result[1].dest_path, "/domain/MMM");
        assert_eq!(result[2].dest_path, "/domain/ZZZ");
    }

    #[test]
    fn test_attributes_pruned_and_sorted_subscription_with_no_descendants() {
        let attributes = vec![
            RunTemplateAttribute::new(
                "/domain/Tags",
                AttributeSource::subscription("comp", "/domain/Tags", None, None),
            ),
            RunTemplateAttribute::new("/domain/Other", AttributeSource::value("other_value")),
        ];

        let component =
            RunTemplateComponent::new("testComponent", "AWS::Test", "test component", attributes);

        let result = component.attributes_pruned_and_sorted();

        // Both should be preserved since there are no descendants
        assert_eq!(result.len(), 2, "Both attributes should be preserved");
        assert_eq!(result[0].dest_path, "/domain/Other");
        assert_eq!(result[1].dest_path, "/domain/Tags");
    }
}
