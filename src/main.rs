extern crate pancurses;

use pancurses::{initscr, curs_set, endwin, Input, noecho};
use std::{thread, time};
use std::collections::VecDeque;

enum Direction {
    Left,
    Right,
    Up,
    Down,
    Still,
}

#[derive(Clone, Debug)]
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
    win.nodelay(true); // Makes getch() non-blocking

    let mut snake = Snake {
        p: VecDeque::new(),
        d: Direction::Still,
        l: 10
    };
    snake.p.push_front(Pos { x: 0, y: 0 });

    // Hide cursor
    curs_set(0);
    noecho();

    loop {
        for pos in snake.p.iter() {
            win.mvaddch(pos.y, pos.x, '@');
        }

        thread::sleep(time::Duration::from_millis(50));
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
        let head = snake.p.front().unwrap().clone();
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
    }
    endwin();
}