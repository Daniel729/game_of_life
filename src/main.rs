use std::time::Duration;
use crossterm::{event::{read, poll}, event};
use crossterm::terminal;
use std::vec::Vec;
use std::io::stdout;
use crossterm::queue;
use std::io::Write;
// use std::thread;
// use std::sync::mpsc;

const FILLED: char = '\u{2588}';
const EMPTY: char = ' ';
const DIRECTIONS: [(i32, i32); 8] = [(0,1),(0,-1),(1,0),(-1,0),(1,1),(1,-1),(-1,1),(-1,-1)];
const FRAMES_PER_UPDATE : u32 = 1;//: Duration = Duration::from_millis(200);
const TIME_FRAME: Duration = Duration::from_millis(1000 / 60);

fn fill(filled: bool) -> char {
    if filled { FILLED } else { EMPTY }
}

fn main() {
    let (columns, rows) = terminal::size().unwrap();
    let mut paused = true;
    let mut table = vec![vec![false; columns as usize]; rows as usize];

    crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture);
    // print_events();

    let mut stdout = stdout();
    let mut counter = 0;

    loop {
        counter += 1;

        if !paused && (counter % FRAMES_PER_UPDATE == 0){
            update_table(&mut table);
            counter = 0;
        }

        crossterm::execute!(stdout, crossterm::cursor::MoveTo(0, 0));
        for row in &table {
            for &cell in row { 
                queue!(stdout, crossterm::style::Print(fill(cell)));
            }
        }
        
        stdout.flush().unwrap();
        
        if read_events(&mut table, &mut paused) {
            break;
        }
    }
        
}

fn update_table(table: &mut Vec<Vec<bool>>) {
    let mut place_holder = table.clone();
    for (x, row) in table.iter().enumerate() {
        for (y, _) in row.iter().enumerate() {
            let mut live_cells : u32 = 0;
            for dir in DIRECTIONS {
                 match table.get(((x as i32) + dir.0) as usize) {
                     Some(x) => {
                        match x.get(((y as i32) +dir.1) as usize) {
                            Some(flag) => {
                                if *flag {
                                    live_cells += 1;
                                }
                            },
                            None => ()
                        }
                     },
                     None => ()
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

fn read_events(table : &mut Vec<Vec<bool>>, paused : &mut bool) -> bool {
    let instant = std::time::Instant::now();
    while TIME_FRAME > instant.elapsed() {
        if poll(TIME_FRAME - instant.elapsed()).unwrap() {
            let ev = read().unwrap();
            match ev {
                event::Event::Mouse(x) => match x.kind {
                    event::MouseEventKind::Down(_) => {
                        table[x.row as usize][x.column as usize] = !table[x.row as usize][x.column as usize];
                    },
                    event::MouseEventKind::Drag(_) => {
                        table[x.row as usize][x.column as usize] = !table[x.row as usize][x.column as usize];
                        read_events(table, paused); 
                    }
                    _ => ()
                },
                event::Event::Key(x) => {
                    if x.code == event::KeyCode::Char('c') && x.modifiers == event::KeyModifiers::CONTROL {
                        return true;
                    } else if x.code == event::KeyCode::Char('p') {
                        *paused = !(*paused);
                    };
                },
                _ => ()
                }
        }
    }
    // let x = instant.elapsed();
    // if x < TIME_FRAME {
    //     std::thread::sleep(TIME_FRAME - x);
    // } 
    return false;
}