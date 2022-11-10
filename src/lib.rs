use crossterm::{event::{read, Event, KeyCode}};

pub fn flatten<T>(nested: Vec<Vec<T>>) -> Vec<T> {
    nested.into_iter().flatten().collect()
}

pub enum Input {
    Up,
    Down,
    Left,
    Right,
    Select
}
pub fn get_input() -> Input {
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

#[derive(Debug, Clone)]
pub struct Point(pub i32,pub i32);
impl Point {
    pub fn add(&self, point2: &Self) -> Self {
        Point(self.0 + point2.0, self.1 + point2.1)
    }
    pub fn mul(&self, point2: &Self) -> Self {
        Point(self.0 * point2.0, self.1 * point2.1)
    }
    pub fn rev(&self) -> Self {
        Point(self.1, self.0)
    }
    pub fn eq(&self, point2: &Self) -> bool {
        if self.0 == point2.0 && self.1 == point2.1 {true} else {false}
    }
    pub fn tup(&self) -> (i32, i32) {
        (self.0, self.1)
    }
    pub fn out_bounds(&self) -> bool {
        if self.0 < 1 || self.0 > 8 {return true}
        if self.1 < 1 || self.1 > 8 {return true}
        false 
    }
}
pub struct Line(pub Point);
impl Line {
    pub fn path(&self) -> Vec<Point> {
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
pub enum Color {
    White,
    Black
}
impl Color {
    pub fn opposite(&self) -> Self {
        if let Color::White = self {Color::Black} else {Color::White}
    }
    pub fn eq(&self, other_color: &Self) -> bool {
        match (self, other_color) {
            (Color::White, Color::White) | (Color::Black, Color::Black) => true,
            _ => false
        }
    }
    pub fn render(&self) -> String {
        match self {
            Color::White => "white",
            Color::Black => "black"
        }.to_string()
    }
}
#[derive(Clone, Debug)]
pub enum MoveType {
    //both attack and move vvv
    Default,
    Jump,
    Attack,
    Move,
}
#[derive(Debug)]
pub struct Movement {
    //in relative coords vvv
    pub points: Vec<Point>,
    pub move_type: MoveType
}
impl Movement {
    pub fn new((x, y): (i32, i32), move_type: MoveType) -> Self {
        match move_type {
            MoveType::Jump => Movement { points: vec![Point(x, y)], move_type },
            _ => Movement { points: Line(Point(x, y)).path(), move_type }
        }
    }
    pub fn mirror_4(&self) -> Vec<Self> {
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

#[derive(Clone)]
pub enum SelectionType {
    Cursor,
    NoSelection
}