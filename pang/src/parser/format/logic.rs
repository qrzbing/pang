use super::*;

use crate::tree::decoder::ber_to_usize;

impl FormatParser {
    pub(super) fn parse_non_terminal_with_mode<'a>(
        &'a self,
        input: &'a [u8],
        label: &str,
        parent_context: &BTreeMap<String, Arc<DerivationTree>>,
        mode: ParseMode,
    ) -> IResult<&'a [u8], Vec<(&'a [u8], Arc<DerivationTree>)>> {
        let expansions = self
            .grammar
            .get(label)
            .ok_or_else(|| Err::Failure(ParseError::from_error_kind(input, ErrorKind::Fix)))?;

        // Collect all successful parses
        let mut successful_parses = Vec::new();

        // Try all possible expansions
        for expansion in expansions {
            if let Ok((remain_input, children)) =
                self.parse_expansion_with_mode(input, label, expansion, parent_context, mode)
            {
                let node = new_node(nt(label), Some(children), None);
                successful_parses.push((remain_input, node));

                // In Fast First mode, return the first successful parse.
                if matches!(mode, ParseMode::First) {
                    break;
                }
            }
        }

        if successful_parses.is_empty() {
            Err(Err::Error(ParseError::from_error_kind(
                input,
                ErrorKind::Alt,
            )))
        } else {
            Ok((input, successful_parses))
        }
    }

    fn parse_expansion_with_mode<'a>(
        &'a self,
        input: &'a [u8],
        label: &str,
        expansion: &Expansion,
        parent_context: &BTreeMap<String, Arc<DerivationTree>>,
        mode: ParseMode,
    ) -> IResult<&'a [u8], Vec<Arc<DerivationTree>>> {
        log::debug!(
            "-> Input: {}",
            input
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ")
        );
        log::debug!("-> Parsing label:{},  expansion: {:#?}", label, expansion);

        // If the expansion has a custom parser, use it.
        if let Some(parser_name) = expansion.options.get("parser").and_then(|v| v.as_str()) {
            if let Some(custom_parser) = self.custom_parsers.get(parser_name) {
                return custom_parser.parse(input, label, expansion, parent_context);
            }
        }

        // Handle `length_is` logic, maybe I should move this to a custom parser?
        if let Some(length_field_name) = expansion.options.get("length_is").and_then(|v| v.as_str())
        {
            // 1. Determine the length of the upcoming data slice
            let dynamic_size: usize = if let Some(val) = self.state.get::<usize>(length_field_name)
            {
                log::debug!("      |-- Found pre-parsed length in SharedState: {}", val);
                val
            } else if let Some(length_node) = parent_context.get(length_field_name) {
                log::debug!(
                    "      |-- Falling back to parsing length from context node '{}'",
                    length_field_name
                );
                let len_bytes = self.extract_value_from_node(length_node);
                match expansion
                    .options
                    .get("length_type")
                    .and_then(|v| v.as_str())
                {
                    // FIXME: Do not use unwrap here
                    Some("ber") => ber_to_usize(&len_bytes).unwrap().1,
                    _ => bytes_to_usize(&len_bytes, expansion.options.get("endian")),
                }
            } else {
                log::error!(
                    "`length_is` field '{}' not found in SharedState or parsing context",
                    length_field_name
                );
                return Err(Err::Failure(ParseError::from_error_kind(
                    input,
                    ErrorKind::Verify,
                )));
            };
            log::debug!("      |-- Final dynamic length is: {}", dynamic_size);

            // 2. Consume the data slice from the main input
            let (remaining_after_slice, data_slice) =
                nom::bytes::complete::take(dynamic_size)(input)?;

            if expansion.symbols.len() == 1
                && matches!(
                    &expansion.symbols[0],
                    Symbol::Terminal {
                        kind: TerminalKind::Binary(BinaryKind::Dynamic)
                    }
                )
            {
                log::debug!(
                    "      |-- Consuming raw slice of {} bytes for Dynamic terminal.",
                    data_slice.len()
                );
                let value_node = new_node(
                    expansion.symbols[0].clone(),
                    Some(vec![]),
                    Some(data_slice.to_vec()),
                );
                return Ok((remaining_after_slice, vec![value_node]));
            } else {
                log::debug!(
                    "      |-- Recursively parsing {} bytes for symbols: {:?}",
                    data_slice.len(),
                    expansion
                        .symbols
                        .iter()
                        .map(|s| s.display_symbol())
                        .collect::<Vec<_>>()
                );

                let mut current_input_for_slice = data_slice;
                let mut children_inside_slice = Vec::new();
                // The context for the recursive parse is fresh.
                let mut slice_context: BTreeMap<String, Arc<DerivationTree>> = BTreeMap::new();

                for symbol in &expansion.symbols {
                    let (next_slice_input, child_node) = self.parse_symbol_with_mode(
                        current_input_for_slice,
                        symbol,
                        expansion,
                        &slice_context, // Use the new, inner context
                        mode,
                    )?;

                    if let Symbol::NonTerminal { label } = &symbol {
                        slice_context.insert(label.clone(), child_node.clone());
                    }
                    current_input_for_slice = next_slice_input;
                    children_inside_slice.push(child_node);
                }

                // 4. Ensure the entire data slice was consumed
                if !current_input_for_slice.is_empty() {
                    log::warn!(
                        "      |-- Parser did not consume the entire length-prefixed slice. {} bytes remain.",
                        current_input_for_slice.len()
                    );
                    return Err(Err::Error(ParseError::from_error_kind(
                        data_slice,
                        ErrorKind::Eof,
                    )));
                }

                log::debug!("      |-- Successfully parsed all symbols inside the slice.");
                // 5. Return the parsed children and the input *after* the slice
                return Ok((remaining_after_slice, children_inside_slice));
            }
        }

        let mut current_input = input;
        let mut children = Vec::new();
        let mut context: BTreeMap<String, Arc<DerivationTree>> = parent_context.clone();

        for symbol in &expansion.symbols {
            log::debug!(
                "  |-- Attempting to parse symbol: {}",
                symbol.display_symbol()
            );
            log::debug!("  |   |-- Context so far: {:?}", context.keys());

            let (next_input, child_node) =
                self.parse_symbol_with_mode(current_input, symbol, expansion, &context, mode)?;

            log::debug!(
                "  |   +-- SUCCESS parsing symbol: {}",
                child_node.symbol.display_symbol()
            );

            log::debug!(
                "  |   +-- symbol: {} | value: {}",
                child_node.symbol.display_symbol(),
                &input[0..current_input.len() - next_input.len()]
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
            );

            current_input = next_input;
            children.push(child_node);

            if let Symbol::NonTerminal { label } = &symbol {
                log::debug!("  |   '-> ADDING to context: key='{}'", label);
                context.insert(label.clone(), Arc::clone(children.last().unwrap()));
            }
        }
        log::debug!("<- SUCCESS parsing expansion.");
        Ok((current_input, children))
    }

    fn parse_symbol_with_mode<'a>(
        &'a self,
        input: &'a [u8],
        symbol: &Symbol,
        expansion: &Expansion,
        context: &BTreeMap<String, Arc<DerivationTree>>,
        mode: ParseMode,
    ) -> IResult<&'a [u8], Arc<DerivationTree>> {
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

                            best_parse.ok_or(Err::Error(ParseError::from_error_kind(
                                input,
                                ErrorKind::Verify,
                            )))
                        }
                    }
                })
            }
            Symbol::Terminal { kind } => {
                self.parse_terminal(input, kind, symbol, expansion, context)
            }
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
