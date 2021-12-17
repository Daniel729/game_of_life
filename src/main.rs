use crossterm::{event, queue, terminal, cursor, execute, event::read};
use std::io::{stdout, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::vec::Vec;

const FILLED: char = '\u{2588}';
const EMPTY: char = ' ';
const DIRECTIONS: [(i32, i32); 8] = [
    (0, 1),
    (0, -1),
    (1, 0),
    (-1, 0),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];
const FRAMES_PER_UPDATE: u32 = 2;
const TIME_FRAME: Duration = Duration::from_millis(1000 / 60);

enum Event {
    Break(),
    Pause(),
    Click(i32, i32),
}

fn fill(filled: bool) -> char {
    if filled {
        FILLED
    } else {
        EMPTY
    }
}

fn main() {
    let (columns, rows) = terminal::size().unwrap();
    let mut paused = true;
    let mut table = vec![vec![false; columns as usize]; rows as usize];
    let mut stdout = stdout();
    let mut counter = 0;

    execute!(stdout, event::EnableMouseCapture);
    execute!(stdout, cursor::Hide);

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        read_events(tx);
    });
    
    'main: loop {
        loop {
            match rx.try_recv() {
                Ok(x) => match x {
                    Event::Break() => {
                        break 'main;
                    }
                    Event::Pause() => {
                        paused = !paused;
                    }
                    Event::Click(x, y) => {
                        table[x as usize][y as usize] = !table[x as usize][y as usize];
                    }
                },
                Err(_) => {
                    break;
                }
            }
        }
        let instant = std::time::Instant::now();
        counter += 1;

        if !paused && (counter % FRAMES_PER_UPDATE == 0) {
            update_table(&mut table);
            counter = 0;
        }

        execute!(stdout, crossterm::cursor::MoveTo(0, 0));
        for row in &table {
            for &cell in row {
                queue!(stdout, crossterm::style::Print(fill(cell)));
            }
        }
        stdout.flush().unwrap();
        let time = instant.elapsed();
        if time < TIME_FRAME {
            thread::sleep(TIME_FRAME - time);
        }
    }
}

fn update_table(table: &mut Vec<Vec<bool>>) {
    let mut place_holder = table.clone();
    for (x, row) in table.iter().enumerate() {
        for (y, _) in row.iter().enumerate() {
            let mut live_cells: u32 = 0;
            for dir in DIRECTIONS {
                if let Some(x) = table.get(((x as i32) + dir.0) as usize) {
                    if let Some(flag) = x.get(((y as i32) + dir.1) as usize) {
                        if *flag {
                                live_cells += 1;
                        }
                    }
                }
            }

            if (live_cells == 2 || live_cells == 3) && table[x][y] {
                place_holder[x][y] = true;
            } else if live_cells == 3 && !table[x][y] {
                place_holder[x][y] = true;
            } else {
                place_holder[x][y] = false;
            }
        }
    }

    *table = place_holder.clone();
}

fn read_events(tx: mpsc::Sender<Event>) {
    loop {
        let ev = read().unwrap();
        match ev {
            event::Event::Mouse(x) => match x.kind {
                event::MouseEventKind::Down(_) => {
                    tx.send(Event::Click(x.row as i32, x.column as i32))
                        .unwrap();
                }
                event::MouseEventKind::Drag(_) => {
                    tx.send(Event::Click(x.row as i32, x.column as i32))
                        .unwrap();
                }
                _ => (),
            },
            event::Event::Key(x) => {
                if x.code == event::KeyCode::Char('c')
                    && x.modifiers == event::KeyModifiers::CONTROL
                {
                    tx.send(Event::Break()).unwrap();
                } else if x.code == event::KeyCode::Char('p') {
                    tx.send(Event::Pause()).unwrap();
                };
            }
            _ => (),
        }
    }
}
