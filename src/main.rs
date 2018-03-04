extern crate pancurses;
extern crate rand;

use pancurses::*;
use rand::distributions::{IndependentSample, Range};
use rand::Rng;
use std::{thread, time};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
    Still,
}

impl rand::Rand for Direction {
    fn rand<R: rand::Rng>(rng: &mut R) -> Self {
        static ALL: [Direction; 4] = [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ];
        return rng.choose(&ALL).unwrap().clone();
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Pos {
    x: i32,
    y: i32,
}

#[derive(Clone)]
struct Snake {
    id: u8,           // unique id
    p: VecDeque<Pos>, // positions
    d: Direction,     // current movement direction
    l: usize,         // length
    c: u8,            // color id
    a: bool,          // ai
}

impl Snake {
    fn head(&mut self) -> Pos {
        return self.p.front().unwrap().clone();
    }

    // Set new direction if allowed
    fn set_dir(&mut self, dir: Direction) {
        match dir {
            Direction::Up => {
                if self.d != Direction::Down {
                    self.d = dir;
                }
            }
            Direction::Down => {
                if self.d != Direction::Up {
                    self.d = dir;
                }
            }
            Direction::Left => {
                if self.d != Direction::Right {
                    self.d = dir;
                }
            }
            Direction::Right => {
                if self.d != Direction::Left {
                    self.d = dir;
                }
            }
            Direction::Still => self.d = dir,
        }
    }

    // Reads keyboard input and update snake direction
    fn set_dir_from_input(&mut self, k: Input) {
        match k {
            Input::KeyUp => self.set_dir(Direction::Up),
            Input::KeyLeft => self.set_dir(Direction::Left),
            Input::KeyDown => self.set_dir(Direction::Down),
            Input::KeyRight => self.set_dir(Direction::Right),
            Input::Character('w') => self.set_dir(Direction::Up),
            Input::Character('a') => self.set_dir(Direction::Left),
            Input::Character('s') => self.set_dir(Direction::Down),
            Input::Character('d') => self.set_dir(Direction::Right),
            _ => (),
        }
    }

    // Move snake according to direction
    fn mv(&mut self, win: &Window) {
        win.attrset(ColorPair(self.c));
        let head = self.head();
        win.mvaddch(head.y, head.x, '@');
        match self.d {
            Direction::Up => self.p.push_front(Pos {
                x: head.x,
                y: head.y - 1,
            }),
            Direction::Down => self.p.push_front(Pos {
                x: head.x,
                y: head.y + 1,
            }),
            Direction::Left => self.p.push_front(Pos {
                x: head.x - 1,
                y: head.y,
            }),
            Direction::Right => self.p.push_front(Pos {
                x: head.x + 1,
                y: head.y,
            }),
            Direction::Still => (),
        }
        if self.p.len() > self.l {
            let back = self.p.pop_back().unwrap();
            win.mvaddch(back.y, back.x, ' ');
        }
    }

    // Collision checks
    fn collision(
        &mut self,
        win: &Window,
        fruits: &mut Vec<Pos>,
        fruitsymbol: char,
        snakes: &mut Vec<Snake>,
    ) {
        let max = win.get_max_yx();
        let head = self.head();
        if head.y < 0 || head.x < 0 || head.y > max.0 || head.x > max.1 {
            self.d = Direction::Still;
            win.attrset(ColorPair(2));
            for p in self.p.iter() {
                win.mvaddch(p.y, p.x, 'X');
            }
        }

        for snake in snakes.iter_mut() {
            if snake.id == self.id {
                snake.p.pop_front(); // Can remove head since we work on a copy
                if snake.p.contains(&self.p[0]) {
                    self.d = Direction::Still;
                    for p in self.p.iter() {
                        win.mvaddch(p.y, p.x, 'X');
                    }
                }
            } else if snake.p.contains(&self.p[0]) {
                self.d = Direction::Still;
                for p in self.p.iter() {
                    win.mvaddch(p.y, p.x, 'X');
                }
            }
        }

        let mut eaten = None;
        for (i, fruit) in fruits.iter().enumerate() {
            if *fruit == head {
                self.l += 2;
                eaten = Some(i);
                break;
            }
        }
        match eaten {
            Some(i) => {
                let _ = fruits.remove(i);
                // Add new fruits
                win.attrset(ColorPair(3));
                let mut rng = rand::thread_rng();
                loop {
                    let y = Range::new(0, max.0).ind_sample(&mut rng);
                    let x = Range::new(0, max.1).ind_sample(&mut rng);
                    let pos = Pos { x, y };
                    if !fruits.contains(&pos) {
                        fruits.push(pos);
                        win.mvaddch(y, x, fruitsymbol);
                        break;
                    }
                }
            }
            None => {}
        }
    }

    fn length(&mut self, win: &Window, offset: i32) {
        win.attrset(ColorPair(self.c));
        win.mvaddstr(offset, 0, &format!("Length: {}", self.l));
    }

    fn input_dir(&mut self, win: &Window, key: Option<Input>) -> bool {
        if self.d != Direction::Still && self.a {
            let max = win.get_max_yx();
            let head = self.head();
            let mut forbidden = VecDeque::new();
            if head.x == 0 {
                forbidden.push_back(Direction::Left);
            } else if head.x == max.1 - 1 {
                forbidden.push_back(Direction::Right);
            }
            if head.y == 0 {
                forbidden.push_back(Direction::Up);
            } else if head.y == max.0 - 1 {
                forbidden.push_back(Direction::Down);
            }
            if rand::thread_rng().gen_weighted_bool(10) {
                self.set_dir(rand::random::<Direction>());
            }
            while forbidden.contains(&self.d) {
                self.set_dir(rand::random::<Direction>());
            }
        } else if !self.a {
            // Read key and take action
            match key {
                Some(k) => match k {
                    Input::Character('q') => return false,
                    _ => self.set_dir_from_input(k),
                },
                None => (),
            }
        }
        return true;
    }
}

fn main() {
    let win = initscr();
    start_color();
    use_default_colors();

    // Colors
    init_pair(1, COLOR_GREEN, COLOR_BLACK);
    init_pair(2, COLOR_RED, COLOR_BLACK);
    init_pair(3, COLOR_YELLOW, COLOR_BLACK);
    init_pair(4, COLOR_BLUE, COLOR_BLACK);
    init_pair(5, COLOR_WHITE, COLOR_BLACK);
    init_pair(6, COLOR_CYAN, COLOR_BLACK);

    win.nodelay(true); // Makes getch() non-blocking
    win.keypad(true); // Return special keys as single keys (like arrow keys)
    let max = win.get_max_yx();

    let mut snake = Snake {
        id: 1, // TODO: Autogenerate this
        p: VecDeque::new(),
        d: Direction::Still,
        l: 3,
        c: 1,
        a: false,
    };
    snake.p.push_front(Pos {
        x: max.1 / 2,
        y: max.0 / 2,
    });

    let mut bad_snake = Snake {
        id: 2,
        p: VecDeque::new(),
        d: Direction::Right,
        l: 3,
        c: 2,
        a: true,
    };
    bad_snake.p.push_front(Pos { x: 10, y: 10 });

    let mut bad_snake2 = Snake {
        id: 3,
        p: VecDeque::new(),
        d: Direction::Right,
        l: 3,
        c: 4,
        a: true,
    };
    bad_snake2.p.push_front(Pos { x: 30, y: 30 });

    let mut bad_snake3 = Snake {
        id: 4,
        p: VecDeque::new(),
        d: Direction::Right,
        l: 3,
        c: 5,
        a: true,
    };
    bad_snake3.p.push_front(Pos { x: 50, y: 50 });

    let mut bad_snake4 = Snake {
        id: 5,
        p: VecDeque::new(),
        d: Direction::Right,
        l: 3,
        c: 6,
        a: true,
    };
    bad_snake4.p.push_front(Pos { x: 70, y: 50 });

    let mut snakes = vec![snake, bad_snake, bad_snake2, bad_snake3, bad_snake4];

    // Hide cursor
    curs_set(0);
    noecho();

    // Add some fruits
    let fruitsymbol = '#';
    let mut fruits = Vec::new();
    win.attrset(ColorPair(3));
    let mut rng = rand::thread_rng();
    for _ in 0..50 {
        let y = Range::new(0, max.0).ind_sample(&mut rng);
        let x = Range::new(0, max.1).ind_sample(&mut rng);
        fruits.push(Pos { x: x, y: y });
        win.mvaddch(y, x, fruitsymbol);
    }

    loop {
        let mut done = false;
        // FIXME: Find a better way to avoid the ownership issues other than
        //        copying the whole snake vector. Performance issue.
        let mut snakes_copy = snakes.clone();
        for (i, s) in snakes.iter_mut().enumerate() {
            // FIXME: Multiple presses within one loop are ignored.
            let key = win.getch();

            if !s.input_dir(&win, key) {
                done = true;
                break;
            }
            s.mv(&win);
            s.collision(&win, &mut fruits, fruitsymbol, &mut snakes_copy);
            s.length(&win, i as i32);
        }
        if done {
            break;
        }

        thread::sleep(time::Duration::from_millis(70));
    }
    endwin();
}
