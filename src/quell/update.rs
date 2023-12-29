use super::{cells::Grid, manipulation::{push, rotate_by, can_generate}, direction::Direction, cell_data::{MOVER, GENERATOR, ROTATOR_CCW, ROTATOR_CW}};

macro_rules! loop_each {
    (for $x:ident, $y:ident, $name:ident in $grid:expr; $code:block) => {
        for $y in 0..$grid.height as isize {
            for $x in 0..$grid.width as isize {
                if let Some($name) = $grid.get_mut($x, $y) {
                    $code
                }
            }
        }
    };
}

macro_rules! loop_each_dir {
    (for $dir:ident $({ $($s:stmt;)* })?, $x:ident, $y:ident, $name:ident in $grid:expr; $code:block) => {
        for $dir in [
            Direction::Right,
            Direction::Left,
            Direction::Up,
            Direction::Down,
        ] {
            $($( $s )*)?
            if $dir == Direction::Right || $dir == Direction::Up {
                let mut $y = $grid.height as isize - 1;
                while $y >= 0 {
                    let mut $x = $grid.width as isize - 1;
                    while $x >= 0 {
                        if let Some($name) = $grid.get_mut($x, $y) {
                            $code
                        }
                        $x -= 1;
                    }
                    $y -= 1;
                }
            }
            else {
                for $y in 0..$grid.height as isize {
                    for $x in 0..$grid.width as isize {
                        if let Some($name) = $grid.get_mut($x, $y) {
                            $code
                        }
                    }
                }
            }
        }
    };
}

/// Performs a single update step.
pub fn update(grid: &mut Grid) {
    let mut cell_flags = 0u64;

    for y in 0..grid.height as isize {
        for x in 0..grid.width as isize {
            if let Some(cell) = grid.get_mut(x, y) {
                cell.set_updated(false);
                cell_flags |= 1 << cell.id();
            }
        }
    }

    macro_rules! subticks {
        ($( $($cell:ident),*: $fn_name:ident)* ) => {
            $( if cell_flags & ($(1 << $cell)|*) != 0 { $fn_name(grid); } )*
        }
    }

    subticks! {
        GENERATOR       : do_gens
        ROTATOR_CW, ROTATOR_CCW: do_rotators
        MOVER           : do_movers
    }

    grid.tick_count += 1;
}

#[inline(never)]
fn do_gens(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let push_offset = dir.to_vector();
        let cell_offset = dir.flip().to_vector();
    }, x, y, cell in grid; {
        if cell.id() == GENERATOR && cell.direction() == dir && !cell.updated() {
            cell.set_updated(true);
            if let Some(cell) = grid.get(x + cell_offset.x, y + cell_offset.y) {
                if can_generate(cell) {
                    push(grid, x + push_offset.x, y + push_offset.y, dir, 1, Some(cell.clone()), false);
                }
            }
        }
    });
}

#[inline(never)]
fn do_rotators(grid: &mut Grid) {
    loop_each!(for x, y, cell in grid; {
        if !cell.updated() {
            if cell.id() == ROTATOR_CW {
                cell.set_updated(true);
                rotate_by(grid, x + 1, y, Direction::Down, Direction::Left);
                rotate_by(grid, x, y - 1, Direction::Down, Direction::Up);
                rotate_by(grid, x - 1, y, Direction::Down, Direction::Right);
                rotate_by(grid, x, y + 1, Direction::Down, Direction::Down);
            }
            else if cell.id() == ROTATOR_CCW {
                cell.set_updated(true);
                rotate_by(grid, x + 1, y, Direction::Up, Direction::Left);
                rotate_by(grid, x, y - 1, Direction::Up, Direction::Up);
                rotate_by(grid, x - 1, y, Direction::Up, Direction::Right);
                rotate_by(grid, x, y + 1, Direction::Up, Direction::Down);
            }
        }
    });
}

#[inline(never)]
fn do_movers(grid: &mut Grid) {
    loop_each_dir!(for dir, x, y, cell in grid; {
        if cell.id() == MOVER && cell.direction() == dir && !cell.updated() {
            cell.set_updated(true);
            push(grid, x, y, dir, 0, None, true);
        }
    });
}
