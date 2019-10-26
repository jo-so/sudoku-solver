#![feature(vec_remove_item)]
use std::{
    env,
    io::{self, Read},
    process,
};

mod sudoku;
use sudoku::{Board, Field};

fn main() {
    println!("Input initial board setting (space and newline are ignored, \
              non-digit charaters define empty fields)\n");

    let mut buffer = String::new();
    {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        if let Err(err) = handle.read_to_string(&mut buffer) {
            eprintln!("Failed to read data: {}", err);
            process::exit(1);
        }
    }

    let mut board = board_from_string(&buffer);
    drop(buffer);

    if env::args().nth(1).map_or(false, |x| x == "-s") {
        board.record_steps(true);
    }

    board.solve();

    print!("\nSolution:\n\n  ");
    let mut first = true;
    for (i, e) in board.fields().iter().enumerate() {
        if first {
            first = false;
        } else if i % 27 == 0 {
            print!("\n\n  ");
        } else if i % 9 == 0 {
            print!("\n  ");
        } else if i % 3 == 0 {
            print!("   ");
        }

        match e {
            Field::Options(_) => print!("."),
            Field::Value(v) => print!("{}", v),
        }
    }
    println!();

    if let Some(steps) = board.steps() {
        println!("\nSteps:");
        for (i, (idx, val)) in steps.iter().enumerate() {
            println!("  {:2}. ({}, {}) = {}", i + 1, (idx / 9) + 1, (idx % 9) + 1, val);
        }
    }
}

fn board_from_string(data: &str) -> Board {
    let mut board = Board::new();
    board.fill(
        data.chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_digit(10).map(|x| x as u8))
    );
    board
}

#[cfg(test)]
mod tests {
    use super::*;
    use sudoku::Field;

    fn to_string(board: &Board) -> String {
        board.fields().iter().map(|x| match x {
            Field::Options(_) => '_',
            Field::Value(v) => (v + 0x30).into(),
        }).collect::<String>()
    }

    const TEST_BOARD : &str =
        "92_______\
         5__87____\
         _38_91___\
         _5293_16_\
         _9_____3_\
         _73_6498_\
         ___41_25_\
         ____53__1\
         _______73";

    #[test]
    fn fill_board_simple() {
        let board = board_from_string(TEST_BOARD);

        assert_eq!(to_string(&board), TEST_BOARD);
    }

    #[test]
    fn fill_board_placeholder_x() {
        let board = board_from_string(
            "92xxxxxxx\
             5xx87xxxx\
             x38x91xxx\
             x5293x16x\
             x9xxxxx3x\
             x73x6498x\
             xxx41x25x\
             xxxx53xx1\
             xxxxxxx73"
        );

        assert_eq!(to_string(&board), TEST_BOARD);
    }

    #[test]
    fn fill_board_newline() {
        let board = board_from_string(
            "92_______\n\
             5__87____\n\
             _38_91___\n\
             _5293_16_\n\
             _9_____3_\n\
             _73_6498_\n\
             ___41_25_\n\
             ____53__1\n\
             _______73\n"
        );

        assert_eq!(to_string(&board), TEST_BOARD);
    }

    #[test]
    fn fill_board_spaces() {
        let board = board_from_string(
            "92_ ___\t___
             5__ 87_\t___
             _38 _91\t___
             _52 93_\t16_
             _9_ ___\t_3_
             _73 _64\t98_
             ___ 41_\t25_
             ___ _53\t__1
             ___ ___\t_73"
        );

        assert_eq!(to_string(&board), TEST_BOARD);
    }

    #[test]
    fn fill_board_spaces_newline() {
        let board = board_from_string(
            "92_ ___\t___\n
             5__ 87_\t___\n
             _38 _91\t___\n
             \n
             _52 93_\t16_\n
             _9_ ___\t_3_\n
             _73 _64\t98_\n
             \n
             ___ 41_\t25_\n
             ___ _53\t__1\n
             ___ ___\t_73\n"
        );

        assert_eq!(to_string(&board), TEST_BOARD);
    }
}
