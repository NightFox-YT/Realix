#include "pci.h"
#include "../../include/io.h"
#include "../serial/com.h"
#include "../../libc/klibc/stdio.h"
#include "../../include/driver.h"

uint32_t pci_read_config(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset)
{
	uint32_t b = ((uint32_t)bus)    & 0xFF;  // 8 бит
    uint32_t s = ((uint32_t)slot)   & 0x1F;  // 5 бит (0-31)
    uint32_t f = ((uint32_t)func)   & 0x07;  // 3 бита (0-7)
    uint32_t o = ((uint32_t)offset) & 0xFC;  // 6 бит, выровненных по 4 байтам

    uint32_t address = (1            << 31) | 
                       (b            << 16) | 
                       (s            << 11) | 
                       (f            << 8)  | 
                       o;

    outl(PCI_CONFIG_ADDRESS, address);

	io_wait();
    
    return inl(PCI_CONFIG_DATA);
}

void pci_check_device(uint8_t bus, uint8_t device)
{
    uint32_t reg0 = pci_read_config(bus, device, 0, 0);
    uint16_t vendor_id = reg0 & 0xFFFF;
    if (vendor_id == 0xFFFF || vendor_id == 0x0000) return;

    uint32_t reg3 = pci_read_config(bus, device, 0, 0x0C);
    uint8_t header_type = (reg3 >> 16) & 0xFF;

    uint8_t total_functions = (header_type & 0x80) ? 8 : 1;

    for (uint8_t function = 0; function < total_functions; function++)
    {
        uint32_t current_reg0 = pci_read_config(bus, device, function, 0);
        uint16_t current_vendor = current_reg0 & 0xFFFF;
        uint16_t current_device = (current_reg0 >> 16) & 0xFFFF;

        if (current_vendor == 0xFFFF || current_vendor == 0x0000) continue;

        uint32_t reg2 = pci_read_config(bus, device, function, 0x08);
        uint8_t base_class = (reg2 >> 24) & 0xFF;
        uint8_t sub_class = (reg2 >> 16) & 0xFF;

        serial_print("FOUND DEVICE\n");

        if (base_class == 0x01 && sub_class == 0x01) {
            serial_print("FOUND ATA DISK\n");
        }

        char buf[128];
        snprintf(buf, sizeof(buf), "PCI: Bus %d, Dev %d, Func %d -> Vendor: 0x%04X, Device: 0x%04X, Class: 0x%02X, Sub: 0x%02X\n", 
                 (uint32_t)bus, (uint32_t)device, (uint32_t)function, 
                 (uint32_t)current_vendor, (uint32_t)current_device, 
                 (uint32_t)base_class, (uint32_t)sub_class);
        serial_print(buf);

        if (base_class == 0x03) {
            serial_print("	[^] Found Videocard.\n");
        }
    }
}

void pci_write_config(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset, uint32_t value)
{
    uint32_t b = ((uint32_t)bus)    & 0xFF;
    uint32_t s = ((uint32_t)slot)   & 0x1F;
    uint32_t f = ((uint32_t)func)   & 0x07;
    uint32_t o = ((uint32_t)offset) & 0xFC;

    uint32_t address = (1            << 31) | 
                       (b            << 16) | 
                       (s            << 11) | 
                       (f            << 8)  | 
                       o;

    outl(PCI_CONFIG_ADDRESS, address);
    io_wait();
    
    outl(PCI_CONFIG_DATA, value);
    io_wait();
}

int pci_init(void)
{
    serial_print("PCI: Initializing and scanning bus...\n");
    
    for (uint16_t device = 0; device < 32; device++) 
    {
        pci_check_device(0, device);
    }

    return 0;
}

REALIX_COMPONENT("pci_driver", pci_init);