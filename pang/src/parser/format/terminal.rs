use log::debug;

use crate::symbol::{DecodeResult, Symbol, terminals::TerminalKind};

use super::*;

impl FormatParser {
    pub(super) fn parse_terminal<'a>(
        &self,
        input: &'a [u8],
        kind: &Arc<dyn TerminalKind>,
        _symbol: &Symbol,
        context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<DerivationTree>> {
        debug!("Parsing input: {:?}", input);

        let (remaining_input, new_kind) = kind.parse(input, &self.state, context)?;

        let new_symbol = Symbol::Terminal { kind: new_kind };

        Ok((remaining_input, new_node(new_symbol, Some(vec![]))))
    }
}
