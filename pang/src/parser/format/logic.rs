use super::*;

use crate::symbol::{DecodeError, DecodeResult, Symbol, nt};

impl FormatParser {
    pub(super) fn parse_non_terminal_with_mode<'a>(
        &'a self,
        input: &'a [u8],
        label: &str,
        parent_context: &BTreeMap<String, Arc<DerivationTree>>,
        mode: ParseMode,
    ) -> DecodeResult<'a, Vec<(&'a [u8], Arc<DerivationTree>)>> {
        let expansions = self.grammar.get(label).expect("No Expansion found.");

        // Collect all successful parses
        let mut successful_parses = Vec::new();

        // Try all possible expansions
        for expansion in expansions {
            if let Ok((remain_input, children)) =
                self.parse_expansion_with_mode(input, label, expansion, parent_context, mode)
            {
                let node = new_node(nt(label), Some(children));
                successful_parses.push((remain_input, node));

                // In Fast First mode, return the first successful parse.
                if matches!(mode, ParseMode::First) {
                    break;
                }
            }
        }

        if successful_parses.is_empty() {
            Err(DecodeError::Incomplete)
        } else {
            Ok((input, successful_parses))
        }
    }

    fn parse_expansion_with_mode<'a>(
        &'a self,
        input: &'a [u8],
        _label: &str,
        expansion: &Expansion,
        parent_context: &BTreeMap<String, Arc<DerivationTree>>,
        mode: ParseMode,
    ) -> DecodeResult<'a, Vec<Arc<DerivationTree>>> {
        let mut current_input = input;
        let mut children = Vec::new();

        let mut context = parent_context.clone();

        for symbol in &expansion.symbols {
            let (next_input, child_node) =
                self.parse_symbol_with_mode(current_input, symbol, &context, mode)?;

            current_input = next_input;
            if let Symbol::NonTerminal { label } = &child_node.symbol {
                context.insert(label.clone(), child_node.clone());
            } else if let Symbol::NonTerminal { label } = &symbol {
                context.insert(label.clone(), child_node.clone());
            }
            children.push(child_node);
        }
        Ok((current_input, children))
    }

    fn parse_symbol_with_mode<'a>(
        &'a self,
        input: &'a [u8],
        symbol: &Symbol,
        context: &BTreeMap<String, Arc<DerivationTree>>,
        mode: ParseMode,
    ) -> DecodeResult<'a, Arc<DerivationTree>> {
        // Default logic
        match symbol {
            Symbol::NonTerminal { label } => {
                let start_ptr = input.as_ptr() as usize;
                let result = self.parse_non_terminal_with_mode(input, label, context, mode);
                if mode == ParseMode::Region {
                    if let Ok((_remaining_input, trees)) = &result {
                        if let Some((rem, _tree)) = trees.first() {
                            let consumed_len = input.len() - rem.len();
                            if consumed_len > 0 {
                                let original_ptr = *self.original_input_ptr.lock().unwrap();
                                let start_offset = start_ptr - original_ptr;
                                let end_offset = start_offset + consumed_len;
                                if end_offset - start_offset > 1 {
                                    let region = Region {
                                        start: start_offset,
                                        end: end_offset,
                                    };
                                    self.regions
                                        .lock()
                                        .unwrap()
                                        .entry(label.clone())
                                        .or_default()
                                        .insert(region);
                                }
                            }
                        }
                    }
                }
                result.and_then(|(_input, parses)| {
                    match mode {
                        ParseMode::First | ParseMode::Region => {
                            // The first successful parse is the result.
                            let (remaining_input, tree) = &parses[0];
                            Ok((*remaining_input, Arc::clone(tree)))
                        }
                        ParseMode::Forest => {
                            // Find the parse that consumed the most input (left the least).
                            let mut best_parse = None;
                            let mut min_remaining = input.len();

                            for (rem, tree) in parses {
                                if rem.len() <= min_remaining {
                                    min_remaining = rem.len();
                                    best_parse = Some((rem, tree));
                                }
                            }

                            best_parse.ok_or(DecodeError::Incomplete)
                        }
                    }
                })
            }
            Symbol::Terminal { kind } => self.parse_terminal(input, kind, symbol, context),
        }
    }

    pub(super) fn parse_and_collect_regions(
        &self,
        text: &[u8],
    ) -> Result<HashMap<String, HashSet<Region>>, String> {
        self.regions.lock().unwrap().clear();
        *self.original_input_ptr.lock().unwrap() = text.as_ptr() as usize;

        let _ = self.parse_non_terminal_with_mode(
            text,
            &self.start_symbol,
            &BTreeMap::new(),
            ParseMode::Region,
        );

        Ok(self.regions.lock().unwrap().drain().collect())
    }
}
