extern crate pancurses;

use pancurses::{initscr, curs_set, endwin, Input, noecho};
use std::{thread, time};

enum Direction {
    Left,
    Right,
    Up,
    Down,
    Still,
}

struct Snake {
    x: i32,
    y: i32,
    d: Direction,
}

fn main() {
    let win = initscr();
    win.nodelay(true); // Makes getch() non-blocking

    let mut snake = Snake {
        x: 0,
        y: 0,
        d: Direction::Still,
    };

    // Hide cursor
    curs_set(0);
    noecho();

    loop {
        win.mvaddch(snake.y, snake.x, '@');
        thread::sleep(time::Duration::from_millis(100));
        match win.getch() {
            Some(k) => {
                //println!("{:?}", k);
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
            Direction::Up => snake.y -= 1,
            Direction::Down => snake.y += 1,
            Direction::Left => snake.x -= 1,
            Direction::Right => snake.x += 1,
            Direction::Still => (),
        }
    }
    endwin();
}
