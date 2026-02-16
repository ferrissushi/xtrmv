use xtrmv::editor::Editor;
use xtrmv::row::Row;

fn create_test_editor(content: Vec<&str>) -> Editor {
    let rows: Vec<Row> = content.into_iter().map(|s| Row::new(s)).collect();
    Editor::new_for_test(rows)
}

#[test]
fn test_cursor_move_left_at_start() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_col = 0;
    editor.move_cursor_left();
    assert_eq!(editor.cursor_col, 0);
    assert_eq!(editor.cursor_row, 0);
}

#[test]
fn test_cursor_move_left_from_beginning_of_line() {
    let mut editor = create_test_editor(vec!["hello", "world"]);
    editor.cursor_row = 1;
    editor.cursor_col = 0;
    editor.move_cursor_left();
    assert_eq!(editor.cursor_row, 0);
    assert_eq!(editor.cursor_col, 5); // End of "hello"
}

#[test]
fn test_cursor_move_left_normal() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_col = 3;
    editor.move_cursor_left();
    assert_eq!(editor.cursor_col, 2);
}

#[test]
fn test_cursor_move_right_at_end_of_line() {
    let mut editor = create_test_editor(vec!["hello", "world"]);
    editor.cursor_row = 0;
    editor.cursor_col = 5; // End of "hello"
    editor.move_cursor_right();
    assert_eq!(editor.cursor_row, 1);
    assert_eq!(editor.cursor_col, 0);
}

#[test]
fn test_cursor_move_right_at_end_of_file() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_row = 0;
    editor.cursor_col = 5;
    editor.move_cursor_right();
    // Should stay at end of last row
    assert_eq!(editor.cursor_row, 0);
    assert_eq!(editor.cursor_col, 5);
}

#[test]
fn test_cursor_move_right_normal() {
    let mut editor = create_test_editor(vec!["hello"]);
    editor.cursor_col = 2;
    editor.move_cursor_right();
    assert_eq!(editor.cursor_col, 3);
}

#[test]
fn test_cursor_move_up_at_start() {
    let mut editor = create_test_editor(vec!["hello", "world"]);
    editor.cursor_row = 0;
    editor.move_cursor_up();
    assert_eq!(editor.cursor_row, 0);
}

#[test]
fn test_cursor_move_up_normal() {
    let mut editor = create_test_editor(vec!["hello", "world"]);
    editor.cursor_row = 1;
    editor.move_cursor_up();
    assert_eq!(editor.cursor_row, 0);
}

#[test]
fn test_cursor_move_down_at_end() {
    let mut editor = create_test_editor(vec!["hello", "world"]);
    editor.cursor_row = 1;
    editor.move_cursor_down();
    assert_eq!(editor.cursor_row, 1);
}

#[test]
fn test_cursor_move_down_normal() {
    let mut editor = create_test_editor(vec!["hello", "world"]);
    editor.cursor_row = 0;
    editor.move_cursor_down();
    assert_eq!(editor.cursor_row, 1);
}

#[test]
fn test_cursor_move_down_empty_file() {
    let mut editor = create_test_editor(vec![]);
    editor.move_cursor_down();
    assert_eq!(editor.cursor_row, 0);
}

#[test]
fn test_cursor_bounds_after_rowlen_check() {
    let mut editor = create_test_editor(vec!["hi", "hello world"]);
    editor.cursor_row = 1;
    editor.cursor_col = 20; // Beyond row length
    let rowlen = editor.rowlen(1);
    assert_eq!(rowlen, 11);
    if editor.cursor_col > rowlen {
        editor.cursor_col = rowlen;
    }
    assert_eq!(editor.cursor_col, 11);
}
