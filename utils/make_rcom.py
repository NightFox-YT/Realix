#!/usr/bin/env python3
import os
import struct
import sys

__driver_magic_LE 0x53484954
__rcom_magic_LE 0x52434F4D
__rcom_header_size = 32

def pack_file(input, output, name):
    if not os.path.exists(input):
        print(f"[-] Error: File {input} not found!")
        sys.exit(1)

    with open(input, 'rb') as f:
        raw_code = f.read()

    size = len(raw_code)
    exec_offfset = __rcom_header_size

    name_bytes = component_name.encode('utf-8')[:15]
    name_bytes = name_bytes.ljust(16, b'\x00')

    # Формирование заголовка RHINO, описано в файле about/rcom.md
    header = struct.pack('<4sIII16s',
        __rcom_magic_LE,
        __driver_magic_LE,
        exec_offfset,
        size,
        name_bytes)

    rcom_data = header + raw_code

    remainder = len(rcom_data) % 4
    if remainder != 0:
        rcom_data += b'\x00' * (4 - remainder)

    with open(output, 'wb') as f:
        f.write(rcom_data)

    print("Done.")

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python3 make_rcom.py <input.bin> <output.rcom> <name>")
        print("The realix utils.")
        sys.exit(0)
    
    pack_file(sys.argv[1], sys.argv[2], sys.argv[3])