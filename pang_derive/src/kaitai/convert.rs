// src/generator.rs

use crate::kaitai::{KsySpec, KsyType, RepeatType, SeqItem, TypeDef};
use anyhow::{Result, bail};
use log::debug;

pub fn generate_rust_code(spec: &KsySpec) -> Result<String> {
    let mut grammar_rules = Vec::new();
    let mut extra_rules = Vec::new(); // Generated rules like switches

    // Main rule from meta.id
    let main_seq_symbols: Vec<String> = spec
        .seq
        .iter()
        .map(|item| format!("nt(\"{}\")", item.id))
        .collect();

    let main_rule = format!(
        "\"{}\" => [exp([{}])]",
        spec.meta.id,
        main_seq_symbols.join(", ")
    );

    debug!("main rule {}", main_rule);

    grammar_rules.push(main_rule);

    // Create rules for each item in the top-level `seq`
    for item in &spec.seq {
        if item.repeat == Some(RepeatType::Eos) {
            // This is the 'packets' case from pcap.ksy
            let type_name = if let Some(KsyType::Simple(name)) = &item.type_def {
                name
            } else {
                bail!("Item with repeat:eos must have a simple type.");
            };
            let rule = format!(
                "\"{}\" => [exp([nt(\"{}\"), nt(\"{}\")]), exp([nt(\"{}\")]), exp([t_dyn()])]",
                item.id, type_name, type_name, item.id
            );
            debug!("eos rule {}", rule);
            grammar_rules.push(rule);
        } else {
            // This is the 'hdr' case from pcap.ksy
            let type_name = if let Some(KsyType::Simple(name)) = &item.type_def {
                name
            } else {
                bail!("Seq item must have a simple type");
            };
            let rule = format!("\"{}\" => [exp([nt(\"{}\")])]", item.id, type_name);
            debug!("simple rule {}", rule);
            grammar_rules.push(rule);
        }
    }

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

fn generate_expansions_for_type(
    type_name: &str,
    type_def: &TypeDef,
    extra_rules: &mut Vec<String>,
) -> Result<String> {
    if !type_def.seq.is_empty() {
        return generate_expansions_for_seq(&type_def.seq, extra_rules);
    }
    bail!("Type '{}' has no sequence or is unsupported", type_name)
}

fn generate_expansions_for_seq(seq: &[SeqItem], extra_rules: &mut Vec<String>) -> Result<String> {
    let symbols: Vec<String> = seq
        .iter()
        .map(|item| map_item_to_symbol(item, extra_rules))
        .collect();

    let expansion = format!("exp([{}])", symbols.join(", "));

    Ok(expansion)
}

fn map_item_to_symbol(item: &SeqItem, extra_rules: &mut Vec<String>) -> String {
    // Convert bytes values to a `tl_bytes_val`
    if let Some(contents) = &item.contents {
        let bytes: Vec<String> = contents.iter().map(|b| format!("0x{:02x}", b)).collect();
        return format!("tl_bytes_val(\"{}\", &[{}])", item.id, bytes.join(", "));
    }

    if let Some(type_def) = &item.type_def {
        match type_def {
            KsyType::Simple(type_name) => {
                return match type_name.as_str() {
                    "u2" | "s2" => format!("tl_bytes(\"{}\", 2)", item.id),
                    "u4" | "s4" => format!("tl_bytes(\"{}\", 4)", item.id),
                    "u8" | "s8" => format!("tl_bytes(\"{}\", 8)", item.id),
                    _ => format!("nt(\"{}\")", type_name),
                };
            }
            KsyType::Switch(switch_def) => {
                // When finding a switch, we create a new grammar rule for it.
                // The name of the rule is the `id` of the item.
                let switch_rule_name = item.id.clone();
                let switch_expansions: Vec<String> = switch_def
                    .cases
                    .values()
                    .map(|case_type| format!("exp([nt(\"{}\")])", case_type))
                    .collect();

                let rule = format!(
                    "\"{}\" => [{}]",
                    switch_rule_name,
                    switch_expansions.join(", ")
                );
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
        extra_rules.push(format!(
            "\"{}\" => [exp_dc([t_dyn()], body_decode_callbackfn)]",
            item.id
        ));
        return format!("nt(\"{}\")", item.id);
    }

    format!("nt(\"{}\")", item.id)
}
