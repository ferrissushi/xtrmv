use xtrmv::row::Row;

#[test]
fn test_row_insert_char_at_end() {
    let mut row = Row::new("hello");
    row.insert_char(5, '!');
    assert_eq!(row.characters, "hello!");
}

#[test]
fn test_row_insert_char_at_start() {
    let mut row = Row::new("hello");
    row.insert_char(0, '!');
    assert_eq!(row.characters, "!hello");
}

#[test]
fn test_row_insert_char_in_middle() {
    let mut row = Row::new("hello");
    row.insert_char(2, 'x');
    assert_eq!(row.characters, "hexllo");
}

#[test]
fn test_row_delete_char_at_start() {
    let mut row = Row::new("hello");
    row.delete_char(0);
    assert_eq!(row.characters, "ello");
}

#[test]
fn test_row_delete_char_at_end() {
    let mut row = Row::new("hello");
    row.delete_char(4);
    assert_eq!(row.characters, "hell");
}

#[test]
fn test_row_delete_char_in_middle() {
    let mut row = Row::new("hello");
    row.delete_char(2);
    assert_eq!(row.characters, "helo");
}

#[test]
fn test_row_delete_char_out_of_bounds() {
    let mut row = Row::new("hello");
    row.delete_char(10); // Should not panic
    assert_eq!(row.characters, "hello");
}

#[test]
fn test_row_truncate() {
    let mut row = Row::new("hello world");
    let removed = row.truncate(5);
    assert_eq!(row.characters, "hello");
    assert_eq!(removed, " world");
}

#[test]
fn test_row_truncate_at_start() {
    let mut row = Row::new("hello");
    let removed = row.truncate(0);
    assert_eq!(row.characters, "");
    assert_eq!(removed, "hello");
}

#[test]
fn test_row_truncate_at_end() {
    let mut row = Row::new("hello");
    let removed = row.truncate(5);
    assert_eq!(row.characters, "hello");
    assert_eq!(removed, "");
}

#[test]
fn test_row_append_str() {
    let mut row = Row::new("hello");
    row.append_str(" world");
    assert_eq!(row.characters, "hello world");
}

#[test]
fn test_row_cx_to_rx_simple() {
    let row = Row::new("hello");
    assert_eq!(row.cx_to_rx(0), 0);
    assert_eq!(row.cx_to_rx(3), 3);
    assert_eq!(row.cx_to_rx(5), 5);
}

#[test]
fn test_row_new_empty() {
    let row = Row::new("");
    assert_eq!(row.characters, "");
    assert_eq!(row.render, "");
}

#[test]
fn test_row_render_with_tabs() {
    let row = Row::new("hello\tworld");
    // TAB_STOP is 8, so tab expands to 8 spaces
    assert!(row.render.contains(" "));
    assert!(row.render.contains("hello"));
    assert!(row.render.contains("world"));
}
