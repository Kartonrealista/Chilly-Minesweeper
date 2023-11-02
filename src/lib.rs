use crate::MineOrHint::*;
extern crate alloc;
use iced::{
    alignment::{self, Horizontal, Vertical},
    executor, font,
    widget::{self, button, container, mouse_area, text, Column, Row, Text},
    Application, Color, Command, Font, Theme, {theme, Length},
    {Alignment, Element},
};
use rand::{seq::index::sample, thread_rng};
use std::{
    fmt::{Display, Formatter, Result},
    result,
};

pub const WIDTH: usize = 16;
pub const HEIGHT: usize = 16;

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
            Winstate::InProgress => "In progress...",
        };
        write!(f, "{}", out)
    }
}

fn pair_to_index(i: usize, j: usize) -> usize {
    j + i * WIDTH
}
fn index_to_pair(id: usize) -> (usize, usize) {
    let j = id % WIDTH;
    let i = id / WIDTH;
    (i, j)
}
#[derive(Clone, Copy)]
enum MineOrHint {
    Hint(u8),
    Mine,
}

#[derive(Clone)]
pub struct Tile {
    mined: bool,
    hidden: bool,
    marked: bool,
    id: usize,
}

#[derive(Clone)]
pub struct Board(pub Vec<Tile>);

pub struct Game {
    board: Board,
    has_ended: bool,
    winstate: Winstate,
    marked_count: u32,
}

impl Board {
    fn neighbours(id: usize) -> Vec<usize> {
        let (i, j) = index_to_pair(id);
        let mut out = Vec::new();
        if i + 1 < HEIGHT {
            let temp = pair_to_index(i + 1, j);
            out.push(temp);
        }
        if i > 0 {
            let temp = pair_to_index(i - 1, j);
            out.push(temp);
        }
        if j + 1 < WIDTH {
            let temp = pair_to_index(i, j + 1);
            out.push(temp);
        }
        if j > 0 {
            let temp = pair_to_index(i, j - 1);
            out.push(temp);
        }

        if i > 0 && j + 1 < WIDTH {
            let temp = pair_to_index(i - 1, j + 1);
            out.push(temp);
        }
        if j > 0 && i + 1 < HEIGHT {
            let temp = pair_to_index(i + 1, j - 1);
            out.push(temp);
        }
        if i + 1 < HEIGHT && j + 1 < WIDTH {
            let temp = pair_to_index(i + 1, j + 1);
            out.push(temp);
        }
        if i > 0 && j > 0 {
            let temp = pair_to_index(i - 1, j - 1);
            out.push(temp);
        }

        out
    }

    fn mine_or_hint(&self, id: usize) -> MineOrHint {
        if !self.0[id].mined {
            let h = Self::neighbours(id).iter().fold(0, |acc, &tile| {
                if self.0[tile].mined {
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

    pub fn gen_empty() -> Board {
        let mut out = vec![];
        for id in 0..(WIDTH * HEIGHT) {
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
        for id in sample(&mut thread_rng(), WIDTH * HEIGHT, mine_count) {
            self.0[id].mined = true
        }
    }

    fn reveal_all_mines(&mut self) {
        self.0
            .iter_mut()
            .filter(|tile| tile.mined)
            .filter(|tile| !tile.marked)
            .for_each(|tile| {
                tile.hidden = false;
            })
    }
}
impl Default for Board {
    fn default() -> Self {
        Self::gen_empty()
    }
}
impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut out = String::new();
        out += "    ";
        for i in 0..WIDTH {
            out += &i.to_string().chars().nth(0).unwrap_or(' ').to_string();
            out += " "
        }
        out += "\n  ";

        out += "  ";
        for i in 0..WIDTH {
            out += &i.to_string().chars().nth(1).unwrap_or(' ').to_string();
            out += " "
        }
        out += "\n  ";

        for _ in 0..WIDTH {
            out += "__"
        }
        out += "___";
        out += "\n |";
        for _ in 0..WIDTH {
            out += "  "
        }
        out += "   |\n";
        for i in 0..HEIGHT {
            let mut line = String::new();
            line.push_str(" |  ");
            for j in 0..WIDTH {
                let tile = &self.0[pair_to_index(i, j)];
                let tile_string: String;
                match tile.marked {
                    true => match tile.hidden {
                        true => tile_string = "⚑".to_string(),
                        false => match self.mine_or_hint(tile.id) {
                            Hint(h) => {
                                tile_string =
                                    format!("\x1B[1m\x1B[1;3{h}m{h}\x1B[0m")
                            }
                            Mine => tile_string = "🟐".to_string(),
                        },
                    },
                    false => match tile.hidden {
                        true => tile_string = "□".to_string(),
                        false => match self.mine_or_hint(tile.id) {
                            Hint(h) => {
                                tile_string =
                                    format!("\x1B[1m\x1B[1;3{h}m{h}\x1B[0m")
                            }
                            Mine => tile_string = "🟐".to_string(),
                        },
                    },
                }

                line += &tile_string;
                line.push(' ')
            }

            line.push_str(" | ");
            line.push_str(&i.to_string());
            out += &line;
            out += "\n"
        }
        out += " |";
        for _ in 0..WIDTH {
            out += "__"
        }
        out += "___";
        out += "|\n";
        write!(f, "{}", out)
    }
}

impl Game {
    fn guess_helper(&mut self, id: usize, check: bool) {
        for tile2 in Board::neighbours(id) {
            if self.board.0[tile2].hidden {
                if let Hint(h) = self.board.mine_or_hint(tile2) {
                    if h == 0 {
                        if self.board.0[tile2].marked {
                            self.unmark(tile2)
                        }
                        self.board.0[tile2].hidden = false;
                        self.guess_helper(tile2, true)
                    } else if check {
                        if self.board.0[tile2].marked {
                            self.unmark(tile2)
                        }
                        self.board.0[tile2].hidden = false;
                    }
                }
                if let Hint(h0) = self.board.mine_or_hint(id) {
                    if h0 == 0 {
                        if self.board.0[tile2].marked {
                            self.unmark(tile2)
                        }
                        self.board.0[tile2].hidden = false;
                    }
                }
            }
        }
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
            self.board.reveal_all_mines();
            self.winstate = Winstate::Lost;
            self.has_ended = true
        }
        self.guess_helper(id, false);

        let is_won = (0..HEIGHT * WIDTH)
            .filter(|&id| !self.board.0[id].mined)
            .all(|id| !self.board.0[id].hidden);
        if is_won {
            self.board.reveal_all_mines();
            self.winstate = Winstate::Won;
            self.has_ended = true
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Reset,
    MarkedPressed(usize),
    EmptyPressed(usize),
    HiddenRightClick(usize),
    MarkedRightClick(usize),
    NotHiddenPressed,
    FontLoaded(result::Result<(), iced::font::Error>),
}

impl Application for Game {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Game, iced::Command<Message>) {
        let mut game = Game {
            board: Board::gen_empty(),
            has_ended: false,
            winstate: Winstate::InProgress,
            marked_count: 0,
        };
        game.board.set_mines(35);
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
            Message::Reset => (*self, _) = Game::new(()),
            Message::MarkedPressed(id) | Message::MarkedRightClick(id)
                if !self.has_ended =>
            {
                self.unmark(id)
            }
            Message::EmptyPressed(id) if !self.has_ended => self.guess(id),
            Message::NotHiddenPressed if !self.has_ended => {}
            Message::HiddenRightClick(id) if !self.has_ended => self.mark(id),
            Message::FontLoaded(p) => p.expect("font fail"),
            _ => {}
        };

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        playfield(self)
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

fn playfield(game: &Game) -> iced::widget::Container<Message> {
    let tilebutton = |id| match game.board.0[id] {
        Tile {
            mined: _,
            hidden: true,
            marked: false,
            id: pos,
        } => mouse_area(
            button(" ")
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
            button(
                icon('⚑')
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center),
            )
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
        } => mouse_area(
            button(
                match game.board.mine_or_hint(pos) {
                    Hint(h) => {
                        if h == 0 {
                            text("")
                        } else {
                            text(format!("{h}")).style(theme::Text::Color(
                                Color::from_rgb(
                                    (((h as f32) / 25.5 * 100.0).floor()
                                        as f32
                                        - ((h as f32) / 25.5 * 100.0))
                                        .abs(),
                                    (((h as f32) / 65.33 * 100.0).ceil()
                                        as f32
                                        - ((h as f32) / 65.33 * 100.0))
                                        .abs(),
                                    (((h as f32) / 15.73 * 100.0).ceil()
                                        as f32
                                        - ((h as f32) / 15.73 * 100.0))
                                        .abs(),
                                ),
                            ))
                        }
                    }
                    Mine => icon('🟐'),
                }
                .horizontal_alignment(alignment::Horizontal::Center)
                .vertical_alignment(alignment::Vertical::Center),
            )
            .style(theme::Button::Secondary)
            .height(35)
            .width(35),
        )
        .on_press(Message::NotHiddenPressed),
        _ => {
            panic!()
        }
    };

    let playboard = (0..WIDTH).fold(Row::new(), |acc, column| {
        let new_column = (0..HEIGHT).fold(Column::new(), |acc2, row| {
            acc2.push(tilebutton(pair_to_index(row, column) as usize))
        });
        acc.push(new_column.spacing(2).align_items(Alignment::Center))
    });

    container(
        widget::column![
            widget::row![button("RESET")
                .on_press(Message::Reset)
                .style(theme::Button::Destructive),]
            .padding(20)
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
    .into()
}
const FONT: Font = Font::with_name("Symbola-Regular");
fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(FONT)
        .shaping(iced::widget::text::Shaping::Advanced)
}
