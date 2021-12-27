use crate::{state::CompactState, Board};

pub fn visualize_state(board: &Board, state: &CompactState) {
    let first_row_char = "___";
    let last_row_char = "‾‾‾";
    let first_column_char = "|";
    let last_column_char = "|";
    let gold_char = "g";
    let silver_char = "s";

    let bottom_opposite = "↑";
    let top_opposite = "↓";
    let left_opposite = "→";
    let right_opposite = "←";

    let ball_char = "B";
    let falling_ball_char = "F";
    let blocked_char = "X";
    let open_char = "O";

    let shift_text_modifieds = [
        "",
        "\u{0332}",
        "\u{0332}\u{0305}",
        "\u{0332}\u{0305}\u{0336}",
    ];

    let mut result = "".to_owned();
    let mut first_row = " ".to_owned();
    let mut last_row = " ".to_owned();

    let ball_depth = state.depth();
    for layer in 0..4 {
        if board.layer_horizontal(layer) {
            first_row += first_row_char;
            last_row += last_row_char;
        } else {
            for gate in 0..3 {
                let s = board.gates_silver[layer as usize][gate as usize];
                let char = if s { silver_char } else { gold_char };
                let char_styling = shift_text_modifieds[state.get_shift(layer, gate) as usize];

                let char_owned = char.to_owned() + char_styling;
                let char = char_owned.as_str();
                let t = board.topleft(layer, gate);
                let (fc, lc) = if t {
                    (char, bottom_opposite)
                } else {
                    (top_opposite, char)
                };
                first_row += fc;
                last_row += lc;
            }
        }

        first_row += "   ";
        last_row += "   ";
    }
    for row in 0..3 {
        for layer in 0..4 {
            let row_bits = (state.get_gate_bits() >> (layer * 9 + row * 3)) & 0b111;
            let mut row_str = "".to_owned();

            let (first_char, last_char) = if board.layer_horizontal(layer) {
                let gate_silver = board.gates_silver[layer as usize][row as usize];
                let char = if gate_silver { silver_char } else { gold_char };

                if board.topleft(layer, row) {
                    (char, right_opposite)
                } else {
                    (left_opposite, char)
                }
            } else {
                (first_column_char, last_column_char)
            };
            for column in 0..3 {
                let cell_blocked = (row_bits >> column) & 1 == 0;
                let ball_present = ball_depth[(row * 3 + column) as usize] == layer;
                row_str += match (cell_blocked, ball_present) {
                    (true, true) => ball_char,
                    (true, false) => blocked_char,
                    (false, true) => falling_ball_char,
                    (false, false) => open_char,
                }
            }
            result = format!("{result}{first_char}{row_str}{last_char} ");
        }
        result += "\n"
    }
    println!("{}\n{}{}\n", first_row, result, last_row);
}
