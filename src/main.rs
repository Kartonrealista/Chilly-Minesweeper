use iced::{Application, Settings};
use minesweeper::*;
fn main() -> iced::Result {
    // let mut some_board = Board::gen_empty();
    // some_board.set_mines(50);
    // println!("{}", some_board);
    Game::run(Settings::default())
    // loop {
    //     let mut buf = String::new();
    //     std::io::stdin()
    //         .read_line(&mut buf)
    //         .expect("Failed to read input");
    //     let mut buf_split = buf.trim_end().split(' ');
    //     match buf_split.next().unwrap() {
    //         "Q" => break,
    //         "G" => {
    //             some_board.guess(
    //                 buf_split.next().unwrap().to_string().parse().unwrap(),
    //                 buf_split.next().unwrap().to_string().parse().unwrap(),
    //             );
    //         }
    //         "M" => some_board.mark(
    //             buf_split.next().unwrap().to_string().parse().unwrap(),
    //             buf_split.next().unwrap().to_string().parse().unwrap(),
    //         ),
    //         _ => {}
    //     }

    //     println!("{}", some_board);
    // }
}
