use speedy2d::dimen::Vector2;
use crate::quell::{direction::Direction, cells::{Cell, Grid}, cell_data::{WALL, SLIDE, MOVER, TRASH, ENEMY}};

/// A force a cell is moved with.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveForce {
    Push
}

/// The result when pushing a cell.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PushResult {
    Moved,
    NotMoved,
    Trashed,
}

/// Checks if a cell can move in a certain direction with the given force.
#[inline]
pub fn can_move(cell: &Cell, direction: Direction, _force: MoveForce) -> bool {
    match cell.id() {
        WALL  => false,
        SLIDE if cell.direction() % 2 != direction % 2 => false,
        _ => true,
    }
}

/// Checks if a cell is a trash in the direction it is being pushed.
#[inline]
pub fn is_trash(cell: &Cell, _direction: Direction) -> bool {
    match cell.id() {
        TRASH | ENEMY => true,
        _ => false,
    }
}

/// Checks if a cell can be generated.
#[inline(always)]
pub fn can_generate(_cell: &Cell) -> bool {
    true
}

/// Pushes the specified cell in a direction. Returns whether the cell was moved.
/// You can also specify a replacement cell that should be put where the old one was.
// #[inline(never)]
pub fn push(grid: &mut Grid, x: isize, y: isize, dir: Direction, mut force: usize, pushing: Option<Cell>, setupdated: bool) -> PushResult {
    let mut tx = x;
    let mut ty = y;

    let orig_dir = dir;
    let mut dir = dir;

    // Check if the cell can be pushed.
    loop {
        if !grid.is_in_bounds(tx, ty) { return PushResult::NotMoved; }

        let cell = grid.get(tx, ty);
        if let Some(cell) = cell {
            if cell.id() == MOVER {
                if cell.direction() == dir {
                    force += 1;
                }
                else if cell.direction() == dir.flip() {
                    force -= 1;
                }
            }

            if is_trash(cell, dir) { break; }

            if !can_move(cell, dir, MoveForce::Push) {
                return PushResult::NotMoved;
            }

            let Vector2 { x: ox, y: oy } = dir.to_vector();
            tx += ox;
            ty += oy;
        }
        else {
            break;
        }

        if force == 0 { return PushResult::NotMoved; }
        if tx == x && ty == y && dir == orig_dir { break; }
    }

    // Push the cell and all following.
    // Works like this:
    //  >=#   replacement cell is air
    //  ^
    // replace cell with air and store the old cell in the replacement cell
    // then go forward one cell
    //   =#   replacement cell is mover
    //   ^
    // repeat ^
    //   >#   replacement cell is slide
    //   >=   replacement cell is push
    //   >=#
    // we moved forward one cell!

    dir = orig_dir;
    let mut x = x;
    let mut y = y;
    let mut next_cell = pushing;
    let mut push_result = PushResult::Trashed;
    loop {
        if let Some(ref mut cell) = next_cell {
            // Update mover cell `.updated`.
            if cell.id() == MOVER && cell.direction() == dir && setupdated {
                cell.set_updated(true);
            }
        }

        if let Some(cell) = grid.get_mut(x, y) {
            // When trash then break.
            if cell.id() == ENEMY {
                // Cell is deleted and enemy destroyed.
                grid.delete(x, y);
                break;
            }
            else if is_trash(cell, dir) {
                // Cell is trashed.
                break;
            }
        }

        // Push cell and store current one in next push replacement.
        push_result = PushResult::Moved;
        let old_cell = grid.take(x, y);
        grid.set_cell(x, y, next_cell);
        next_cell = old_cell;
        if tx == x && ty == y { break; }

        let Vector2 { x: ox, y: oy } = dir.to_vector();
        x += ox;
        y += oy;
    }

    push_result
}

/// Checks if a cell can be rotated from a specific direction.
#[inline]
pub fn can_rotate(cell: &Cell, _side: Direction) -> bool {
    match cell.id() {
        WALL => false,
        _ => true,
    }
}

// internal helper
#[inline(always)]
fn rotate(cell: &mut Cell, dir: Direction, side: Direction) -> bool {
    if can_rotate(cell, side) {
        cell.set_direction(dir);
        true
    }
    else {
        false
    }
}

/// Rotates a cell by a specific amount.
#[inline(always)]
// #[inline(never)]
pub fn rotate_by(grid: &mut Grid, x: isize, y: isize, dir: Direction, side: Direction) -> bool {
    match grid.get_mut(x, y) {
        Some(cell) => rotate(cell, cell.direction() + dir, side),
        None => false,
    }
}