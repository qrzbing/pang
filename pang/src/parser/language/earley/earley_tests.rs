use super::*;
use crate::grammar::{c_sample_grammar, xml_grammar};

#[test]
fn test_state_advance() {
    let grammar = c_sample_grammar();
    let item_name = "B";
    let expr = grammar.get(item_name).unwrap()[1].clone();
    let a_state = State::new(item_name, &expr, 0, 0);
    assert_eq!(a_state.at_dot().unwrap(), &nt("D"));

    let another_state = a_state.advance();
    assert_eq!(another_state.is_finished(), true);
}

#[test]
fn test_state_at_dot() {
    let grammar = c_sample_grammar();

    let mut col_0 = Column::new(0, None);
    let start_symbol = "start";
    let expr = grammar.get(start_symbol).unwrap()[0].clone();

    let state_to_add = State::new(start_symbol, &expr, 0, 0);
    col_0.add(state_to_add);

    let label_to_process: String;

    let start_state_in_col = &col_0.states[0];

    let symbol_at_dot = start_state_in_col
        .at_dot()
        .expect("Should have a symbol at dot");

    assert_eq!(symbol_at_dot, &nt("A"));

    if let Symbol::NonTerminal { label } = symbol_at_dot {
        label_to_process = label.clone();
    } else {
        panic!("Expected a NonTerminal symbol");
    }

    for alt_expr in grammar.get(&label_to_process).unwrap() {
        col_0.add(State::new(&label_to_process, alt_expr, 0, 0));
    }

    println!("{}", col_0);
    assert_eq!(col_0.states.len(), 3);
}

#[test]
fn test_predict() {
    let grammar = c_sample_grammar();

    let mut col_0 = Column::new(0, None);
    let start_symbol = "start";
    let expr = grammar.get(start_symbol).unwrap()[0].clone();

    let state_to_add = State::new(start_symbol, &expr, 0, 0);
    col_0.add(state_to_add);

    let ep = EarleyParser::new(grammar, "start", HashSet::new(), true);

    let mut chart = vec![col_0];
    let col_idx = 0;

    let non_terminal_to_predict = {
        let initial_state = &chart[col_idx].states[0];
        initial_state.at_dot().unwrap().label().to_string()
    };

    ep.predict(&mut chart, col_idx, &non_terminal_to_predict);
    for s in &chart[0].states {
        println!("{}", s);
    }
    assert_eq!(chart[0].states.len(), 3);
}

#[test]
fn test_scan() {
    let grammar = c_sample_grammar();
    let ep = EarleyParser::new(grammar.clone(), "start", HashSet::new(), true);

    let mut col_0 = Column::new(0, None);
    let start_expr = grammar.get("start").unwrap()[0].clone();
    col_0.add(State::new("start", &start_expr, 0, 0));
    let a_expr_1 = grammar.get("A").unwrap()[0].clone(); // <A> := a <B> c
    let a_expr_2 = grammar.get("A").unwrap()[1].clone(); // <A> := a <A>
    col_0.add(State::new("A", &a_expr_1, 0, 0));
    col_0.add(State::new("A", &a_expr_2, 0, 0));

    let col_1 = Column::new(1, Some(b'a'));

    let mut chart = vec![col_0, col_1];

    let col_idx = 0;
    let text = b"a";

    let state_to_scan = chart[col_idx].states[1].clone();
    let token_to_scan = state_to_scan.at_dot().unwrap().value();

    println!("--- Before scan ---");
    println!("State to scan: {}", state_to_scan);
    println!(
        "Token to scan: {:?}",
        std::str::from_utf8(token_to_scan).unwrap()
    );

    ep.scan(&mut chart, col_idx, &state_to_scan, token_to_scan, text);

    println!("Column 1 state:\n{}", chart[1]);
}

#[test]
fn test_earley_chart_parse() {
    let grammar = c_sample_grammar();
    let ep = EarleyParser::new(grammar, "start", HashSet::new(), true);

    let columns = ep.chart_parse(b"adcd");
    for col in &columns {
        println!("Column: {}", col);
    }
}

#[test]
fn test_earley_parse_prefix() {
    let grammar = c_sample_grammar();
    let ep = EarleyParser::new(grammar, "start", HashSet::new(), false);
    let (cursor, last_states) = ep.parse_prefix(b"abcd");
    println!("Cursor: {}, last states: {:?}", cursor, last_states);
}

#[test]
fn test_earley_chart_parse_for_invalid_input() {
    let input = b"<html><body><i>World</i><br/>>/body></html>";
    let grammar = xml_grammar();
    let xml_tokens = HashSet::from(["id".to_string(), "text".to_string()]);
    let ep = EarleyParser::new(grammar, "start", xml_tokens, true);
    let table = ep.chart_parse(input);
    for column in &table {
        println!("{}", column);
        println!("---")
    }
    let mut cols = Vec::new();
    for col in &table {
        if !col.states.is_empty() {
            cols.push(col);
        }
    }
    let parsable = &input[..cols.len()];
    // println!("Parsable: {}", String::from_utf8_lossy(parsable));
    assert_eq!(parsable, b"<html><body><i>World</i><br/>");
}

#[test]
fn test_earley_parse_region() {
    let input = b"<html><body><i>World</i><br/>>/body></html>";
    let grammar = xml_grammar();
    let xml_tokens = HashSet::from(["id".to_string(), "text".to_string()]);
    let ep = EarleyParser::new(grammar, "start", xml_tokens, true);
    let regions = ep.parse_regions(input).unwrap();
    for (name, regions) in regions {
        println!("{}: {}", name, regions.len());
        for region in regions {
            println!(
                "  {}-{}: {}",
                region.start,
                region.end,
                String::from_utf8_lossy(&input[region.start..region.end])
            );
        }
    }
}
