mod quell;
mod builtin;

use std::io::{stdout, Stdout, Write};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc::channel;

use quell::codes::import;
use quell::update::update;
use quell::cells::Grid;
use quell::cells::Cell;

use crossterm::{
    execute,
    queue,
    style::*,
    terminal::*,
    event::*,
    cursor::*
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Preview a Cell Machine level in the terminal! :staring_cat:", long_about = None)]
struct Args {
    /// V3 only. You can set it to 'ao' for automata opus
    #[arg(short, long)]
    level: String,
    
    /// The amount of time to wait before updating the level in milliseconds
    #[arg(short, long, default_value_t = 200)]
    sleep: u64,

    #[arg(short, long, default_value_t = false)]
    nerd: bool
}

fn cell_to_ascii(cell: Cell) -> char {
    match cell.id() {
        1 => "■■■■",
        2 => "→↓←↑",
        3 => "⇒⇓⇐⇑",
        4 => "↻↻↻↻",
        5 => "↺↺↺↺",
        6 => "####",
        7 => "=‖=‖",
        8 => "🗑🗑🗑🗑",
        9 => "××××",
        _ => todo!()
    }.chars().nth(cell.direction() as usize).unwrap()
}

fn cell_to_color(cell: Cell) -> Color {
    match cell.id() {
        1 => Color::Rgb { r: 88,  g: 88, b: 88 },
        2 => Color::Rgb { r: 76,  g: 121, b: 216 },
        3 => Color::Rgb { r: 2,   g: 205, b: 113 },
        4 => Color::Rgb { r: 225, g: 103, b: 1 },
        5 => Color::Rgb { r: 0,   g: 203, b: 182 },
        6 => Color::Rgb { r: 246, g: 194, b: 57 },
        7 => Color::Rgb { r: 246, g: 194, b: 57 },
        8 => Color::Rgb { r: 155,  g: 0,  b: 206 },
        9 => Color::Rgb { r: 208,  g: 12,  b: 34 },
        _ => todo!()
    }
}

fn render(grid: Grid, mut stdout: &Stdout) {
    /*let mut b1 = true;
    let mut b2 = true;*/
    for y in 0..grid.height {
        for x in 0..grid.width {
            let (cell, cell_color)= if let Some(c) = grid.get(x.try_into().unwrap(), (grid.height - 1 - y).try_into().unwrap()) {
                (cell_to_ascii(c.clone()), cell_to_color(c.clone()))
            } else {
                (' ', Color::White)
            };
            let color = Color::Rgb { r: 0, g: 0, b: 0 };
            /*if b2 {
                Color::AnsiValue(232)
            } else {
                Color::AnsiValue(235)
            };*/
            queue!(
                stdout,
                SetBackgroundColor(color),
                SetForegroundColor(cell_color),
                Print(format!("{} ", cell)),
                ResetColor
            ).unwrap();
            //b2 = !b2;
        }
        /*b1 = !b1;
        b2 = b1;*/
        queue!(
            stdout,
            MoveToColumn(0),
            MoveDown(1),
        ).unwrap();
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let level = match args.level.as_str() {
        "ao" => builtin::AO,
        "clock" => builtin::CLOCK,
        _ => &args.level,
    };
    let mut grid: Grid;
    // if let Some(level) = args.get(1) {
    //     let mut lvl: &str = level;
    //     if level == "ao" {
    //         lvl = AO;
    //     } else if level == "help" {
    //         println!(":staring_cat:\nFirst argument is the level. You can set it to 'ao' for automata opus\nSecond argument (optional) is the amount of time to wait before updating the level in milliseconds. By default it is 200.");
    //         return Ok(());
    //     }
    let g = import(level);
    
    match g {
        Ok(g) => grid = g,
        Err(err) => {
            println!("Invalid level: {}. :staring_cat:", err);
            return Ok(());
        }
    }
    
    let (width, height) = size()?;
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture, EnableFocusChange)?;

    execute!(
        stdout(),
        SetTitle("Cell Machine"),
        SetSize((grid.width*2).try_into().unwrap(), (grid.height+1).try_into().unwrap()),
        Hide,
    )?;

    let (tx, rx) = channel::<()>();
    thread::spawn(move || {
        grid.set(0, 0, Cell::new(2, quell::direction::Direction::Right));
        let mut tps: i32 = 0;
        let mut ticks = 0;
        let mut tps_time = std::time::Instant::now();
        loop {
            match rx.try_recv() {
                Ok(_) => break,
                Err(_) => {}
            }

            if tps_time.elapsed() > Duration::from_secs(1) {
                tps = ticks;
                ticks = 0;
                tps_time = std::time::Instant::now();
            }

            let mut stdout = stdout();
            queue!(stdout, MoveTo(0, 1)).unwrap();

            let _render_time = Instant::now();
            render(grid.clone(), &stdout);
            let render_time = _render_time.elapsed().as_millis();

            let _update_time = Instant::now();
            update(&mut grid);
            ticks += 1;
            let update_time = _update_time.elapsed().as_millis();

            let nerd_sleep = args.sleep.saturating_sub(render_time as u64).saturating_sub(update_time as u64);
            let actual_sleep = if args.nerd == true { nerd_sleep } else { args.sleep };
            queue!(
                stdout,
                crossterm::cursor::MoveTo(0, 0),
                Clear(ClearType::CurrentLine),
                Print(format!(
                    "Tick: {}, TPS: {}, Render: {}ms, Update: {}ms, Sleep: actual = {}ms | user = {}ms | nerd = {}ms",
                    grid.tick_count,
                    tps,
                    render_time,
                    update_time,
                    actual_sleep,
                    args.sleep,
                    nerd_sleep
                ))
            ).unwrap();
            thread::sleep(Duration::from_millis(actual_sleep));
            stdout.flush().unwrap();
        }
    });
    
    // TODO: Use poll() instead of read() to get rid of the render thread
    loop {
        match read().unwrap() {
            Event::Key(k) => {
                // Stop the program if escape or ctrl-c is pressed
                if k.code == KeyCode::Esc || (k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL)) {
                    tx.send(()).unwrap();
                    break;
                }
            },
            _ => {}
        }
    }

    // The terminal does some weird stuff when stuff gets written to stdout while we are disabling stuff so we have to flush
    // Update: Now the flush alone doesnt work either so i added the sleeps
    thread::sleep(Duration::from_millis(100));
    stdout().flush()?;
    thread::sleep(Duration::from_millis(100));

    execute!(stdout(), Clear(ClearType::All), SetSize(width, height), Show, DisableFocusChange, DisableMouseCapture, LeaveAlternateScreen)?;
    
    disable_raw_mode()
}