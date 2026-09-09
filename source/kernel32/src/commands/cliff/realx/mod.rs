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
    "x = 5             # assign (int)",
    "y = x + 3 * (2-1) # + - * / ()",
    "print(y)          # print number",
    "print(\"hi\")       # print text",
    "# comment",
    "if x > 3:",
    "    print(x)",
    "end",
    "while x > 0:",
    "    x = x - 1",
    "end",
    "Cmp: == != < <= > >=",
    "No elif/else/def - Esc closes",
];
