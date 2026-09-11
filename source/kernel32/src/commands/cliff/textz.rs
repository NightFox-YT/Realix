// © Realix > Cliff: приложение "TextZ" (ультра-базовый текстовый редактор)
// ================
// ❗️ См. cliff::editor - фиксированный буфер, без вставки/сдвига строк, ввод
// в режиме перезаписи. Никакого сохранения на диск (нет файловой системы)

use super::editor::Editor;

pub struct TextZApp {
    pub editor: Editor,
}

impl TextZApp {
    pub fn new() -> Self {
        TextZApp { editor: Editor::new() }
    }
}
