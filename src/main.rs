extern crate pancurses;
extern crate rand;

use pancurses::*;
use rand::distributions::{IndependentSample, Range};
use rand::Rng;
use std::{thread, time};
use std::collections::VecDeque;
use std::collections::HashSet;
use std::collections::HashMap;
use std::slice::Iter;

#[derive(Clone, Debug, PartialEq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
    Still,
}

impl Direction {
    /// Iterator over the non still directions
    pub fn iterator() -> Iter<'static, Direction> {
        static DIRECTIONS: [Direction; 4] = [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ];
        DIRECTIONS.into_iter()
    }
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

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn go(&mut self, d: Direction) -> &Self {
        match d {
            Direction::Up => self.y -= 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
            Direction::Right => self.x += 1,
            Direction::Still => (),
        }
        return self;
    }

    fn get(&self, d: Direction) -> Pos {
        let mut pos = self.clone();
        pos.go(d);
        return pos;
    }
}

#[derive(Clone)]
struct Snake<'s> {
    /// Unique ID
    id: u8,
    /// Snake body positions
    p: VecDeque<Pos>,
    /// Current movement direction
    d: Direction,
    /// Length
    l: usize,
    /// Color ID
    c: u8,
    /// Is the snake dead?
    dead: bool,
    /// Takes function that steer the Snake
    input_handler: &'s Fn(&mut Snake, &Window, Option<Input>),
}

impl<'s> Snake<'s> {
    fn head(&self) -> Pos {
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
            Direction::Still => (),
            _ => self.p.push_front(head.get(self.d.clone())),
        }
        if self.p.len() > self.l {
            let back = self.p.pop_back().unwrap();
            win.mvaddch(back.y, back.x, ' ');
        }
    }

    /// Snake is dead, stop and visualize
    fn die(&mut self, win: &Window) {
        self.d = Direction::Still;
        self.dead = true;
        win.attrset(ColorPair(2));
        for p in self.p.iter() {
            win.mvaddch(p.y, p.x, 'X');
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
            self.die(win);
        }

        for snake in snakes.iter_mut() {
            if snake.id == self.id {
                snake.p.pop_front(); // Can remove head since we work on a copy
                if snake.p.contains(&self.p[0]) {
                    self.die(win);
                }
            } else if snake.p.contains(&self.p[0]) {
                self.die(win);
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

    /// Use Lee's algorithm to find the nearest fruit
    /// FIXME: Avoid passing snakes and fruits around?
    fn closest_fruit(&self, win: &Window, fruits: &Vec<Pos>, snakes: &Vec<Snake>) -> Option<Pos> {
        //print!("1 l:{} p:{:?}", d, p);

        let mut visited = HashSet::new();
        let mut to_visit = VecDeque::new();
        let mut meta = HashMap::<Pos, Direction>::new();
        to_visit.push_back(self.head());

        while !to_visit.is_empty() {
            let pos = to_visit.pop_front().unwrap();

            if fruits.contains(&pos) {

                /* Draw shortest path
                let mut bt = Vec::<Pos>::new();
                let mut wp = pos.clone();
                loop {
                    match meta.get(&wp) {
                        Some(d) => match d {
                            &Direction::Left => bt.push(wp.go(Direction::Right).clone()),
                            &Direction::Right => bt.push(wp.go(Direction::Left).clone()),
                            &Direction::Up => bt.push(wp.go(Direction::Down).clone()),
                            &Direction::Down => bt.push(wp.go(Direction::Up).clone()),
                            &Direction::Still => (),
                        },
                        None => break,
                    }
                }
                for p in bt {
                    win.mvaddch(p.y, p.x, 'T');
                }*/

                return Some(pos);
            }

            // No fruit, search further
            for dir in Direction::iterator() {
                let np = pos.get(dir.clone());

                if visited.contains(&np) {
                    continue;
                }

                // Out of bounds?
                let max = win.get_max_yx();
                if np.y < 0 || np.x < 0 || np.y > max.0 || np.x > max.1 {
                    continue;
                }

                // Hitting a snake?
                for snake in snakes {
                    if snake.p.contains(&np) {
                        continue;
                    }
                }

                // Empty location - recurse
                if !to_visit.contains(&np) {
                    to_visit.push_back(np.clone());
                    meta.insert(np.clone(), dir.clone());
                }
            }

            visited.insert(pos);
        }

        return None;
    }
}
/// Collect some data about the snake and it's surroundings
//fn stats(&self) {
        // Shortest fruit distances

    //}

/// A simple AI that just moves around randomly
fn random_ai(snake: &mut Snake, win: &Window, _key: Option<Input>) {
    if snake.d != Direction::Still {
        let max = win.get_max_yx();
        let head = snake.head();
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
            snake.set_dir(rand::random::<Direction>());
        }
        while forbidden.contains(&snake.d) {
            snake.set_dir(rand::random::<Direction>());
        }
    }
}

/// Manual input by a human using keypresses
fn human(snake: &mut Snake, _win: &Window, key: Option<Input>) {
    if !snake.dead {
        match key {
            Some(k) => match k {
                _ => snake.set_dir_from_input(k),
            },
            None => (),
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
    init_pair(4, COLOR_BLUE, COLOR_BLACK);
    init_pair(5, COLOR_WHITE, COLOR_BLACK);
    init_pair(6, COLOR_CYAN, COLOR_BLACK);
    init_pair(7, COLOR_MAGENTA, COLOR_BLACK);

    win.nodelay(true); // Makes getch() non-blocking
    win.keypad(true); // Return special keys as single keys (like arrow keys)
    let max = win.get_max_yx();

    let mut snakes = Vec::new();

    // Human snake
    let human_snake = true;

    if human_snake {
        let mut snake = Snake {
            id: 0,
            p: VecDeque::new(),
            d: Direction::Still,
            l: 3,
            c: 1,
            dead: false,
            input_handler: &human,
        };
        snake.p.push_front(Pos {
            x: max.1 / 2,
            y: max.0 / 2,
        });
        snakes.push(snake);
    }

    // AI snakes
    let ai_snakes_count = 4;
    let mut rng = rand::thread_rng();

    for i in 0..ai_snakes_count {
        let mut bad_snake = Snake {
            id: 1 + i,
            p: VecDeque::new(),
            d: rand::random::<Direction>(),
            l: 3,
            c: (i % 6) + 2,
            dead: false,
            input_handler: &random_ai,
        };
        bad_snake.p.push_front(Pos {
            x: Range::new(0, max.1).ind_sample(&mut rng),
            y: Range::new(0, max.0).ind_sample(&mut rng),
        });
        snakes.push(bad_snake);
    }

    // Hide cursor
    curs_set(0);
    noecho();

    // Add some fruits
    let fruitsymbol = '#';
    let mut fruits = Vec::new();
    win.attrset(ColorPair(3));
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

            if key.is_some() && key.unwrap() == Input::Character('q') {
                done = true;
                break;
            }

            // Closest fruit for snake
            if s.id == 0 {
            let cfruit = s.closest_fruit(&win, &fruits, &snakes_copy);
            match cfruit {
                Some(cfruit) => {
                    win.attrset(ColorPair(3));
                    for fruit in fruits.iter() {
                        win.mvaddch(fruit.y, fruit.x, '#');
                    }
                    win.attrset(ColorPair(7));
                    win.mvaddch(cfruit.y, cfruit.x, 'O');
                }
                None => (),
            }
            }

            (s.input_handler)(s, &win, key);
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
