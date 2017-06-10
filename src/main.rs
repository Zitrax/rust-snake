extern crate pancurses;
extern crate rand;

use pancurses::*;
use rand::distributions::{IndependentSample, Range};
use std::{thread, time};
use std::collections::VecDeque;

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
    p: VecDeque<Pos>,
    d: Direction,
    l: usize
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
    let max = win.get_max_yx();

    let mut snake = Snake {
        p: VecDeque::new(),
        d: Direction::Still,
        l: 3
    };
    snake.p.push_front(Pos { x: 0, y: 0 });

    // Hide cursor
    curs_set(0);
    noecho();

    // Add some fruits
    let mut fruits = Vec::new();
    win.attrset(ColorPair(3));
    let mut rng = rand::thread_rng();
    for _ in 0..5 {
        let y = Range::new(0, max.0).ind_sample(&mut rng);
        let x = Range::new(0, max.1).ind_sample(&mut rng);
        fruits.push(Pos{x: x, y: y});
        win.mvaddch(y, x, '¤');
    }
    win.attrset(ColorPair(1));

    loop {
        let head = snake.p.front().unwrap().clone();
        win.mvaddch(head.y, head.x, '@');

        thread::sleep(time::Duration::from_millis(100));
        match win.getch() {
            Some(k) => {
                match k {
                    // FIXME: Arrow keys are not detected (sends three chars)
                    Input::Character('w') => snake.d = Direction::Up,
                    Input::Character('a') => snake.d = Direction::Left,
                    Input::Character('s') => snake.d = Direction::Down,
                    Input::Character('d') => snake.d = Direction::Right,
                    Input::Character('q') => break,
                    _ => (),
                }
            }
            None => (),
        }
        match snake.d {
            Direction::Up => snake.p.push_front(Pos{x: head.x, y: head.y-1}),
            Direction::Down => snake.p.push_front(Pos{x: head.x, y: head.y+1}),
            Direction::Left => snake.p.push_front(Pos{x: head.x-1, y: head.y}),
            Direction::Right => snake.p.push_front(Pos{x: head.x+1, y: head.y}),
            Direction::Still => ()
        }
        if snake.p.len() > snake.l {
            let back = snake.p.pop_back().unwrap();
            win.mvaddch(back.y, back.x, ' ');
        }

        // Collision check
        let new_head = snake.p.front().unwrap().clone();
        if new_head.y < 0 || new_head.x < 0 || new_head.y > max.0 || new_head.x > max.1 {
            snake.d = Direction::Still;
            win.attrset(ColorPair(2));
            for p in snake.p.iter() {
                win.mvaddch(p.y, p.x, 'X');
            }
            win.attrset(ColorPair(1));
        }
        let mut eaten = None;
        for (i, fruit) in fruits.iter().enumerate() {
            if *fruit == new_head {
                snake.l += 2;
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
                fruits.push(Pos{x: x, y: y});
                win.mvaddch(y, x, '¤');
                win.attrset(ColorPair(1));
            },
            None => {}
        }
    }
    endwin();
}
