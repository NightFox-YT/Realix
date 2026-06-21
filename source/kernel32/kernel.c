#include "../include/io.h"
#include "../drivers-32/serial/com.h"
#include "../drivers-32/pci/pci.h"
#include "../include/sysenter.h"
#include "../include/gdt.h"
#include "../include/idt.h"
#include "../libc/stdio.h"
#include "../include/slab.h"
#include "../drivers-32/rxbdph/graphics.h"
#include "../drivers-32/ata/ata.h"
#include "../drivers-32/fat32/fat32.h"
#include "../vfs/vfs.h"


extern void _jump_to_userspace(uint32_t user_eip, uint32_t user_esp);

void temporary_user_stub(void)
{
	//serial_print("printing string\n");
	printf("Hello from ring 3!");
    FILE *file = fopen("kakashka.bin", "w");
    if (file) { printf("hey, FAT32 is works!"); fclose(file); }
    else { printf("file doesnt work :("); }
    while(1);
}

struct vfs_node *root_fs_node = NULL;

int mount_fat32_root(struct ata_channel *ch, uint8_t drive)
{
    struct fat32_volume *vol = kmalloc(sizeof(struct fat32_volume));
    if (!vol) return -1;

    int status = fat32_init_volume(vol, ch, drive);
    if (status != 0)
    {
        kfree(vol);
        return status;
    }

    struct vfs_node *node = kmalloc(sizeof(struct vfs_node));
    if (!node)
    {
        kfree(vol);
        return -1;
    }

    char *name_src = "dsk0";
    for(int i = 0; i < 5; i++) node->name[i] = name_src[i];

    node->type = 1;
    node->oprs = &fat32_node_ops;
    node->priv_data = vol;
    node->next = NULL;

    root_fs_node = node;

    return 0;
}

void kernel_exec(void)
{
    gdt_init();
    idt_init();
    pic_init();
    kmalloc_init();
    pci_init();
    ata_init();
    
    serial_print("Info: Attempting to mount root...\n");

    uint8_t raw_status = inb(0x1F7);
    char dbg_msg[64];
    snprintf(dbg_msg, sizeof(dbg_msg), "[DBG] Raw ATA Status Port (0x1F7) = 0x%02X\n", (uint32_t)raw_status);
    serial_print(dbg_msg);

    snprintf(dbg_msg, sizeof(dbg_msg), "[DBG] ata_primary_ptr Addr = 0x%08X\n", (uint32_t)ata_primary_ptr);
    serial_print(dbg_msg);
    if (ata_primary_ptr) {
        snprintf(dbg_msg, sizeof(dbg_msg), "[DBG] ata_primary_ptr->cmd_base = 0x%04X\n", (uint32_t)ata_primary_ptr->cmd_base);
        serial_print(dbg_msg);
    }

    int mount_res = mount_fat32_root(ata_primary_ptr, 0);
    
    if (rxbdph_init() == 0) 
    {
        serial_print("Initialized RXBDPH Driver\n");
        rxbdph_clear(VGA_LGRAY);

        rxbdph_fill_rect(50, 50, 300, 200, VGA_BLUE);
        rxbdph_draw_rect(50, 50, 300, 200, VGA_BLACK); 
        rxbdph_fill_circle(200, 150, 40, VGA_RED);

        rxbdph_draw_line(0, 0, 1024, 768, VGA_YELLOW);
    } 

    uint32_t *sysenter_stack = kmalloc(4096);
    uint32_t sysenter_stack_top = (uint32_t)sysenter_stack + 4096;
    sysenter_init(sysenter_stack_top);
    
    set_tss_esp0(sysenter_stack_top);
    
    if (init_serial() == 0)
    {
        serial_print("Realix Info: OS started up, the Realix Serial\n");
    }

    serial_print("Kernel: Allocating user stack and dropping to Ring 3...\n");

    uint32_t *user_stack = kmalloc(4096);
    uint32_t user_stack_top = (uint32_t)user_stack + 4096;

    _jump_to_userspace((uint32_t)temporary_user_stub, user_stack_top);

    while(1) {
        __asm__ volatile("hlt");
    }   
}