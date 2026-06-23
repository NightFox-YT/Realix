# rcom 1.0
__rcom__ or __realix component__ is a format created for fast and comfort parsing by kernel and add small signature check

* Align: __4 bytes__

* Endian: __Little Endian__

## Structure
File .rcom have 2 logical partitions
- RHINO Header
- Body
  - Contains binary

## Структура заголовка
```c
struct __attribute__((packed)) rcom_header {
    char     magic[4];       // 0x00: rcom signature
    uint32_t magic_check;     // 0x04: Driver signature
    uint32_t exec_offset;    // 0x08: Offset from start of file
    uint32_t component_size; // 0x0C: Size of component's binary
    char     name[16];       // 0x10: Name of component in ASCII 
};
```

## Body
Just after 32 bytes, pure x86 binary code begins

The very first byte in the body (offset 0x20 from the start) must be the driver initialization function.
The linker configures it so that when jumping to this offset, control is transferred to the initialization code.

Within the body, the driver can use any local functions compiled with it.

## Offset
The final size of the .rcom file on disk must be a strict multiple of 4 bytes. If the header and body size are not a multiple of 4, the script automatically appends 1-3 zero bytes.

## Note
A driver packaged in rcom must have initialization in the form of __int driver_init()__, returning 0 on success, and anything but zero on error.
More details in the instructions for writing drivers for the Realix kernel: [click](driver.md)

## rcom scheme
![rcom_scheme](assets/rcomscheme.png)