cmake_minimum_required(VERSION 3.20)
project(Realix NONE)

set(SRC_DIR "${CMAKE_CURRENT_SOURCE_DIR}/source")
set(BUILD_OUT "${CMAKE_CURRENT_BINARY_DIR}")

set(BOOTIX_BIN "${BUILD_OUT}/bootix.bin")
set(INITRIX_BIN "${BUILD_OUT}/initrix.bin")
set(KERNEL_BIN "${BUILD_OUT}/kernel.bin")
set(REALIX_IMG "${BUILD_OUT}/realix.img")

find_program(NASM nasm REQUIRED)
find_program(DD dd REQUIRED)
find_program(MFORMAT mformat REQUIRED)
find_program(MCOPY mcopy REQUIRED)

find_program(QEMU
    NAMES qemu-system-i386 qemu-system-x86_64
)

file(GLOB_RECURSE ASM_SOURCES CONFIGURE_DEPENDS "${SRC_DIR}/*.asm")

function(assemble_bin target source output)
    add_custom_command(
        OUTPUT "${output}"
        COMMAND "${NASM}" -f bin "-i${SRC_DIR}/" "${SRC_DIR}/${source}" -o "${output}"
        DEPENDS ${ASM_SOURCES}
        VERBATIM
        COMMENT "Building ${target}"
    )
endfunction()

assemble_bin(bootix  "bootloader/bootix.asm"  "${BOOTIX_BIN}")
assemble_bin(initrix "bootloader/initrix.asm" "${INITRIX_BIN}")
assemble_bin(kernel  "kernel16/kernel.asm"    "${KERNEL_BIN}")

add_custom_command(
    OUTPUT "${REALIX_IMG}"
    COMMAND "${DD}" if=/dev/zero "of=${REALIX_IMG}" bs=512 count=2880 status=none
    COMMAND "${MFORMAT}" -i "${REALIX_IMG}" -f 1440 ::
    COMMAND "${DD}" "if=${BOOTIX_BIN}" "of=${REALIX_IMG}" conv=notrunc status=none
    COMMAND "${MCOPY}" -i "${REALIX_IMG}" "${INITRIX_BIN}" "::INITRIX.BIN"
    COMMAND "${MCOPY}" -i "${REALIX_IMG}" "${KERNEL_BIN}" "::KERNEL.BIN"
    DEPENDS "${BOOTIX_BIN}" "${INITRIX_BIN}" "${KERNEL_BIN}"
    VERBATIM
    COMMENT "Creating realix.img"
)

add_custom_target(realix ALL DEPENDS "${REALIX_IMG}")
add_custom_target(image DEPENDS "${REALIX_IMG}")

if(QEMU)
    add_custom_target(run
        COMMAND "${QEMU}" -drive "file=${REALIX_IMG},format=raw,if=floppy"
        DEPENDS realix
        VERBATIM
        USES_TERMINAL
    )

    add_custom_target(run-debug
        COMMAND "${QEMU}"
                -no-reboot
                -no-shutdown
                -d int,cpu_reset,guest_errors
                -D "${BUILD_OUT}/qemu-debug.log"
                -drive "file=${REALIX_IMG},format=raw,if=floppy"
        DEPENDS realix
        VERBATIM
        USES_TERMINAL
    )
endif()

message(STATUS "NASM: ${NASM}")
message(STATUS "dd: ${DD}")
message(STATUS "mformat: ${MFORMAT}")
message(STATUS "mcopy: ${MCOPY}")
message(STATUS "QEMU: ${QEMU}")
message(STATUS "Image: ${REALIX_IMG}")
