use xtrmv::editor::Editor;
use xtrmv::row::Row;

fn create_test_editor(content: Vec<&str>) -> Editor {
    let rows: Vec<Row> = content.into_iter().map(|s| Row::new(s)).collect();
    Editor::new_for_test(rows)
}

#[test]
fn test_insert_char_at_end() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_col = 5;
    editor.insert_char('!');
    assert_eq!(editor.rows[0].characters, "hello!");
    assert_eq!(editor.cursor_col, 6);
    assert!(editor.modified);
}

#[test]
fn test_insert_char_at_start() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_col = 0;
    editor.insert_char('!');
    assert_eq!(editor.rows[0].characters, "!hello");
    assert_eq!(editor.cursor_col, 1);
}

#[test]
fn test_insert_char_in_middle() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_col = 2;
    editor.insert_char('x');
    assert_eq!(editor.rows[0].characters, "hexllo");
    assert_eq!(editor.cursor_col, 3);
}

#[test]
fn test_insert_char_in_empty_file() {
    let mut editor = create_test_editor(vec![]);
    editor.cursor_row = 0;
    editor.cursor_col = 0;
    editor.insert_char('a');
    assert_eq!(editor.rows.len(), 1);
    assert_eq!(editor.rows[0].characters, "a");
    assert_eq!(editor.cursor_col, 1);
}

#[test]
fn test_insert_char_beyond_row() {
    let mut editor = create_test_editor(vec!["hi"]);
    editor.cursor_row = 0;
    editor.cursor_col = 100; // Beyond row length
    editor.insert_char('x');
    assert_eq!(editor.rows[0].characters, "hix");
    assert_eq!(editor.cursor_col, 3);
}

#[test]
fn test_insert_new_line_at_start() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_col = 0;
    editor.insert_new_line();
    assert_eq!(editor.rows.len(), 2);
    assert_eq!(editor.rows[0].characters, "");
    assert_eq!(editor.rows[1].characters, "hello");
    assert_eq!(editor.cursor_row, 1);
    assert_eq!(editor.cursor_col, 0);
}

#[test]
fn test_insert_new_line_in_middle() {
    let mut editor = create_test_editor(vec!["hello world"]);
    editor.cursor_col = 5;
    editor.insert_new_line();
    assert_eq!(editor.rows.len(), 2);
    assert_eq!(editor.rows[0].characters, "hello");
    assert_eq!(editor.rows[1].characters, " world");
    assert_eq!(editor.cursor_row, 1);
    assert_eq!(editor.cursor_col, 0);
}

#[test]
fn test_insert_new_line_at_end() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_col = 5;
    editor.insert_new_line();
    assert_eq!(editor.rows.len(), 2);
    assert_eq!(editor.rows[0].characters, "hello");
    assert_eq!(editor.rows[1].characters, "");
    assert_eq!(editor.cursor_row, 1);
    assert_eq!(editor.cursor_col, 0);
}

#[test]
fn test_insert_new_line_beyond_rows() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_row = 5; // Beyond existing rows
    editor.insert_new_line();
    assert_eq!(editor.rows.len(), 2);
}

#[test]
fn test_delete_char_normal() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_col = 3;
    editor.delete_char();
    assert_eq!(editor.rows[0].characters, "helo");
    assert_eq!(editor.cursor_col, 2);
    assert!(editor.modified);
}

#[test]
fn test_delete_char_at_start() {
    let mut editor = create_test_editor(vec!["hello", "world"]);
    editor.cursor_row = 1;
    editor.cursor_col = 0;
    editor.delete_char();
    // Should merge with previous row
    assert_eq!(editor.rows.len(), 1);
    assert_eq!(editor.rows[0].characters, "helloworld");
    assert_eq!(editor.cursor_row, 0);
    assert_eq!(editor.cursor_col, 5);
}

#[test]
fn test_delete_char_at_file_start() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_col = 0;
    editor.delete_char();
    // Should do nothing
    assert_eq!(editor.rows[0].characters, "hello");
    assert_eq!(editor.cursor_col, 0);
}

#[test]
fn test_delete_char_empty_file() {
    let mut editor = create_test_editor(vec![]);
    editor.delete_char();
    // Should not panic
    assert_eq!(editor.rows.len(), 0);
}

#[test]
fn test_numrows() {
    let editor = create_test_editor(vec!["line1", "line2", "line3"]);
    assert_eq!(editor.numrows(), 3);
}

#[test]
fn test_numrows_empty() {
    let editor = create_test_editor(vec![]);
    assert_eq!(editor.numrows(), 0);
}

#[test]
fn test_rowlen() {
    let editor = create_test_editor(vec!["hello", "world!"]);
    assert_eq!(editor.rowlen(0), 5);
    assert_eq!(editor.rowlen(1), 6);
}

#[test]
fn test_rowlen_out_of_bounds() {
    let editor = create_test_editor(vec!["hello"]);
    assert_eq!(editor.rowlen(100), 0);
}

#[test]
fn test_modified_flag_set_on_insert() {
    let mut editor = create_test_editor(vec!["hello"]);
    assert!(!editor.modified);
    editor.insert_char('x');
    assert!(editor.modified);
}

#[test]
fn test_modified_flag_set_on_delete() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_col = 1;
    assert!(!editor.modified);
    editor.delete_char();
    assert!(editor.modified);
}

#[test]
fn test_insert_multiple_chars() {
    let mut editor = create_test_editor(vec![""]);
    editor.insert_char('h');
    editor.insert_char('i');
    editor.insert_char('!');
    assert_eq!(editor.rows[0].characters, "hi!");
    assert_eq!(editor.cursor_col, 3);
}
