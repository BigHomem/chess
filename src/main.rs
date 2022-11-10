#![allow(unused)]
use std::{cmp::{Ord, Ordering::{Greater, Less}}};
use crossterm::{event::{read, Event, KeyCode}};

enum Input {
    Up,
    Down,
    Left,
    Right,
    Select
}

fn get_input() -> Input {
    loop {
        match read().unwrap() {
            Event::Key(key) => match key.code {
                KeyCode::Char('w') => return Input::Up,
                KeyCode::Char('a') => return Input::Left,
                KeyCode::Char('s') => return Input::Down,
                KeyCode::Char('d') => return Input::Right,
                KeyCode::Char(' ') => return Input::Select,
                _ => continue
            }
            _ => continue
        };
    }
}
fn flatten<T>(nested: Vec<Vec<T>>) -> Vec<T> {
    nested.into_iter().flatten().collect()
}
#[derive(Debug, Clone)]
struct Point(i32, i32);
impl Point {
    fn add(&self, point2: &Self) -> Self {
        Point(self.0 + point2.0, self.1 + point2.1)
    }
    fn mul(&self, point2: &Self) -> Self {
        Point(self.0 * point2.0, self.1 * point2.1)
    }
    fn rev(&self) -> Self {
        Point(self.1, self.0)
    }
    fn eq(&self, point2: &Self) -> bool {
        if self.0 == point2.0 && self.1 == point2.1 {true} else {false}
    }
    fn tup(&self) -> (i32, i32) {
        (self.0, self.1)
    }
    fn clamp(&self, Point(xmin, ymin): &Point, Point(xmax, ymax): &Point) -> Self {
        Point(self.0.clamp(*xmin, *xmax),
              self.1.clamp(*ymin, *ymax))
    }
    fn out_bounds(&self) -> bool {
        if self.0 < 1 || self.0 > 8 {return true}
        if self.1 < 1 || self.1 > 8 {return true}
        false 
    }
}
struct Line(Point);
impl Line {
    fn path(&self) -> Vec<Point> {
        match &self.0 {
            Point(x, y) => {
                let (scaler, range) = match (x, y) {
                    (x, 0) => (if *x > 0 {Point(1, 0)} else {Point(-1, 0)}, 1..=x.abs()),
                    (0, y) => (if *y > 0 {Point(0, 1)} else {Point(0, -1)}, 1..=y.abs()),
                    (x, y) if x.abs() == y.abs() => {
                        match (*x > 0, *y > 0) {
                            (true, true) => (Point(1, 1), 1..=x.abs()),
                            (true, false) => (Point(1, -1), 1..=x.abs()),
                            (false, true) => (Point(-1, 1), 1..=x.abs()),
                            _ => (Point(-1, -1), 1..=x.abs()),
                        }
                    }
                    _ => panic!("Invalid Line")
                };
                let mut result = vec![];
                for i in range {
                    result.push(Point(i, i).mul(&scaler))
                }
                result
            }
        }
    }
} 
#[derive(Clone)]
enum SelectionType {
    Cursor,
    Move,
    NoSelection
}
#[derive(Clone)]
struct Tile {
    position: Point,
    piece: Option<Piece>,
    color: Color,
    selected: SelectionType
}
impl Tile {
    fn render(&self) -> String {
        let piece = match &self.piece {
            Some(x) => x.render(),
            None => " ".to_string()
        };
        
        let color_ansi = if let Color::White = self.color {"\x1b[48;5;250m"} else {"\x1b[48;5;245m"};
        let colored = format!("{}{}",color_ansi, piece);
        let tile_selected = match self.selected {
            SelectionType::Cursor => format!("{}\x1b[38;5;0m<", colored),
            _ => format!("{} ",colored)
        };
        tile_selected
    }
}
#[derive(Clone)]
struct Piece {
    piece_type: PieceType,
    color: Color
}
impl Piece {
    fn render(&self) -> String {
        let icon = match self.piece_type {
            PieceType::Pawn(_) => "♟︎",
            PieceType::Knight => "♞",
            PieceType::Bishop => "♝",
            PieceType::Rook(_) => "♜",
            PieceType::Queen => "♛",
            PieceType::King(_) => "♚"
        }.to_string();
        let color_ansi = if let Color::White = self.color {"\x1b[38;5;255m"} else {"\x1b[38;5;0m"};
        format!("{}{}", color_ansi, icon)
    }
    fn moves(&self) -> Vec<Movement> {
        let color_scaler = if let Color::White = self.color {Point(1, 1)} else {Point(1, -1)};
        match &self.piece_type {
            PieceType::Pawn(moved) => {
                vec![
                    Movement::new(Point(1,-1).mul(&color_scaler).tup(), MoveType::Attack),
                    Movement::new(Point(-1,-1).mul(&color_scaler).tup(), MoveType::Attack),
                    Movement::new(Point(0, if *moved {-1} else {-2}).mul(&color_scaler).tup(), MoveType::Move)
                ]
            },
            PieceType::Knight => {
                flatten(vec![
                    Movement::new((1, 2), MoveType::Jump).mirror_4(),
                    Movement::new((-1, 2), MoveType::Jump).mirror_4()
                ])
            },
            PieceType::King(_) => {
                flatten(vec![
                    Movement::new((1, 1), MoveType::Default).mirror_4(),
                    Movement::new((0, 1), MoveType::Default).mirror_4()
                ])
            },
            PieceType::Bishop => {
                flatten(vec![
                    Movement::new((8, 8), MoveType::Default).mirror_4()
                ])
            },
            PieceType::Rook(_) => {
                flatten(vec![
                    Movement::new((0, 8), MoveType::Default).mirror_4()
                ])
            },
            PieceType::Queen => {
                flatten(vec![
                    Movement::new((8, 8), MoveType::Default).mirror_4(),
                    Movement::new((0, 8), MoveType::Default).mirror_4()
                ])
            },
        }
    }
}
#[derive(Debug)]
struct Movement {
    //in relative coords vvv
    points: Vec<Point>,
    move_type: MoveType
}
impl Movement {
    fn new((x, y): (i32, i32), move_type: MoveType) -> Self {
        match move_type {
            MoveType::Jump => Movement { points: vec![Point(x, y)], move_type },
            MoveType::Default => Movement { points: Line(Point(x, y)).path(), move_type },
            MoveType::Attack => Movement { points: Line(Point(x, y)).path(), move_type },
            MoveType::Move => Movement { points: Line(Point(x, y)).path(), move_type },
        }
    }
    fn mirror_4(&self) -> Vec<Self> {
        let mut result: Vec<Self> = vec![];
        let move_type = &self.move_type;
        for i in 1..=4 {
            let scaler = match i {
                1 => Point(1,1),
                2 => Point(-1,-1),
                3 => Point(-1,1),
                _ => Point(1,-1),
            };
            let reverse = if let 3 | 4 = i {true} else {false}; 
            let scaled_points: Vec<Point> =self.points
                .iter()
                .map(|point| point.mul(&scaler))
                .collect();
            let reversed: Vec<Point> = if reverse {
                scaled_points.iter()
                    .map(|x| x.rev())
                    .collect()
            } else {scaled_points};
            result.push(Movement {
                points: reversed,
                move_type: move_type.clone()
            });
        };

        result
    } 
}
#[derive(Clone, Debug)]
enum MoveType {
    //both attack and move vvv
    Default,
    Jump,
    Attack,
    Move,
}
#[derive(Clone)]
enum PieceType {
    //piece(has_moved)
    Pawn(bool),
    Knight,
    Bishop,
    Rook(bool),
    Queen,
    King(bool)
}
#[derive(Clone)]
enum Color {
    White,
    Black
}
impl Color {
    fn opposite(&self) -> Self {
        if let Color::White = self {Color::Black} else {Color::White}
    }
    fn eq(&self, other_color: &Self) -> bool {
        match (self, other_color) {
            (Color::White, Color::White) | (Color::Black, Color::Black) => true,
            _ => false
        }
    }
    fn render(&self) -> String {
        match self {
            Color::White => "white",
            Color::Black => "black"
        }.to_string()
    }
}
struct Game {
    board: Vec<Tile>,
    cursor: Point,
    selected: Option<Point>,
    turn: Color
}
impl<'a> Game {
    fn render(&self) {
        print!("\x1B[2J");
        println!("\x1B[H{}'s turn", self.turn.render());
        for line in self.board.chunks_exact(8) {
            let rendered_line: String = line.iter()
                .map(|line| line.render())
                .fold("".to_string(), |acc, x| format!("{}{}", acc, x));
            println!("{}", rendered_line);
        }
        println!("\x1B[0m ");
    }
    fn new_board() -> Vec<Tile> {
        let mut board: Vec<Tile> = vec![];
        for y in 1..=8 {
            for x in 1..=8 {
                board.push(Tile{
                    position: Point(x, y),
                    selected: SelectionType::NoSelection,
                    piece:  match (x, y) {
                                (_, 2) => Some(Piece{piece_type: PieceType::Pawn(false), color: Color::Black}),
                                (_, 7) => Some(Piece{piece_type: PieceType::Pawn(false), color: Color::White}),
                                (1 | 8, 1 | 8) => Some(Piece{piece_type: PieceType::Rook(false), color: if y == 1 {Color::Black} else {Color::White}}),
                                (2 | 7, 1 | 8) => Some(Piece{piece_type: PieceType::Knight, color: if y == 1 {Color::Black} else {Color::White}}),
                                (3 | 6, 1 | 8) => Some(Piece{piece_type: PieceType::Bishop, color: if y == 1 {Color::Black} else {Color::White}}),
                                (4, 1 | 8) => Some(Piece{piece_type: PieceType::Queen, color: if y == 1 {Color::Black} else {Color::White}}),
                                (5, 1 | 8) => Some(Piece{piece_type: PieceType::King(false), color: if y == 1 {Color::Black} else {Color::White}}),
                                _ => None
                            },
                    color: if x%2 == y%2 {Color::White} else {Color::Black}
                })
            }
        }
        board
    }
    fn can_move(&self, Point(x1, y1): &Point, Point(x2, y2): &Point) -> bool {
        match self.valid_moves(&Point(*x1, *y1)) {
            Some(moves) => {
                moves.iter()
                     .map(|point|(&Point(*x1, *y1)).add(point).eq(&Point(*x2, *y2)))
                     .any(|x| x && true)
            },
            None => false
        }
    }
    fn valid_moves(&self, Point(x, y): &Point) -> Option<Vec<Point>> {
        let from_tile = self.get_tile(&Point(*x, *y));
        let piece = if from_tile.piece.is_none() {return None} else {from_tile.piece.as_ref().unwrap()};
        let mut valid_moves: Vec<Point> = vec![]; 
        for movement in piece.moves() {

            let mut found_enemy = false;

            movement.points.iter()
                .take_while(|point| {

                    let move_point = point.add(&Point(*x, *y));
                    let mut result = !move_point.out_bounds();

                    if result {match self.get_tile(&move_point) {
                        Tile {position: _, piece: Some(to_piece), ..} => {
                            result = match (&piece.color, &to_piece.color, &movement.move_type) {
                                (_, _, MoveType::Move) => false,
                                (Color::White, Color::Black, _) | (Color::Black, Color::White, _) if !found_enemy => {
                                    found_enemy = true;
                                    true
                                },
                                _ => false
                            };
                        }
                        Tile {position: _, piece: None, ..} => {result =  if let MoveType::Attack = movement.move_type {false} else {true}}
                    }}
                    result

                })
                .for_each(|point| valid_moves.push(point.clone()));
        };
        Some(valid_moves)
    }
    fn move_piece(&mut self, point1: &Point, point2: &Point) {
        let piece = self.get_tile_mut(point1).piece.clone().unwrap();
        if let PieceType::Pawn(_) = piece.piece_type {
            self.get_tile_mut(point2).piece = Some(Piece { piece_type: PieceType::Pawn(true), color: piece.color });
        } else {
            self.get_tile_mut(point2).piece = Some(piece);
        }
        self.get_tile_mut(point1).piece = None;
    }
    fn get_tile(&'a self, Point(x, y): &Point) -> &Tile {
        self.board.get((x + (y - 1) * 8 - 1) as usize).unwrap()
    }
    fn get_tile_mut(&mut self, Point(x, y): &Point) -> &mut Tile {
        self.board.get_mut((x + (y - 1) * 8 - 1) as usize).unwrap()
    }
    fn handle_input(&'a mut self) {
        let Point(x, y) = self.cursor;
        let event = get_input();
        match event {
            Input::Up => self.cursor = self.cursor.add(&Point(0, -1)),
            Input::Down => self.cursor = self.cursor.add(&Point(0, 1)),
            Input::Left => self.cursor = self.cursor.add(&Point(-1, 0)),
            Input::Right => self.cursor = self.cursor.add(&Point(1, 0)),
            Input::Select if self.selected.is_none() => {
                self.selected = Some(Point(x, y));
            },
            Input::Select if self.selected.is_some() => {
                let from_tile = self.selected.to_owned().unwrap();
                let can_move = self.can_move(&self.selected.as_ref().unwrap(), &Point(x, y));
                let valid_turn = if let Some(x) = &self.get_tile(&from_tile).piece {
                    x.color.eq(&self.turn)
                } else {false};
                if can_move && valid_turn {
                    self.move_piece(&from_tile, &Point(x, y));
                    self.turn = self.turn.opposite();
                }
                self.selected = None;
            }
            _ => return   
            }
    }
    fn update(&'a mut self) -> &mut Self {
        self.handle_input();
        self.cursor = Point(self.cursor.0.clamp(1, 8), self.cursor.1.clamp(1, 8));
        for tile in &mut self.board {
            if let SelectionType::Cursor = tile.selected {tile.selected = SelectionType::NoSelection}
            if tile.position.0 == self.cursor.0 && tile.position.1 == self.cursor.1 {
                tile.selected = SelectionType::Cursor;
            }
        }
        self
    }
    fn game_loop(&mut self) {
        loop {
            self.render();
            self.update();
        }
    }
}
fn main() {
    Game {
        selected: None,
        cursor: Point(4, 4),
        board: Game::new_board(),
        turn: Color::White
    }.game_loop();
}