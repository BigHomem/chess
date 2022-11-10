use std::cmp::Ord;
mod lib;
use lib::*;

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
        match piece.piece_type {
            PieceType::Pawn(_) => self.get_tile_mut(point2).piece = Some(Piece { piece_type: PieceType::Pawn(true), color: piece.color }),
            PieceType::Rook(_) => self.get_tile_mut(point2).piece = Some(Piece { piece_type: PieceType::Rook(true), color: piece.color }),
            PieceType::King(_) => self.get_tile_mut(point2).piece = Some(Piece { piece_type: PieceType::King(true), color: piece.color }),
            _ => self.get_tile_mut(point2).piece = Some(piece),
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