; Realix > Print
; (C) v0.02 | 17.08.25
; ==============

; [Print] Вывод строки на экран
; Параметры:
;  - ds:si: адрес строки
print:
	push si
	push ax
	push bx

    mov ah, 0x0E ; TTY mode (Вывод с прокруткой курсора)
	xor bh, bh   ; Номер страницы (0)

.next_char:
	lodsb        ; Загрузка символа из si в al
	test al, al  ; Проверка на 0 (Конец строки)
	jz .done
	
	int 0x10
	jmp .next_char

.done:
	pop bx
	pop ax
	pop si
	ret