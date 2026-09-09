// © Realix > Real Mode: BIOS video mode thunk
// (09.09.26) v0.1
// ================
// ❗️ Kernel32 работает в защищённом режиме и не может напрямую вызывать
//    BIOS-прерывания (int 10h) - для смены видеорежима эта функция временно
//    выключает Protected Mode (сбрасывает CR0.PE), делает вызов BIOS в Real
//    Mode и возвращается обратно. Работает только т.к. paging не используется
//    (линейный адрес = физический) и kernel32 физически загружен по адресу
//    0x10000 - т.е. полностью в пределах 1 МБ, доступных в Real Mode.
// ❗️ Смена видеорежима не ждёт завершающего аппаратного IRQ, как дисковые
//    BIOS-вызовы - но некоторые реализации BIOS (напр. в VirtualBox) всё
//    равно опрашивают тик таймера (0040:0006Ch, обновляется обработчиком
//    IRQ0) для внутренней задержки при смене режима (напр. ожидание
//    стабилизации DAC/CRTC). Если держать прерывания выключенными (cli) на
//    всё время вызова, этот тик никогда не обновится - BIOS зависает в
//    цикле опроса навечно (симптом: "печатает Starting..., дальше тишина").
//    Поэтому, как и в дисковом thunk'е, включаем прерывания (sti) и
//    временно возвращаем PIC на "родные" смещения BIOS - см. set_mode().
// ❗️ Прямые ссылки на символы (`[SYMBOL]`) из 16-битного кода не работают:
//    линкер требует, чтобы 16-битное смещение само по себе влезало в 16 бит,
//    а весь код/данные kernel32 линкуются с базы 0x10000 (уже больше 64 КБ).
//    Поэтому единственная область параметров (RmVideoParams) адресуется
//    через регистр esi (32-битная база, вычисленная через `lea` ещё в
//    защищённом режиме) + маленькое константное смещение поля - смещения
//    не требуют релокации, а esi переживает все переходы между режимами.
// ❗️ Требует, чтобы весь двоичный образ kernel32.bin оставался < 64 КБ -
//    иначе смещения меток внутри 16-битного дескриптора (база 0x10000,
//    лимит 0xFFFF) перестанут влезать.

use core::mem::offset_of;
use crate::x86::gdt::REALMODE_CODE_SELECTOR;

// Размер временного стека Real Mode
const RM_STACK_SIZE: usize = 512;

/// Единая область параметров/рабочих переменных Real Mode вызова.
/// Адресуется из 16-битного кода через esi + offset_of!(...) - см. заголовок файла.
#[repr(C, packed)]
struct RmVideoParams {
    // --- Заполняется вызывающей стороной перед rm_bios_video_call ---
    mode: u8, // Номер видеорежима (al для int 10h ah=0)

    // --- Предвычисленные смещения меток перехода ---
    pm16_offset: u16,
    pm16_return_offset: u16,
    realmode_offset: u16,
    pm32_offset: u32,
    stack_top_offset: u16,

    // --- IDTR: сохранённый (защищённого режима) и временный (Real Mode) ---
    saved_idtr_limit: u16,
    saved_idtr_base: u32,
    real_idtr_limit: u16,
    real_idtr_base: u32,

    // --- Сохранённый ESP вызывающей стороны (16-битный `mov sp` ниже
    // трогает только младшую половину ESP - без явного восстановления
    // popfd/popad в конце читали бы не тот стек) ---
    saved_esp: u32,
}

#[no_mangle]
static mut RM_VIDEO_PARAMS: RmVideoParams = RmVideoParams {
    mode: 0,
    pm16_offset: 0, pm16_return_offset: 0, realmode_offset: 0,
    pm32_offset: 0, stack_top_offset: 0,
    saved_idtr_limit: 0, saved_idtr_base: 0,
    // IVT-совместимый IDTR: настоящий IVT по 0000:0000, 256 записей x 4 байта
    real_idtr_limit: 0x03FF, real_idtr_base: 0,
    saved_esp: 0,
};

// Временный стек для работы в Real Mode (собственный кадр kernel32,
// отдельный от любого другого real-mode перехода в проекте)
#[no_mangle]
static mut RM_VIDEO_STACK: [u8; RM_STACK_SIZE] = [0; RM_STACK_SIZE];

unsafe extern "C" {
    /// Единственная точка входа: сохраняет состояние, уходит в Real Mode,
    /// вызывает int 10h ah=0 (смена видеорежима, al = RM_VIDEO_PARAMS.mode),
    /// возвращается в Protected Mode.
    fn rm_bios_video_call();
}

core::arch::global_asm!(
    ".global rm_bios_video_call",
    ".section .text",
    ".code32",
    "rm_bios_video_call:",
    "    pushad",
    "    pushfd",
    "    cli",

    // esi - база RM_VIDEO_PARAMS на всю функцию (пережив ает дальние
    // переходы - никакой код здесь его не трогает)
    "    lea esi, [RM_VIDEO_PARAMS]",

    // Сохраняем ESP вызывающей стороны
    "    mov [esi + {off_saved_esp}], esp",

    // Сохраняем текущий (защищённого режима) IDTR
    "    sidt [esi + {off_saved_idtr}]",

    // Предвычисляем смещения меток ещё в безопасном 32-битном контексте:
    // - pm16_offset / pm16_return_offset / realmode_offset / stack_top_offset:
    //   относительно базы 0x10000 (16-битный дескриптор GDT И сегмент 0x1000
    //   в настоящем Real Mode - та же база) - маскируем в 16 бит
    // - pm32_offset: полный 32-битный адрес (сегмент kernel code, база 0)
    "    lea eax, [pm16_entry]",
    "    and eax, 0xFFFF",
    "    mov [esi + {off_pm16_offset}], ax",

    "    lea eax, [rm_entry]",
    "    and eax, 0xFFFF",
    "    mov [esi + {off_realmode_offset}], ax",

    "    lea eax, [pm16_return]",
    "    and eax, 0xFFFF",
    "    mov [esi + {off_pm16_return_offset}], ax",

    "    lea eax, [pm32_entry]",
    "    mov [esi + {off_pm32_offset}], eax",

    "    lea eax, [RM_VIDEO_STACK]",
    "    add eax, {stack_size}",
    "    and eax, 0xFFFF",
    "    mov [esi + {off_stack_top_offset}], ax",

    // --- Переход A: 32-битный PM -> 16-битный PM (доп. дескриптор GDT) ---
    "    push {rm_code_sel}",
    "    movzx eax, word ptr [esi + {off_pm16_offset}]",
    "    push eax",
    "    retf",

    ".code16",
    "pm16_entry:",
    // ds всё ещё kernel_data_sel (0x10, база 0) - не менялся с самого вызова,
    // поэтому esi-адресация (esi хранит АБСОЛЮТНЫЙ адрес RM_VIDEO_PARAMS)
    // пока корректна без изменений
    "    mov bx, [esi + {off_realmode_offset}]",
    "    mov dx, [esi + {off_stack_top_offset}]",

    // Сбрасываем PE в CR0 - выходим из Protected Mode
    "    mov eax, cr0",
    "    and eax, 0xFFFFFFFE",
    "    mov cr0, eax",

    // ss/sp настраиваем ДО перехода ниже, чтобы сам push+retf уже использовал
    // правильный (не мусорный) стек. 0x1000*16 = 0x10000 - совпадает с базой
    // 16-битного дескриптора GDT, поэтому уже прочитанное смещение верно
    "    mov ax, 0x1000",
    "    mov ss, ax",
    "    mov sp, dx",

    // Дальний переход довершает вход в настоящий Real Mode
    "    push 0x1000",
    "    push bx",
    "    retf",

    "rm_entry:",
    // ds/es пока хранят "устаревший" protected-mode кэш (0x10, база 0) -
    // это ещё валидно для чтения; явно приводим к сегменту 0 (то же самое
    // по факту) для ясности перед настоящим Real Mode кодом
    "    xor ax, ax",
    "    mov ds, ax",
    "    mov es, ax",

    // IVT-совместимый IDTR (настоящий Real Mode, база 0000:0000).
    // ❗️ Прерывания ВКЛЮЧАЕМ (sti): см. заголовок файла - некоторые BIOS
    // опрашивают тик таймера при смене видеорежима. Безопасно ТОЛЬКО потому,
    // что set_mode() уже перепрошил PIC на "родные" смещения BIOS (08h/70h)
    // перед вызовом - иначе IRQ0 ушёл бы не в настоящий BIOS-обработчик
    "    lidt [esi + {off_real_idtr}]",
    "    sti",

    // --- Сама смена видеорежима: int 10h, ah=0, al=режим ---
    "    mov al, [esi + {off_mode}]",
    "    mov ah, 0x00",
    "    int 0x10",

    // --- Возврат в Protected Mode (двумя шагами, симметрично приходу) ---
    "    cli",
    "    mov eax, cr0",
    "    or eax, 1",
    "    mov cr0, eax",

    // Шаг 1: real mode -> тот же 16-битный PM сегмент. ds всё ещё сегмент 0
    // (не менялся с rm_entry) - валиден для чтения смещения
    "    mov bx, [esi + {off_pm16_return_offset}]",
    "    push {rm_code_sel}",
    "    push bx",
    "    retf",

    "pm16_return:",
    // Шаг 2: 16-битный PM -> обычный 32-битный PM ядра (полный физический
    // адрес). 32-битные регистры дают 32-битный push автоматически (0x66);
    // retf кодируется вручную байтами (0x66 + 0xCB) для гарантии 32-битного
    // размера операнда возврата - иначе в 16-битном контексте он был бы 16-бит
    "    mov eax, [esi + {off_pm32_offset}]",
    "    mov ecx, {kernel_code_sel}",
    "    push ecx",
    "    push eax",
    "    .byte 0x66, 0xCB",

    ".code32",
    "pm32_entry:",
    "    mov ax, {kernel_data_sel}",
    "    mov ds, ax",
    "    mov es, ax",
    "    mov fs, ax",
    "    mov gs, ax",
    "    mov ss, ax",

    // Восстанавливаем IDTR защищённого режима (PIT/клавиатура снова рабочие)
    "    lidt [esi + {off_saved_idtr}]",

    // Восстанавливаем ESP вызывающей стороны - без этого popfd/popad читали
    // бы временный стек Real Mode вместо настоящих сохранённых значений
    "    mov esp, [esi + {off_saved_esp}]",

    "    popfd",
    "    popad",
    "    ret",

    rm_code_sel = const REALMODE_CODE_SELECTOR,
    kernel_code_sel = const crate::x86::gdt::KERNEL_CODE_SELECTOR,
    kernel_data_sel = const crate::x86::gdt::KERNEL_DATA_SELECTOR,
    stack_size = const RM_STACK_SIZE,

    off_mode = const offset_of!(RmVideoParams, mode),
    off_pm16_offset = const offset_of!(RmVideoParams, pm16_offset),
    off_pm16_return_offset = const offset_of!(RmVideoParams, pm16_return_offset),
    off_realmode_offset = const offset_of!(RmVideoParams, realmode_offset),
    off_pm32_offset = const offset_of!(RmVideoParams, pm32_offset),
    off_stack_top_offset = const offset_of!(RmVideoParams, stack_top_offset),
    off_saved_idtr = const offset_of!(RmVideoParams, saved_idtr_limit),
    off_real_idtr = const offset_of!(RmVideoParams, real_idtr_limit),
    off_saved_esp = const offset_of!(RmVideoParams, saved_esp),
);

/// Переключает видеорежим BIOS через Real Mode thunk
/// Параметры:
///  - mode: номер видеорежима (см. shared/config.asm: TEXT_MODE_80x25,
///    VIDEO_MODE_320x200 - то же значение, что уже использует switcher.asm
///    при загрузке через ветку "Protected Mode с Video Mode")
// Порты данных (масок) master/slave PIC - см. x86::pic
const PIC1_DATA: u16 = 0x21;
const PIC2_DATA: u16 = 0xA1;

pub fn set_mode(mode: u8) {
    unsafe {
        RM_VIDEO_PARAMS.mode = mode;

        // См. заголовок файла: перепрошиваем PIC на "родные" смещения BIOS
        // на время вызова, разрешая только IRQ0 (таймер - его тик читает
        // BIOS для внутренней задержки при смене режима); IRQ1 (клавиатура)
        // остаётся замаскированным, чтобы нажатия во время вызова не ушли в
        // буфер BIOS вместо очереди kernel32 (тот же приём, что и в
        // x86::realmode::bios_disk_call - 8259A защёлкивает IRQ по фронту
        // даже в маске, доходя до kernel32 сразу после восстановления маски)
        //
        // Прерывания на время ЭТОЙ перепрошивки отключаем на уровне CPU (не
        // только внутри rm_bios_video_call): иначе в окне между
        // reinit_offsets(BIOS) и входом в асм-функцию залётный IRQ0 ушёл бы
        // на вектор 0x08 - обработчик Double Fault в IDT kernel32, не таймер
        crate::x86::idt::interrupts_disable();

        let saved_mask1 = crate::utils::inb(PIC1_DATA);
        let saved_mask2 = crate::utils::inb(PIC2_DATA);

        crate::x86::pic::reinit_offsets(
            crate::x86::pic::BIOS_PIC1_OFFSET,
            crate::x86::pic::BIOS_PIC2_OFFSET,
        );
        // Маскируем всё, кроме IRQ0 (бит 0) на master; slave не нужен
        crate::utils::outb(PIC1_DATA, !(1u8 << 0));
        crate::utils::outb(PIC2_DATA, 0xFF);

        rm_bios_video_call();

        crate::x86::pic::reinit_offsets(
            crate::x86::pic::PIC1_OFFSET,
            crate::x86::pic::PIC2_OFFSET,
        );
        crate::utils::outb(PIC1_DATA, saved_mask1);
        crate::utils::outb(PIC2_DATA, saved_mask2);

        crate::x86::idt::interrupts_enable();
    }
}
