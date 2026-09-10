// © Realix > Cliff (graphical): приложение "Paint"
// ================
// ❗️ Только в графическом Cliff - "квадраты" тут буквально закрашенные
// участки холста (пиксели), у текстового Cliff нет для этого сетки
// ❗️ Как и все окна Cliff, каждый кадр отрисовывается заново из состояния
// (cells/painted), а не поверх предыдущего кадра - курсору не нужно
// отдельно "стирать" себя со старой позиции, он просто рисуется поверх
// свежего кадра холста
// ❗️ Окно Paint намеренно НЕ входит в общий Ctrl+WASD/стрелки-двигают-окно
// (см. cliff_gfx::handle_top) - по просьбе: окно фиксировано, стрелки -
// только курсор рисования, а не перемещение/размер самого окна

use crate::drivers::vga::Color;

pub const CELL_PX: usize = 8;
pub const COLS: usize = 26;
pub const ROWS: usize = 12;

const PALETTE: [Color; 8] = [
    Color::White, Color::Red, Color::Yellow, Color::Green,
    Color::Cyan, Color::Blue, Color::Magenta, Color::LightGray,
];

const PALETTE_NAMES: [&str; 8] = [
    "WHITE", "RED", "YELLOW", "GREEN", "CYAN", "BLUE", "MAGENTA", "LGRAY",
];

pub struct PaintApp {
    cells: [[Color; COLS]; ROWS],
    painted: [[bool; COLS]; ROWS],
    pub cursor_col: usize,
    pub cursor_row: usize,
    color_index: usize,
}

impl PaintApp {
    pub fn new() -> Self {
        PaintApp {
            cells: [[Color::Black; COLS]; ROWS],
            painted: [[false; COLS]; ROWS],
            cursor_col: 0,
            cursor_row: 0,
            color_index: 0,
        }
    }

    pub fn move_up(&mut self) { self.cursor_row = self.cursor_row.saturating_sub(1); }
    pub fn move_down(&mut self) { self.cursor_row = (self.cursor_row + 1).min(ROWS - 1); }
    pub fn move_left(&mut self) { self.cursor_col = self.cursor_col.saturating_sub(1); }
    pub fn move_right(&mut self) { self.cursor_col = (self.cursor_col + 1).min(COLS - 1); }

    /// Ставит квадрат текущего цвета в клетку под курсором
    pub fn place_square(&mut self) {
        self.cells[self.cursor_row][self.cursor_col] = self.current_color();
        self.painted[self.cursor_row][self.cursor_col] = true;
    }

    /// Следующий цвет палитры (по кругу)
    pub fn cycle_color(&mut self) {
        self.color_index = (self.color_index + 1) % PALETTE.len();
    }

    pub fn current_color(&self) -> Color {
        PALETTE[self.color_index]
    }

    pub fn current_color_name(&self) -> &'static str {
        PALETTE_NAMES[self.color_index]
    }

    /// Цвет клетки, если она закрашена
    pub fn cell(&self, row: usize, col: usize) -> Option<Color> {
        if self.painted[row][col] { Some(self.cells[row][col]) } else { None }
    }
}
