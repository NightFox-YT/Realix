#!/usr/bin/env python3

# Rcom files builder to ramfs 
# Copyright(C) 2026 Alexander Silaev <thebinaryblob@gmail.com>.
import sys
import struct
import os

def build_ramfs(output, input):
    ramfs_data = b''
    
    for filepath in input:
        if not os.path.exists(filepath):
            print(f"[-] Error: File {filepath} not found!")
            sys.exit(1)
            
        filename = os.path.basename(filepath)
        size = os.path.getsize(filepath)
        
        name_bytes = filename.encode('utf-8')[:31].ljust(32, b'\x00')
        header = struct.pack('<32sI', name_bytes, size)
        
        with open(filepath, 'rb') as f:
            file_data = f.read()
            
        ramfs_data += header + file_data
        
    ramfs_data += b'\x00' * 36
    
    with open(output, 'wb') as f:
        f.write(ramfs_data)
        
    
if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python3 make_ramfs.py <output_ramfs.img> <file1.rcom> <file2.rcom> ...")
        print("The realix utils.")
        sys.exit(1)
        
    build_ramfs(sys.argv[1], sys.argv[2:])