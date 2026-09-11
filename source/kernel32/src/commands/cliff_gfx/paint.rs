// © Realix > Cliff (graphical): приложение "Paint"
// ================
// ❗️ Только в графическом Cliff - "квадраты" тут буквально закрашенные
// участки холста (пиксели), у текстового Cliff нет для этого сетки
// ❗️ Как и все окна Cliff, каждый кадр отрисовывается заново из состояния
// (cells/painted), а не поверх предыдущего кадра - курсору не нужно
// отдельно "стирать" себя со старой позиции, он просто рисуется поверх
// свежего кадра холста
// ❗️ Окно Paint можно менять в размере (Ctrl+Shift+WASD) - при этом
// становится видно больше клеток одного и того же буфера фиксированного
// размера (MAX_COLS x MAX_ROWS), а не выделяется/пересоздаётся холст
// заново - см. cliff_gfx::paint_visible_size. Двигать окно по-прежнему
// нельзя (см. cliff_gfx::handle_top - `movable`, отдельно от `resizable`)

use crate::drivers::vga::Color;

pub const CELL_PX: usize = 8;

// Ёмкость буфера - с запасом под самое большое окно, которое допускает
// cliff_gfx::resize_bounds для Paint; РЕАЛЬНО видимая (и доступная для
// рисования) часть меньше и зависит от текущего размера окна - см.
// cliff_gfx::paint_visible_size
pub const MAX_COLS: usize = 52;
pub const MAX_ROWS: usize = 32;

const PALETTE: [Color; 8] = [
    Color::White, Color::Red, Color::Yellow, Color::Green,
    Color::Cyan, Color::Blue, Color::Magenta, Color::LightGray,
];

const PALETTE_NAMES: [&str; 8] = [
    "WHITE", "RED", "YELLOW", "GREEN", "CYAN", "BLUE", "MAGENTA", "LGRAY",
];

pub struct PaintApp {
    cells: [[Color; MAX_COLS]; MAX_ROWS],
    painted: [[bool; MAX_COLS]; MAX_ROWS],
    pub cursor_col: usize,
    pub cursor_row: usize,
    color_index: usize,
}

impl PaintApp {
    pub fn new() -> Self {
        PaintApp {
            cells: [[Color::Black; MAX_COLS]; MAX_ROWS],
            painted: [[false; MAX_COLS]; MAX_ROWS],
            cursor_col: 0,
            cursor_row: 0,
            color_index: 0,
        }
    }

    /// Ставит курсор напрямую (напр. под точку, куда наведена мышь) -
    /// см. cliff_gfx::paint_cell_at_point/run() про рисование мышью.
    /// paint_cell_at_point уже проверяет попадание в ВИДИМУЮ область сам,
    /// так что здесь достаточно ограничить только ёмкостью буфера
    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(MAX_ROWS - 1);
        self.cursor_col = col.min(MAX_COLS - 1);
    }

    pub fn move_up(&mut self) { self.cursor_row = self.cursor_row.saturating_sub(1); }
    pub fn move_left(&mut self) { self.cursor_col = self.cursor_col.saturating_sub(1); }

    /// Вниз/вправо - ограничены ВИДИМОЙ (см. visible_rows/cols - текущий
    /// размер окна), а не максимальной ёмкостью буфера: иначе курсор мог
    /// бы уйти туда, где холст ещё не открыт увеличением окна
    pub fn move_down(&mut self, visible_rows: usize) {
        self.cursor_row = (self.cursor_row + 1).min(visible_rows.saturating_sub(1)).min(MAX_ROWS - 1);
    }
    pub fn move_right(&mut self, visible_cols: usize) {
        self.cursor_col = (self.cursor_col + 1).min(visible_cols.saturating_sub(1)).min(MAX_COLS - 1);
    }

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
        if row < MAX_ROWS && col < MAX_COLS && self.painted[row][col] {
            Some(self.cells[row][col])
        } else {
            None
        }
    }
}
