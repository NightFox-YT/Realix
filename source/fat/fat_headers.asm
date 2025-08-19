; Realix > FAT headers
; (C) v0.03 | 19.08.25
; ====================

bdb_oem:                   db 'MSWIN4.1' ; Идентификатор OEM (8 байт)
bdb_bytes_per_sector:      dw 512        ; Кол-во байт на сектор (Floppy: 512)
bdb_sectors_per_cluster:   db 1          ; Кол-во секторов на кластер
bdb_reserved_sectors:      dw 1          ; Кол-во зарезервированных секторов (Сектор загрузчика)
bdb_fats:                  db 2          ; Кол-во FAT таблиц
bdb_dir_entries:           dw 0x0E0      ; Кол-во записей корневого каталога
bdb_total_sectors:         dw 2880       ; Кол-во секторов (2880 * 512 = 1.44 мб)
bdb_media_descriptor_type: db 0x0F0      ; Тип диска (F0 - 3.5" floppy disk)
bdb_sectors_per_fat:       dw 9          ; Кол-во секторов на FAT таблицу
bdb_sectors_per_track:     dw 18         ; Кол-во секторов на дорожку
bdb_heads:                 dw 2          ; Кол-во голов
bdb_hidden_sectors:        dd 0          ; Кол-во скрытых секторов
bdb_large_sectors:         dd 0          ; Кол-во секторов, следующих дальше 65535 сектора

; Дополнительные параметры (extended boot record)
ebr_drive_number:          db 0                  ; Номер диска (0x00 floppy / 0x80 hdd)
                           db 0                  ; Зарезервировано
ebr_signature:             db 0x29               ; Подпись (0x28 или 0x29)
ebr_volume_id:             db 23h, 07h, 20h, 25h ; Серийный номер (Произвольный)
ebr_volume_label:          db '  Realix   '      ; Название тома (11 байт)
ebr_system_id:             db 'FAT12   '         ; Тип файловой системы (FAT12, FAT16... / 8 байт)