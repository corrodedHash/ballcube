use crate::{state::Compact, Board, Player};

pub fn visualize_state(board: &Board, state: &Compact) {
    let first_row_char = "_";
    let last_row_char = "\u{203e}";
    let first_column_char = "|";
    let last_column_char = "|";
    let gold_char = "g";
    let silver_char = "s";

    let bottom_opposite = "\u{2191}";
    let top_opposite = "\u{2193}";
    let left_opposite = "\u{2192}";
    let right_opposite = "\u{2190}";

    // let corners_tl_tr_bl_br = ["┘", "└", "┐", "┌"];
    let corners_tl_tr_bl_br = [" ", " ", " ", " "];

    let ball_char_gold = "G";
    let ball_char_silver = "S";
    let falling_ball_char_gold = "F";
    let falling_ball_char_silver = "f";
    let blocked_char = "X";
    let open_char = "O";

    let shift_text_modifiers = [
        "",
        "\u{0332}",
        "\u{0332}\u{0305}",
        "\u{0332}\u{0305}\u{0336}",
    ];

    let mut result_lines: [String; 5] = Default::default();

    let ball_depth = state.depth();

    let get_side_char = |layer: u8, gate: u8| -> String {
        let owner_char = match board.gate(layer, gate) {
            crate::Player::Gold => gold_char,
            crate::Player::Silver => silver_char,
        };
        let char_styling = shift_text_modifiers[state.get_shift(layer, gate) as usize];
        format!("{owner_char}{char_styling}")
    };

    for layer in 0..4 {
        let mut first_column = [(); 3].map(|_| first_column_char.to_owned());
        let mut last_column = [(); 3].map(|_| last_column_char.to_owned());
        let mut first_row = [(); 3].map(|_| first_row_char.to_owned());
        let mut last_row = [(); 3].map(|_| last_row_char.to_owned());
        let mut field: [String; 9] = Default::default();

        let (tl_side, br_side, tl_opp, br_opp) = if board.layer_horizontal(layer) {
            (
                &mut first_column,
                &mut last_column,
                left_opposite,
                right_opposite,
            )
        } else {
            (&mut first_row, &mut last_row, top_opposite, bottom_opposite)
        };

        for (gate, (tl, br)) in (0_u8..).zip(tl_side.iter_mut().zip(br_side.iter_mut())) {
            if board.topleft(layer, gate) {
                *tl = get_side_char(layer, gate);
                *br = br_opp.to_owned();
            } else {
                *tl = tl_opp.to_owned();
                *br = get_side_char(layer, gate);
            }
        }

        for (id, (bd, cell)) in (0_u8..).zip(ball_depth.iter().zip(field.iter_mut())) {
            let ball_present = bd == &layer;
            let hole_present = state.get_gate_bits() & (1 << (9 * layer + id)) > 0;
            let ball_color = if ball_present { board.ball(id) } else { None };
            *cell = match (hole_present, ball_color) {
                (false, Some(Player::Gold)) => ball_char_gold,
                (false, Some(Player::Silver)) => ball_char_silver,
                (false, None) => blocked_char,
                (true, Some(Player::Gold)) => falling_ball_char_gold,
                (true, Some(Player::Silver)) => falling_ball_char_silver,
                (true, None) => open_char,
            }
            .to_owned();
        }

        result_lines[0] += &format!(
            "{}{}{} ",
            corners_tl_tr_bl_br[0],
            first_row.join(""),
            corners_tl_tr_bl_br[1]
        );
        for i in 0..3 {
            result_lines[i + 1] += &format!(
                "{}{}{} ",
                first_column[i],
                field[(i * 3)..((i + 1) * 3)].join(""),
                last_column[i]
            );
        }
        result_lines[4] += &format!(
            "{}{}{} ",
            corners_tl_tr_bl_br[2],
            last_row.join(""),
            corners_tl_tr_bl_br[3]
        );
    }
    println!("{}", result_lines.join("\n"));
}
