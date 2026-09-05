#!/usr/bin/env bash

set -euo pipefail

ALWAYS_AGREE=false
HIDE_PROCESS=true

show_help() {
    echo "Usage: $0 [-aA/-V/-v]"
    echo "-aA/--always-agree Always Agree"
    echo "-V/--verbose Show install logs"
    echo "-v/--version Show version"
    echo "-h/--help    Show help"
    exit 0
}

explain_ewszz() {
    echo "┌───────────────────────── EWS00 - Explain Why Sudo 00 ────────────────────────────────┐"
    echo "│ Sudo is required for installing toolchains aka. Rust nightly and GCC i686-elf.       │"
    echo "│ Sudo also required for installing dependencies for build via system package manager. │"
    echo "│ Sudo rights are being updated every 2 minutes while script is executing on machine.  │"
    echo "└──────────────────────────────────────────────────────────────────────────────────────┘"
}

view_installed_packages_zz() {
    echo "┌───────────────────────── VIP00 - View Installed Packages ────────────────────────────────┐"
    echo "│ mtools, nmap, qemu-desktop, i686-elf-gcc, i686-elf-binutils, curl, wget, Makefile, rust, │"
    echo "│ llvm-tools and rust toolchain nightly. The GCC Toolchain is installed in /usr/local/bin/ │"
    echo "└──────────────────────────────────────────────────────────────────────────────────────────┘"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -aA|--always-agree)
            ALWAYS_AGREE=true
            shift
            ;;
        -V|--verbose)
            HIDE_PROCESS=false
            shift
            ;;
        -h|--help)
            show_help
            shift
            ;;
        EWS00)
            explain_ewszz
            shift
            ;;
        VIP00)
            view_installed_packages_zz
            shift
            ;;
        -v|--version)
            echo "Realix Setup Shell Script v0.1"
            exit 0
            ;;
        *)
            echo "Unknown parameter: $1"
            show_help
            ;;
    esac
done

run_sudo() {
    if [ "$EUID" -ne 0 ]; then
        sudo "$@"
    else
        "$@"
    fi
}

show_loader() {
    local pid=$1
    local msg=$2
    local delay=0.5
    local dots=""

    tput civvis 2>/dev/null || true

    while kill -0 "$pid" 2>/dev/null; do
        dots="$dots ."
        if [ "${#dots}" -gt 8 ]; then
            dots=""
        fi
        printf "\r\033[K%s%-8s" "$msg" "$dots"
        sleep "$delay"
    done

    tput cnorm 2>/dev/null || true

    wait "$pid"
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        printf "\r\033[K%s [ \033[32m✓\033[0m ]\n" "$msg"
    else
        printf "\r\033[K%s [ \033[31mFAIL\033[0m ]\n" "$msg"
        printf "\033[31mError\033[0m: process exited with code $exit_code. Rerun with --verbose to get logs.\n"
        exit $exit_code
    fi
}

SYS_PM=""
ASSUME_YES=""

run_task() {
    local msg=$1
    shift

    if [ "$HIDE_PROCESS" = true ]; then
        "$@" >/dev/null 2>&1 &
        local pid=$!
        show_loader "$pid" "$msg"
    else
        printf "\033[31m>\033[32m>\033[35m>\033[0m $msg\n"
        "$@"
    fi
}

detect_env() {
    if command -v brew &> /dev/null; then
        PM="brew"
        [ "$ALWAYS_AGREE" = true ] && ASSUME_YES="-y"
        return 0
    fi

    if [[ "$OSTYPE" == "darwin"* ]]; then
        printf "\033[31mError\033[0m: We detected macOS, but not homebrew!\n"
        echo "We cannot continue without homebrew"
        echo "If it not installed, please install it by executing this command: "
        echo "'/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"'"
        exit 1
    fi

    if [ -f /etc/os-release ]; then
        . /etc/os-release
        case "$ID" in
            debian|ubuntu|mint)
                SYS_PM="apt"
                [ "$ALWAYS_AGREE" = true ] && ASSUME_YES="-y"
                ;;
            arch|manjaro|cachyos)
                SYS_PM="pacman"
                [ "$ALWAYS_AGREE" = true ] && ASSUME_YES="--noconfirm"
                ;;
            void)
                SYS_PM="xbps"
                [ "$ALWAYS_AGREE" = true ] && ASSUME_YES="-y"
                ;;
            opensuse*|suse)
                SYS_PM="zypper"
                [ "$ALWAYS_AGREE" = true ] && ASSUME_YES="-n"
                ;;
            rhel|centos|fedora)
                SYS_PM="dnf"
                [ "$ALWAYS_AGREE" = true ] && ASSUME_YES="-y"
                ;;
            *)
                printf "\033[31mError\033[0m: That Linux Distro $ID is not supported\n"
                exit 1
                ;;
        esac
    else
        printf "\033[31mError\033[0m: Failed to identify OS\n"
        exit 1
    fi
}

install_system_packages() {
    local msg="Installing: Dependencies, via '$SYS_PM'"

    case "$SYS_PM" in
        brew)
            run_task "$msg" brew install nasm mtools qemu curl wget
            ;;
        apt)
            run_task "$msg" run_sudo apt-get update
            run_task "$msg" run_sudo apt-get install $ASSUME_YES nasm mtools qemu-desktop curl base-devel wget
            ;;
        pacman)
            run_task "$msg" run_sudo pacman -S --needed $ASSUME_YES nasm mtools qemu-desktop curl base-devel wget
            ;;
        xbps)
            run_task "$msg" run_sudo xbps-install -S $ASSUME_YES nasm mtools qemu curl base-devel wget
            ;;
        zypper)
            run_task "$msg" run_sudo zypper install $ASSUME_YES nasm mtools qemu-x86 curl wget -t pattern devel_basis
            ;;
        dnf)
            run_task "$msg" run_sudo dnf install $ASSUME_YES nasm mtools qemu-system-x86 curl wget @"Development Tools"
            ;;
    esac
}

install_rust() {
    local msg="Installing: Rust"
    local rustup_args=""
    [ "$ALWAYS_AGREE" = true ] && rustup_args="-y"

    if ! command -v rustup &> /dev/null; then
        run_task "$msg" bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- $rustup_args"
        source "$HOME/.cargo/env"
    else 
        if [ "$HIDE_PROCESS" = false ]; then echo "Rust is already installed." 
        fi
    fi
}

install_rust_toolchain() {
    local msg="Installing: Rust nightly toolchain"
    run_task "$msg" bash -c "
        rustup toolchain install nightly && \
        rustup component add rust-src llvm-tools --toolchain nightly && \
        cargo install cargo-binutils
    "
}

install_elf_toolchain() {
    local msg="Installing: i686-elf toolchain(GCC)"

    if [ "$SYS_PM" = "brew" ]; then
        # на macOS или linux при наличии brew лучше поставить тулчейн через него
        run_task_wrap "$msg" brew install i686-elf-gcc i686-elf-binutils
    else 
        local msg2="Downloading i686-elf-tools for linux(this will take few minutes)"
        local msg3="Unpacking build tools"
        local tmp_dir
        tmp_dir=$(mktemp -d)

        run_task "$msg2" curl -L "https://github.com/RealixTeam/buildutils/releases/download/require_buildutils2/i686-elf-tools-linux-x86_64.tar.gz" -o "$tmp_dir/tools.tar.gz"
        run_task "$msg3" run_sudo tar -xzf "$tmp_dir/tools.tar.gz" -C /usr/local --strip-components=1

        rm -rf "$tmp_dir"
    fi
}

set_sudo() {
    if [ "$EUID" -ne 0 ] && [ "$SYS_PM" != "brew" ]; then
        echo "Sudo required. P.S. Enter EWS00 in args to get explain."
        sudo -v 
        clear

        while true; do sudo -n true; sleep 60; kill -0 "$$" || exit; done 2>/dev/null &   
    fi
}

main() {
    detect_env
    set_sudo
    install_system_packages
    install_rust
    install_rust_toolchain
    install_elf_toolchain
    printf "\033[32m✓\033[0m Done. Type VIP00 as argument to view all installed packages.\n"
}   

main