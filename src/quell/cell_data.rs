use crate::quell::cells::CellType;

// helper for easier cell definitions
macro_rules! celld {
    {$(
        $id_name:ident $id:literal {
            $name:literal,
            $description:literal,
            sides $sides:literal,
            texture $texture_name:literal
        }
    )*} => {
        $( pub const $id_name: CellType = $id; )*
        #[allow(dead_code)]
        pub static CELL_DATA: &[CellData] = &[
            $(
                CellData {
                    id: $id_name,
                    name: $name,
                    description: $description,
                    sides: $sides,
                    texture_name: $texture_name,
                },
            )*
        ];
    }
}

celld! {
    WALL 1 {
        "Wall",
        "A solid wall that can't be moved by anything.",
        sides 1,
        texture "wall"
    }
    MOVER 2 {
        "Mover",
        "Pushes the cells in front of it.",
        sides 4,
        texture "mover"
    }
    GENERATOR 3 {
        "Generator",
        "Generates the cell behind to its front.",
        sides 4,
        texture "generator"
    }
    ROTATOR_CW 4 {
        "Rotator CW",
        "Rotates all touching cells clockwise.",
        sides 1,
        texture "rotator_cw"
    }
    ROTATOR_CCW 5 {
        "Rotator CCW",
        "Rotates all touching cells counter-clockwise.",
        sides 1,
        texture "rotator_ccw"
    }
    PUSH 6 {
        "Push",
        "A normal cell that does nothing.",
        sides 1,
        texture "push"
    }
    SLIDE 7 {
        "Slide",
        "Like push cell but can only be moved in two directions.",
        sides 2,
        texture "slide"
    }
    TRASH 8 {
        "Trash",
        "Trashes all cells that get moved into it.",
        sides 1,
        texture "trash"
    }
    ENEMY 9 {
        "Enemy",
        "An enemy that moves randomly. *thanks github copilot*",
        sides 1,
        texture "enemy"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellData {
    pub id: CellType,
    pub name: &'static str,
    pub description: &'static str,
    pub sides: usize,
    pub texture_name: &'static str,
}
