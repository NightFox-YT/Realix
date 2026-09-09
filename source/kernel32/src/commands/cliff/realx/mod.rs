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
    "x = 5             # int",
    "s = \"hi\" + name    # str: '' or \"\", + concat",
    "name = input(\"Name: \")   # reads a line",
    "print(x)          # print(expr) - int or str",
    "if x > 3:",
    "    print(x)",
    "end",
    "while x > 0:",
    "    x = x - 1",
    "end",
    "Cmp == != < <= > >=  (int-int or str-str)",
    "-,*,/ only int; + also concats str",
    "No elif/else/def - Esc closes",
];
