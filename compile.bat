:: Realix Build Script
:: (C) v0.01 | 19.07.2025
:: ================
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
if not exist "%BUILD_DIR%" (
    mkdir "%BUILD_DIR%" 2>nul
    echo [LOG] Created %BUILD_DIR% directory.
)

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
    echo       3. NASM not installed or not in PATH (Download: https://nasm.us)
    pause
    exit /b 1
)

pause
exit /b 0