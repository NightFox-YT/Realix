// © Realix > Cliff: общий буфер строк-редактора (TextZ, RealX IDE)
// ================
// ❗️ "Ультра-базовый" по задумке - фиксированный размер, без вставки/сдвига
// строк: Enter просто переносит курсор на начало след. строки, а не
// вставляет новую (кол-во строк всегда MAX_LINES); ввод символа - в режиме
// перезаписи (не вставки), не нужно сдвигать хвост строки при печати
// ❗️ Никакого сохранения на диск - в kernel32 пока нет файловой системы,
// буфер живёт только пока открыто окно приложения (TextZ/RealX IDE)

pub const LINE_LEN: usize = 46;
pub const MAX_LINES: usize = 14;

pub struct Editor {
    lines: [[u8; LINE_LEN]; MAX_LINES],
    line_len: [usize; MAX_LINES],
    pub cur_row: usize,
    pub cur_col: usize,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            lines: [[0; LINE_LEN]; MAX_LINES],
            line_len: [0; MAX_LINES],
            cur_row: 0,
            cur_col: 0,
        }
    }

    pub fn line_str(&self, row: usize) -> &str {
        core::str::from_utf8(&self.lines[row][..self.line_len[row]]).unwrap_or("")
    }

    pub fn move_up(&mut self) {
        if self.cur_row > 0 {
            self.cur_row -= 1;
            self.cur_col = self.cur_col.min(self.line_len[self.cur_row]);
        }
    }

    pub fn move_down(&mut self) {
        if self.cur_row + 1 < MAX_LINES {
            self.cur_row += 1;
            self.cur_col = self.cur_col.min(self.line_len[self.cur_row]);
        }
    }

    pub fn move_left(&mut self) {
        if self.cur_col > 0 {
            self.cur_col -= 1;
        } else if self.cur_row > 0 {
            self.cur_row -= 1;
            self.cur_col = self.line_len[self.cur_row];
        }
    }

    pub fn move_right(&mut self) {
        if self.cur_col < self.line_len[self.cur_row] {
            self.cur_col += 1;
        } else if self.cur_row + 1 < MAX_LINES {
            self.cur_row += 1;
            self.cur_col = 0;
        }
    }

    /// Backspace - только внутри строки, не сливает её с предыдущей (см.
    /// заголовок файла - строки здесь фиксированные, а не динамический список)
    pub fn backspace(&mut self) {
        if self.cur_col > 0 {
            let row = self.cur_row;
            let len = self.line_len[row];
            for i in (self.cur_col - 1)..(len - 1) {
                self.lines[row][i] = self.lines[row][i + 1];
            }
            self.line_len[row] -= 1;
            self.cur_col -= 1;
        } else if self.cur_row > 0 {
            self.cur_row -= 1;
            self.cur_col = self.line_len[self.cur_row];
        }
    }

    /// Enter - переносит курсор на начало след. строки (не вставляет новую)
    pub fn newline(&mut self) {
        if self.cur_row + 1 < MAX_LINES {
            self.cur_row += 1;
        }
        self.cur_col = 0;
    }

    /// Ввод символа в режиме перезаписи - продлевает длину строки только
    /// если печать происходит в её текущем конце
    pub fn type_char(&mut self, byte: u8) {
        if self.cur_col >= LINE_LEN {
            return;
        }
        let row = self.cur_row;
        self.lines[row][self.cur_col] = byte;
        if self.cur_col == self.line_len[row] {
            self.line_len[row] += 1;
        }
        self.cur_col += 1;
    }
}
