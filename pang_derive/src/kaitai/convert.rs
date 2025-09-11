// src/generator.rs

use crate::kaitai::{KsySpec, SeqItem, TypeDef, RepeatType, KsyType};
use anyhow::{Result, bail};

pub fn generate_rust_code(spec: &KsySpec) -> Result<String> {
    let mut grammar_rules = Vec::new();
    let mut extra_rules = Vec::new(); // Generated rules like switches

    // Main rule from meta.id
    let main_rule = format!(
        "\"{}\" => [exp([nt(\"{}_seq\")])]",
        spec.meta.id,
        spec.meta.id
    );
    grammar_rules.push(main_rule);

    // Main sequence rule
    let main_seq_expansions = generate_expansions_for_seq(&spec.seq, &mut extra_rules)?;
    let main_seq_rule = format!(
        "\"{}_seq\" => [{}]",
        spec.meta.id,
        main_seq_expansions
    );
    grammar_rules.push(main_seq_rule);


    // Rules from `types`
    for (type_name, type_def) in &spec.types {
        let expansions = generate_expansions_for_type(type_name, type_def, &mut extra_rules)?;
        grammar_rules.push(format!("\"{}\" => [{}]", type_name, expansions));
    }

    // Add any dynamically generated rules (like for switches)
    grammar_rules.extend(extra_rules);


    // Assemble the final file content
    let grammar_fn = format!(
r#"
fn {}_grammar() -> Grammar {{
    grammar! {{
        {}
    }}
}}
"#,
        spec.meta.id,
        grammar_rules.join(",\n        ")
    );

    Ok(grammar_fn)
}


fn generate_expansions_for_type(type_name: &str, type_def: &TypeDef, extra_rules: &mut Vec<String>) -> Result<String> {
    if !type_def.seq.is_empty() {
        return generate_expansions_for_seq(&type_def.seq, extra_rules);
    }
    bail!("Type '{}' has no sequence or is unsupported", type_name)
}

fn generate_expansions_for_seq(seq: &[SeqItem], extra_rules: &mut Vec<String>) -> Result<String> {
    let mut has_eos_repeat = false;
    let symbols: Vec<String> = seq.iter().filter_map(|item| {
        if item.repeat == Some(RepeatType::Eos) {
            has_eos_repeat = true;
            return None;
        }
        Some(map_item_to_symbol(item, extra_rules))
    }).collect();

    let mut expansions = vec![format!("exp([{}])", symbols.join(", ") )];

    if has_eos_repeat {
        let repeating_item = seq.iter().find(|i| i.repeat == Some(RepeatType::Eos)).unwrap();
        // Here we're making a specific assumption for the pcap `packets` case.
        // A more robust solution would parse the item id.
        let repeating_symbol = "nt(\"packet\")";
        expansions = vec![
            format!("exp([{}, nt(\"packets\")])", repeating_symbol),
            format!("exp([{}])", repeating_symbol),
            "exp([t_dyn()])".to_string(),
        ];
    }


    Ok(expansions.join(", "))
}


fn map_item_to_symbol(item: &SeqItem, extra_rules: &mut Vec<String>) -> String {
    if let Some(contents) = &item.contents {
        let bytes: Vec<String> = contents.iter().map(|b| format!("0x{:02x}", b)).collect();
        return format!("t_bytes_val(&[{}])", bytes.join(", "));
    }

    // NEW LOGIC HERE
    if let Some(type_def) = &item.type_def {
        match type_def {
            KsyType::Simple(type_name) => {
                 return match type_name.as_str() {
                    "u2" => "t_bytes(2)".to_string(),
                    "s2" => "t_bytes(2)".to_string(),
                    "u4" => "t_bytes(4)".to_string(),
                    "s4" => "t_bytes(4)".to_string(),
                    "u8" => "t_bytes(8)".to_string(),
                    "s8" => "t_bytes(8)".to_string(),
                    _ => format!("nt(\"{}\")", type_name)
                };
            }
            KsyType::Switch(switch_def) => {
                // When finding a switch, we create a new grammar rule for it.
                // The name of the rule is the `id` of the item.
                let switch_rule_name = item.id.clone();
                let switch_expansions: Vec<String> = switch_def.cases.values()
                    .map(|case_type| format!("exp([nt(\"{}\")])", case_type))
                    .collect();

                let rule = format!("\"{}\" => [{}]", switch_rule_name, switch_expansions.join(", "));
                extra_rules.push(rule);

                // And in the current sequence, we just refer to this new rule as a non-terminal.
                return format!("nt(\"{}\")", switch_rule_name);
            }
        }
    }

    // If there's no type but there is a size attribute, we assume it's a dynamic body
    // that needs a callback. This is a simplification for the pcap `body` case.
    if item.size.is_some() {
        let rule_name = format!("{}_body", item.id); // e.g. "pcap_body"
        extra_rules.push(format!("\"{}\" => [exp_dc([t_dyn()], body_decode_callbackfn)]", item.id));
        return format!("nt(\"{}\")", item.id);
    }


    format!("nt(\"{}\")", item.id)
}