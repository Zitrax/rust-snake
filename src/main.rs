extern crate pancurses;
extern crate rand;

use pancurses::*;
use rand::distributions::{IndependentSample, Range};
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

#[derive(Clone, Debug, PartialEq)]
struct Pos {
    x: i32,
    y: i32,
}

struct Snake {
    p: VecDeque<Pos>, // positions
    d: Direction,
    l: usize,         // length
    c: u8             // color id
}

impl Snake {
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
        let head = self.p.front().unwrap().clone();
        win.mvaddch(head.y, head.x, '@');
        match self.d {
            Direction::Up => self.p.push_front(Pos { x: head.x, y: head.y - 1 }),
            Direction::Down => self.p.push_front(Pos { x: head.x, y: head.y + 1 }),
            Direction::Left => self.p.push_front(Pos { x: head.x - 1, y: head.y }),
            Direction::Right => self.p.push_front(Pos { x: head.x + 1, y: head.y }),
            Direction::Still => (),
        }
        if self.p.len() > self.l {
            let back = self.p.pop_back().unwrap();
            win.mvaddch(back.y, back.x, ' ');
        }
    }

    // Collision checks
    fn collision(&mut self, win: &Window, fruits: &mut Vec<Pos>, fruitsymbol: char) {
        let max = win.get_max_yx();
        let new_head = self.p.front().unwrap().clone();
        if new_head.y < 0 || new_head.x < 0 || new_head.y > max.0 || new_head.x > max.1 {
            self.d = Direction::Still;
            win.attrset(ColorPair(2));
            for p in self.p.iter() {
                win.mvaddch(p.y, p.x, 'X');
            }
        }
        let mut eaten = None;
        for (i, fruit) in fruits.iter().enumerate() {
            if *fruit == new_head {
                self.l += 2;
                eaten = Some(i);
                break;
            }
        }
        match eaten {
            Some(i) => {
                let _ = fruits.remove(i);
                // Add new fruits
                // TODO: Make sure it's in an empty spot
                win.attrset(ColorPair(3));
                let mut rng = rand::thread_rng();
                let y = Range::new(0, max.0).ind_sample(&mut rng);
                let x = Range::new(0, max.1).ind_sample(&mut rng);
                fruits.push(Pos { x: x, y: y });
                win.mvaddch(y, x, fruitsymbol);
            }
            None => {}
        }
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

    win.nodelay(true); // Makes getch() non-blocking
    win.keypad(true);  // Return special keys as single keys (like arrow keys)
    let max = win.get_max_yx();

    let mut snake = Snake {
        p: VecDeque::new(),
        d: Direction::Still,
        l: 3,
        c: 1
    };
    snake.p.push_front(Pos { x: max.1 / 2, y: max.0 / 2 });

    // Hide cursor
    curs_set(0);
    noecho();

    // Add some fruits
    let fruitsymbol = '#';
    let mut fruits = Vec::new();
    win.attrset(ColorPair(3));
    let mut rng = rand::thread_rng();
    for _ in 0..5 {
        let y = Range::new(0, max.0).ind_sample(&mut rng);
        let x = Range::new(0, max.1).ind_sample(&mut rng);
        fruits.push(Pos { x: x, y: y });
        win.mvaddch(y, x, fruitsymbol);
    }

    loop {
        // Read key and take action
        match win.getch() {
            Some(k) => {
                match k {
                    Input::Character('q') => break,
                    _ => snake.set_dir_from_input(k),
                }
            }
            None => (),
        }
        snake.mv(&win);
        snake.collision(&win, &mut fruits, fruitsymbol);
        thread::sleep(time::Duration::from_millis(100));
    }
    endwin();
}
