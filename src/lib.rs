use crate::MineOrHint::*;
extern crate alloc;
use iced::{
    alignment::{self, Horizontal, Vertical},
    executor, font,
    widget::{
        self, button, container, mouse_area, text, text_input, Column, Row,
        Text,
    },
    Application, Color, Command, Font, Renderer, Theme, {theme, Length},
    {Alignment, Element},
};
use rand::{seq::index::sample, thread_rng};
use std::{
    fmt::{Display, Formatter, Result},
    result,
};

enum Winstate {
    Won,
    Lost,
    InProgress,
}
impl Display for Winstate {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let out = match self {
            Winstate::Won => "You won!",
            Winstate::Lost => "You lost!",
            Winstate::InProgress => "Game in progress...",
        };
        write!(f, "{}", out)
    }
}

enum MineOrHint {
    Hint(u8),
    Mine,
}

pub struct Tile {
    mined: bool,
    hidden: bool,
    marked: bool,
    id: usize,
}

pub struct Board(pub Vec<Tile>);

pub struct Game {
    board: Board,
    has_ended: bool,
    winstate: Winstate,
    marked_count: u32,
    menu: Menu,
}

struct Menu {
    width_inptut: String,
    height_inptut: String,
    mine_count_inptut: String,
    width: usize,
    height: usize,
    mine_count: usize,
    start_pressed: bool,
}

// impl Display for Board {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//         let mut out = String::new();
//         out += "    ";
//         for i in 0..self.menu.width {
//             out += &i.to_string().chars().nth(0).unwrap_or(' ').to_string();
//             out += " "
//         }
//         out += "\n  ";

//         out += "  ";
//         for i in 0..self.menu.width {
//             out += &i.to_string().chars().nth(1).unwrap_or(' ').to_string();
//             out += " "
//         }
//         out += "\n  ";

//         for _ in 0..self.menu.width {
//             out += "__"
//         }
//         out += "___";
//         out += "\n |";
//         for _ in 0..self.menu.width {
//             out += "  "
//         }
//         out += "   |\n";
//         for i in 0..self.menu.height {
//             let mut line = String::new();
//             line.push_str(" |  ");
//             for j in 0..self.menu.width {
//                 let tile = &self.board.0[self.pair_to_index(i, j)];
//                 let tile_string: String;
//                 match tile.marked {
//                     true => match tile.hidden {
//                         true => tile_string = "⚑".to_string(),
//                         false => match self.mine_or_hint(tile.id) {
//                             Hint(h) => {
//                                 tile_string =
//                                     format!("\x1B[1m\x1B[1;3{h}m{h}\x1B[0m")
//                             }
//                             Mine => tile_string = "🟐".to_string(),
//                         },
//                     },
//                     false => match tile.hidden {
//                         true => tile_string = "□".to_string(),
//                         false => match self.mine_or_hint(tile.id) {
//                             Hint(h) => {
//                                 tile_string =
//                                     format!("\x1B[1m\x1B[1;3{h}m{h}\x1B[0m")
//                             }
//                             Mine => tile_string = "🟐".to_string(),
//                         },
//                     },
//                 }

//                 line += &tile_string;
//                 line.push(' ')
//             }

//             line.push_str(" | ");
//             line.push_str(&i.to_string());
//             out += &line;
//             out += "\n"
//         }
//         out += " |";
//         for _ in 0..self.menu.width {
//             out += "__"
//         }
//         out += "___";
//         out += "|\n";
//         write!(f, "{}", out)
//     }
// }

impl Game {
    fn neighbours(&self, id: usize) -> Vec<usize> {
        let (i, j) = self.index_to_pair(id);
        let mut out = Vec::new();
        if i + 1 < self.menu.height {
            let temp = self.pair_to_index(i + 1, j);
            out.push(temp);
        }
        if i > 0 {
            let temp = self.pair_to_index(i - 1, j);
            out.push(temp);
        }
        if j + 1 < self.menu.width {
            let temp = self.pair_to_index(i, j + 1);
            out.push(temp);
        }
        if j > 0 {
            let temp = self.pair_to_index(i, j - 1);
            out.push(temp);
        }

        if i > 0 && j + 1 < self.menu.width {
            let temp = self.pair_to_index(i - 1, j + 1);
            out.push(temp);
        }
        if j > 0 && i + 1 < self.menu.height {
            let temp = self.pair_to_index(i + 1, j - 1);
            out.push(temp);
        }
        if i + 1 < self.menu.height && j + 1 < self.menu.width {
            let temp = self.pair_to_index(i + 1, j + 1);
            out.push(temp);
        }
        if i > 0 && j > 0 {
            let temp = self.pair_to_index(i - 1, j - 1);
            out.push(temp);
        }
        out
    }
    fn pair_to_index(&self, i: usize, j: usize) -> usize {
        j + i * self.menu.width
    }
    fn index_to_pair(&self, id: usize) -> (usize, usize) {
        let j = id % self.menu.width;
        let i = id / self.menu.width;
        (i, j)
    }
    fn mine_or_hint(&self, id: usize) -> MineOrHint {
        if !self.board.0[id].mined {
            let h = self.neighbours(id).iter().fold(0, |acc, &tile| {
                if self.board.0[tile].mined {
                    acc + 1
                } else {
                    acc
                }
            });
            Hint(h)
        } else {
            Mine
        }
    }

    pub fn gen_empty(width: usize, height: usize) -> Board {
        let mut out = vec![];
        for id in 0..(width * height) {
            out.push(Tile {
                mined: false,
                hidden: true,
                marked: false,
                id,
            })
        }
        Board(out)
    }

    pub fn set_mines(&mut self, mine_count: usize) {
        for id in sample(
            &mut thread_rng(),
            self.menu.width * self.menu.height,
            mine_count,
        ) {
            self.board.0[id].mined = true
        }
    }

    fn reveal_all_mines(&mut self) {
        self.board
            .0
            .iter_mut()
            .filter(|tile| tile.mined)
            .filter(|tile| !tile.marked)
            .for_each(|tile| {
                tile.hidden = false;
            })
    }
    pub fn mark(&mut self, id: usize) {
        let tile = &mut self.board.0[id];
        self.marked_count += 1;
        tile.marked = true
    }

    pub fn unmark(&mut self, id: usize) {
        let tile = &mut self.board.0[id];
        self.marked_count -= 1;
        tile.marked = false
    }

    pub fn guess(&mut self, id: usize) {
        let tile = &mut self.board.0[id];
        tile.hidden = false;
        if tile.mined {
            self.reveal_all_mines();
            self.winstate = Winstate::Lost;
            self.has_ended = true
        }
        self.reveal_empty_and_neighbouring_tiles(id, false);

        let is_won = (0..self.menu.height * self.menu.width)
            .filter(|&id| !self.board.0[id].mined)
            .all(|id| !self.board.0[id].hidden);
        if is_won {
            self.reveal_all_mines();
            self.winstate = Winstate::Won;
            self.has_ended = true
        }
    }

    fn reveal_empty_and_neighbouring_tiles(&mut self, id: usize, check: bool) {
        for tile2 in self.neighbours(id) {
            if self.board.0[tile2].hidden {
                if let Hint(h) = self.mine_or_hint(tile2) {
                    if h == 0 {
                        self.unmark_and_unhide(tile2);
                        self.reveal_empty_and_neighbouring_tiles(tile2, true)
                    } else if check {
                        self.unmark_and_unhide(tile2);
                    }
                }
                if let Hint(h0) = self.mine_or_hint(id) {
                    if h0 == 0 {
                        self.unmark_and_unhide(tile2);
                    }
                }
            }
        }
    }

    fn unmark_and_unhide(&mut self, tile2: usize) {
        if self.board.0[tile2].marked {
            self.unmark(tile2)
        }
        self.board.0[tile2].hidden = false;
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Reset,
    MarkedPressed(usize),
    EmptyPressed(usize),
    HiddenRightClick(usize),
    MarkedRightClick(usize),
    FontLoaded(result::Result<(), iced::font::Error>),
    InputWidth(String),
    InputHeight(String),
    InputMineCount(String),
    StartPressed,
    GotoMenu
}

impl<'a> Application for Game {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Game, iced::Command<Message>) {
        let game = Game {
            board: Self::gen_empty(16, 16),
            has_ended: false,
            winstate: Winstate::InProgress,
            marked_count: 0,
            menu: Menu {
                width_inptut: String::from("16"),
                height_inptut: String::from("16"),
                mine_count_inptut: String::from("40"),
                width: 16,
                height: 16,
                mine_count: 40,
                start_pressed: false,
            },
        };
        (
            game,
            font::load(include_bytes!("../fonts/Symbola_hint.ttf").as_slice())
                .map(Message::FontLoaded),
        )
    }

    fn title(&self) -> String {
        String::from("Minesweeper")
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::GotoMenu => {
                (*self, _) = Game::new(());
            }
            Message::StartPressed => {
                self.menu.width = self.menu.width_inptut.parse().unwrap();
                self.menu.height = self.menu.height_inptut.parse().unwrap();
                self.menu.mine_count =
                    self.menu.mine_count_inptut.parse().unwrap();

                self.board = Self::gen_empty(self.menu.width, self.menu.height);
                self.set_mines(self.menu.mine_count);
                self.menu.start_pressed = true;
            }
            Message::InputWidth(input) => self.menu.width_inptut = input,
            Message::InputHeight(input) => self.menu.height_inptut = input,
            Message::InputMineCount(input) => {
                self.menu.mine_count_inptut = input
            }
            Message::Reset => {
                self.board = Self::gen_empty(self.menu.width, self.menu.height);
                self.set_mines(self.menu.mine_count);
                self.has_ended = false;
            }
            Message::MarkedPressed(id) | Message::MarkedRightClick(id)
                if !self.has_ended =>
            {
                self.unmark(id)
            }
            Message::EmptyPressed(id) if !self.has_ended => self.guess(id),
            Message::HiddenRightClick(id) if !self.has_ended => self.mark(id),

            _ => {}
        };

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.menu.start_pressed {
            false => menu(self),
            true => playfield(self),
        }
        .height(Length::Fill)
        .width(Length::Fill)
        .center_x()
        .center_y()
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}

fn playfield<'a>(game: &'a Game) -> iced::widget::Container<'a, Message> {
    let tilebutton = |id| match game.board.0[id] {
        Tile {
            mined: _,
            hidden: true,
            marked: false,
            id: pos,
        } => mouse_area(
            button("")
                .style(theme::Button::Primary)
                .height(35)
                .width(35),
        )
        .on_press(Message::EmptyPressed(pos))
        .on_right_press(Message::HiddenRightClick(pos)),
        Tile {
            mined: _,
            hidden: true,
            marked: true,
            id: pos,
        } => mouse_area(
            button(centralize_tile_content(icon('⚑')))
                .style(theme::Button::Secondary)
                .height(35)
                .width(35),
        )
        .on_press(Message::MarkedPressed(pos))
        .on_right_press(Message::MarkedRightClick(pos)),

        Tile {
            mined: _,
            hidden: false,
            marked: false,
            id: pos,
        } => {
            let tile_content = match game.mine_or_hint(pos) {
                Hint(h) => {
                    if h == 0 {
                        text("")
                    } else {
                        text_with_varied_colors(h)
                    }
                }
                Mine => icon('🟐'),
            };
            mouse_area(
                button(centralize_tile_content(tile_content))
                    .style(theme::Button::Secondary)
                    .height(35)
                    .width(35),
            )
        }
        _ => {
            panic!()
        }
    };

    let playboard = (0..game.menu.width).fold(Row::new(), |acc, column| {
        let new_column = (0..game.menu.height)
            .fold(Column::new(), |acc2, row| {
                acc2.push(tilebutton(game.pair_to_index(row, column)))
            });
        acc.push(new_column.spacing(2).align_items(Alignment::Center))
    });

    container(
        widget::column![
            widget::row![button("MENU")
            .on_press(Message::GotoMenu)
            .style(theme::Button::Positive),
            button("RESET")
                .on_press(Message::Reset)
                .style(theme::Button::Destructive),]
            .padding(20)
            .spacing(20)
            .align_items(Alignment::Center),
            playboard.spacing(2).align_items(Alignment::Center),
            widget::row![
                text(format!("\n{}", game.winstate)),
                text(format!("\n⚑: {}", game.marked_count))
                    .font(FONT)
                    .shaping(iced::widget::text::Shaping::Advanced)
            ]
            .spacing(10)
        ]
        .padding(20)
        .align_items(Alignment::Center),
    )
}

fn centralize_tile_content(tile_content: Text<Renderer>) -> Text<Renderer> {
    tile_content
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center)
}

fn text_with_varied_colors<'a>(h: u8) -> Text<'a, Renderer> {
    text(format!("{h}")).style(theme::Text::Color(Color::from_rgb(
        (((h as f32) / 25.5 * 100.0).floor() - ((h as f32) / 25.5 * 100.0))
            .abs(),
        (((h as f32) / 65.33 * 100.0).ceil() - ((h as f32) / 65.33 * 100.0))
            .abs(),
        (((h as f32) / 15.73 * 100.0).ceil() - ((h as f32) / 15.73 * 100.0))
            .abs(),
    )))
}

const FONT: Font = Font::with_name("Symbola-Regular");
fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(FONT)
        .shaping(iced::widget::text::Shaping::Advanced)
}

fn menu<'a>(game: &Game) -> iced::widget::Container<'a, Message> {
    let width_box =
        text_input("", &game.menu.width_inptut).on_input(Message::InputWidth);
    let height_box =
        text_input("", &game.menu.height_inptut).on_input(Message::InputHeight);
    let mine_count_box = text_input("", &game.menu.mine_count_inptut)
        .on_input(Message::InputMineCount);
    let start_game_button = button(centralize_tile_content(text("START")))
        .on_press(Message::StartPressed)
        .style(theme::Button::Positive)
        .width(96)
        .height(55);
    container(
        iced::widget::column![
            iced::widget::row![text("Width: "), width_box.width(40)]
                .align_items(Alignment::Center),
            iced::widget::row![text("Height: "), height_box.width(40)]
                .align_items(Alignment::Center),
            iced::widget::row![text("Mines: "), mine_count_box.width(40)]
                .align_items(Alignment::Center),
            start_game_button
        ]
        .spacing(20)
        .align_items(Alignment::Center),
    )
}
