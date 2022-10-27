#![allow(unused)]
use std::{cmp::{Ord, Ordering::{Greater, Less}}, collections::btree_map::Iter, vec};
use crossterm::{event::{read, Event, KeyCode}, cursor};

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
            _ => panic!("Invalid Line")
        }
    }
} 
#[derive(Clone)]
enum Selection {
    Cursor,
    Move,
    NoSelection
}
#[derive(Clone)]
struct Tile {
    position: Point,
    piece: Option<Piece>,
    color: Color,
    selected: Selection
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
            Selection::Cursor => format!("{}\x1b[38;5;0m<", colored),
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
struct Game {
    board: Vec<Tile>,
    cursor: Point,
    selected: Option<Tile>,
    turn: Color
}
impl Game {
    fn render(&self) {
        print!("\x1B[2J");
        println!("\x1B[H ");
        for line in self.board.chunks_exact(8) {
            let rendered_line: String = line.iter()
                .map(|line| line.render())
                .fold("".to_string(), |acc, x| format!("{}{}", acc, x));
            println!("{}", rendered_line);
        }
        println!("\x1B[0m cursor at: [{:?}], selected: [{}]", 
            self.cursor,
            match &self.selected {
                None => "No Selection".to_string(),
                Some(tile) => tile.piece.as_ref().unwrap().render()
            }
        );
    }
    fn new_board() -> Vec<Tile> {
        let mut board: Vec<Tile> = vec![];
        for y in 1..=8 {
            for x in 1..=8 {
                board.push(Tile{
                    position: Point(x, y),
                    selected: Selection::NoSelection,
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
        let from_tile = self.board.get((x1 + (y1 - 1) * 8 - 1) as usize).unwrap();
        let piece = &from_tile.piece;
        //check if there's a piece in the tile vvv
        if piece.is_none() {return false}

        //check if tile is reacheable with possible moves
        let mut result = false;
        for movement in piece.as_ref().unwrap().moves().iter() {
            for point in movement.points.iter() {
                if from_tile.position.add(point).eq(&Point(*x2, *y2)) {
                    result = true;
                    break
                }
            }
        }

        result
    }
    fn handle_input(&mut self) {
        let Point(x, y) = self.cursor;
        let event = get_input();
        match event {
            Input::Up => self.cursor = self.cursor.add(&Point(0, -1)),
            Input::Down => self.cursor = self.cursor.add(&Point(0, 1)),
            Input::Left => self.cursor = self.cursor.add(&Point(-1, 0)),
            Input::Right => self.cursor = self.cursor.add(&Point(1, 0)),
            Input::Select if self.selected.is_none() => {
                match self.board.get((x + (y - 1) * 8 - 1) as usize) {
                    Some(tile) if tile.piece.is_some() => self.selected = Some(tile.clone()),
                    _ => return
                } 
            },
            Input::Select if self.selected.is_some() => {
                let can_move = self.can_move(&self.selected.as_ref().unwrap().position ,&Point(x, y));
                println!("{:?}", can_move);
                get_input();
                self.selected = None;
            }
            _ => return   
            }
    }
    fn update(&mut self) -> &mut Self {
        self.handle_input();
        self.cursor = Point(self.cursor.0.clamp(1, 8), self.cursor.1.clamp(1, 8));
        for tile in &mut self.board {
            if let Selection::Cursor = tile.selected {tile.selected = Selection::NoSelection}
            if tile.position.0 == self.cursor.0 && tile.position.1 == self.cursor.1 {
                tile.selected = Selection::Cursor;
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