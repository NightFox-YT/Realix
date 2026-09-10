// © Realix > Cliff: приложение "RealX IDE"
// ================
// ❗️ '\' открывает документацию (RealXDocs), '`' запускает написанную
// программу в НОВОМ окне (RealXOutput) - см. commands::cliff (Layer/Action)

pub mod lang;

use super::editor::Editor;

pub struct RealXIde {
    pub editor: Editor,
}

impl RealXIde {
    pub fn new() -> Self {
        RealXIde { editor: Editor::new() }
    }
}

/// Текст документации, показываемый по '\' - должен помещаться в окно
/// (см. cliff::DOCS_LINES/editor::MAX_LINES)
pub const DOCS_TEXT: &[&str] = &[
    "RealX - ultra basic, Python-like",
    "x = 5   s = \"hi\"+name   # int / str",
    "name = input(\"Name: \")  # reads a line",
    "if x > 3:",
    "    print(x)",
    "elif x == 0:",
    "    print(\"zero\")",
    "else:",
    "    print(\"neg\")",
    "end   # while: only condition, no elif/else",
    "and or not %  == != < <= > >=  += -= *= /=",
    "mouse_x/y() mouse_down() pixel(x,y,c) cls(c)",
    "beep(hz,ms) wait(ms) ticks() rnd(n) - gfx/hw",
    "No user functions (def) - Esc closes",
];
