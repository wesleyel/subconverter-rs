//! Group generation utilities
//!
//! This module provides functionality for generating proxy groups.

use crate::{
    models::ExtraSettings,
    utils::{
        matcher::{apply_matcher, reg_find},
        starts_with,
    },
    Proxy,
};

/// Generates a filtered list of nodes based on a rule and node list
///
/// # Arguments
///
/// * `rule` - The rule to apply to filter nodes
/// * `nodelist` - List of all available proxy nodes
/// * `filtered_nodelist` - Output parameter that will contain the filtered node list
/// * `add_direct` - Whether to add direct connection to the list
/// * `ext` - Extra settings
///
/// # Returns
///
/// Nothing, modifies filtered_nodelist in-place
pub fn group_generate(
    rule: &str,
    nodelist: &[Proxy],
    filtered_nodelist: &mut Vec<String>,
    add_direct: bool,
    ext: &ExtraSettings,
) {
    // Clear the output list first
    filtered_nodelist.clear();

    let real_rule = String::new();
    // Rule parsing
    if starts_with(rule, "[]") && add_direct {
        filtered_nodelist.push(rule[2..].to_string());
    } else if starts_with(&real_rule, "script:") && ext.authorized {
        // TODO: javascript
    } else {
        // Include only nodes that match the rule
        for node in nodelist {
            let mut real_rule_str = String::new();
            if apply_matcher(rule, &mut real_rule_str, node)
                && (!real_rule_str.is_empty()
                    || reg_find(&node.remark, &real_rule_str)
                        && filtered_nodelist.contains(&node.remark))
            {
                filtered_nodelist.push(node.remark.clone());
            }
        }
    }
}
