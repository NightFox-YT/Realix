:: Realix Build Script (v0.01)
@echo off
setlocal

:: Конфигурация
set "SRC_DIR=source"
set "BUILD_DIR=build"

:: Проверка исходного файла
if not exist "%SRC_DIR%\boot.asm" (
   echo [ERR] Source file not found: '%SRC_DIR%\boot.asm'
   pause
   exit /b 1
)

:: Создание папки сборки
mkdir "%BUILD_DIR%" 2>nul

:: Компиляция загрузчика
echo [LOG] Compiling bootloader...
nasm -f bin "%SRC_DIR%\boot.asm" -o "%BUILD_DIR%\boot.bin"

:: Успешная компиляция
if exist "%BUILD_DIR%\boot.bin" (
    echo [LOG] Bootloader compiled successfully: '%BUILD_DIR%\boot.bin'
    echo [LOG] Ready to create disk image!
) else (
    echo [ERR] Compilation failed!
    echo       Possible reasons:
    echo       1. Syntax errors in assembly code
    echo       2. Invalid file paths
    echo       3. NASM not installed or not in PATH (Download NASM: https://nasm.us)
    pause
    exit /b 1
)

pause
exit /b 0