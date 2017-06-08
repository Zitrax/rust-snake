extern crate pancurses;

use pancurses::{initscr, curs_set, endwin, Input, noecho};
use std::{thread, time};

struct Snake {
    x: i32,
    y: i32,
    l: i32,
}

fn main() {
    let win = initscr();
    win.nodelay(true); // Makes getch() non-blocking

    // TODO: Can we default these ?
    let mut snake = Snake { x: 0, y: 0, l: 1 };

    // Hide cursor
    curs_set(0);
    noecho();

    loop {
        win.mvaddch(snake.y, snake.x, '@');
        win.refresh();
        thread::sleep(time::Duration::from_millis(10));
        match win.getch() {
            Some(k) => {
                //println!("{:?}", k);
                match k {
                    // FIXME: Arrow keys are not detected (sends three chars)
                    Input::Character('w') => snake.y -= 1,
                    Input::Character('a') => snake.x -= 1,
                    Input::Character('s') => snake.y += 1,
                    Input::Character('d') => snake.x += 1,
                    Input::Character('q') => break,
                    _ => (),
                }
            }
            None => (),
        }
    }
    endwin();
}
