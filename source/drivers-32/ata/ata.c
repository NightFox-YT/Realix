#include "ata.h"
#include "../../libc/klibc/stddef.h"
#include "../../include/io.h"
#include "../../include/slab.h"
#include "../../include/errors.h"
#include "../../include/driver.h"
#include "../../drivers-32/serial/com.h"	

struct ata_channel *ata_primary_ptr = 0;
struct ata_channel *ata_secondary_ptr = 0;

static void ata_delay(struct ata_channel *ch)
{
	inb(ch->cmd_base + ATA_REG_STATUS);
	inb(ch->cmd_base + ATA_REG_STATUS);
	inb(ch->cmd_base + ATA_REG_STATUS);
	inb(ch->cmd_base + ATA_REG_STATUS);
}

static uint8_t ata_wait(struct ata_channel *ch, uint8_t mask, uint8_t value, uint32_t timeout)
{
	for(uint32_t i = 0; i < timeout; i++)
	{
		uint8_t status = inb(ch->cmd_base + ATA_REG_STATUS);
		if (!(status & ATA_STATUS_BSY) && ((status & mask) == value)) return status;
		ata_delay(ch);
	}
	return 0;
}

static void ata_parse_model(uint16_t *id_data, char *model_out)
{
	int idx = 0;
	for (int i = 27; i <= 46; i++) 
	{
		uint16_t word = id_data[i];
		model_out[idx++] = (char)(word >> 8); // сначала старший байт
		model_out[idx++] = (char)(word & 0xFF); // потом младший
	}
	model_out[idx] = '\0';

	for (int i = idx - 1; i >= 0 && model_out[i] == ' '; i--) model_out[i] = '\0';
}

static void ata_identify_device(struct ata_channel *ch, uint8_t dev_idx)
{
	uint8_t drive_select = (dev_idx == 0) ? ATA_MASTER : ATA_SLAVE;
	struct ata_device *dev = &ch->dev[dev_idx];

	dev->present = 0;

	outb(ch->cmd_base + ATA_REG_DRIVE_SELECT, drive_select);
	ata_delay(ch);

	outb(ch->cmd_base + ATA_REG_SEC_COUNT, 0);
	outb(ch->cmd_base + ATA_REG_LBA_LOW, 0);	
	outb(ch->cmd_base + ATA_REG_LBA_MID, 0);
	outb(ch->cmd_base + ATA_REG_LBA_HIGH, 0);

	outb(ch->cmd_base + ATA_REG_COMMAND, ATA_CMD_ID);
	ata_delay(ch);

	uint8_t status = inb(ch->cmd_base + ATA_REG_STATUS);
	if (status == 0) return;//если устройства нет

	uint8_t is_patapi = 0;
	status = ata_wait(ch, ATA_STATUS_DRQ, ATA_STATUS_DRQ, 100000);

	if (!status || (status & ATA_STATUS_ERR))
	{
		/* ghjdthrf cbuyfnehs PATAPI*/
		uint8_t cl = inb(ch->cmd_base + ATA_REG_LBA_MID);
		uint8_t chn = inb(ch->cmd_base + ATA_REG_LBA_HIGH);

		if (cl == 0x14 && chn == 0xEB)
		{
			is_patapi = 1;
			outb(ch->cmd_base + ATA_REG_COMMAND, ATA_CMD_ID_PACKET);
			ata_delay(ch);

			status = ata_wait(ch, ATA_STATUS_DRQ, ATA_STATUS_DRQ, 100000);
			if (!status || (status & ATA_STATUS_ERR)) return;
		} else return;		
	}

	uint16_t id_data[256];
	for (int i = 0; i < 256; i++) id_data[i] = inw(ch->cmd_base + ATA_REG_DATA);

	dev->present = 1;
	dev->type = is_patapi ? 1 : 0;

	ata_parse_model(id_data, dev->model);

	if (!is_patapi)
	{
		/*в patapi насколько я помню размер определяется динамически, значит тут определение в секторах, ТОЛЬКО ДЛЯ PATA HDD!!, хотя зачем я это пишу, всё равно ATAPI драйвер не готов*/
		/*слова 60-61 содержат общее количество секторов в LBA28*/
		uint32_t sectors_lbaTE = id_data[60] | ((uint32_t)id_data[61] << 16);
		dev->sectors = sectors_lbaTE;
	} else dev->sectors = 0;
}

static struct ata_dependencies env;

int ata_init(struct kernel_io_interfaces *io, struct ata_dependencies *dep/*johnny*/) 
{
	if (!io || !dep) return -1;
	env = *dep;

	if (!env.kmalloc || !env.memset || !env.pci_read_config || !env.pci_write_config) 
		return -2;

	ata_primary_ptr = (struct ata_channel*)env.kmalloc(sizeof(struct ata_channel));
	if (ata_primary_ptr) env.memset(ata_primary_ptr, 0, sizeof(struct ata_channel));
	ata_secondary_ptr = (struct ata_channel*)env.kmalloc(sizeof(struct ata_channel));
	if (ata_secondary_ptr) env.memset(ata_secondary_ptr, 0, sizeof(struct ata_channel));
	if(!ata_primary_ptr || !ata_secondary_ptr) return ENOMEM;

    ata_primary_ptr->cmd_base  = 0x1F0;
    ata_primary_ptr->ctrl_base = 0x3F6;
    ata_primary_ptr->irq       = 14;

    ata_secondary_ptr->cmd_base  = 0x170;
    ata_secondary_ptr->ctrl_base = 0x376;
    ata_secondary_ptr->irq       = 15;

    for (uint16_t bus = 0; bus < 256; bus++) 
    {
        for (uint8_t slot = 0; slot < 32; slot++) 
        {
            for (uint8_t func = 0; func < 8; func++) 
            {    
                uint32_t id_reg = env.pci_read_config(bus, slot, func, 0x00);
                if ((id_reg & 0xFFFF) == 0xFFFF) continue; // Пустой слот

                uint32_t class_reg = env.pci_read_config(bus, slot, func, 0x08);
                uint8_t base_class = (class_reg >> 24) & 0xFF;
                uint8_t subclass   = (class_reg >> 16) & 0xFF;

                if (base_class == PCI_CLASS_MASS_STORAGE && subclass == PCI_SUBCLASS_IDE) 
                {
                    uint32_t pci_cmd = env.pci_read_config(bus, slot, func, 0x04);
                    pci_cmd |= 0x05;
					env.pci_write_config(bus, slot, func, 0x04, pci_cmd);
					
					uint32_t bar0 = env.pci_read_config(bus, slot, func, 0x10);
					uint32_t bar1 = env.pci_read_config(bus, slot, func, 0x14);
					uint32_t bar2 = env.pci_read_config(bus, slot, func, 0x18);
					uint32_t bar3 = env.pci_read_config(bus, slot, func, 0x1C);

					ata_primary_ptr->cmd_base = (bar0 & 0x1) ? (bar0 & ~0x3) : 0x1F0;
					ata_primary_ptr->ctrl_base = (bar1 & 0x1) ? (bar1 & ~0x3) : 0x3F6;
					ata_secondary_ptr->cmd_base = (bar2 & 0x1) ? (bar2 & ~0x3) : 0x170;
					ata_secondary_ptr->ctrl_base = (bar3 & 0x1) ? (bar3 & ~0x3) : 0x376;

					ata_identify_device(ata_primary_ptr, 0); // master
                    ata_identify_device(ata_primary_ptr, 1); // slave

                    ata_identify_device(ata_secondary_ptr, 0);
                    ata_identify_device(ata_secondary_ptr, 1);
                    return 0;
                }
            }
        }
    }
	return 1;
}

REALIX_COMPONENT("ata_driver", ata_init);

int ata_read_sector(struct ata_channel *ch, uint8_t dev_idx, uint32_t lba, uint16_t *buf)
{
	if (dev_idx > 1 || !ch->dev[dev_idx].present) return -3;
	if (ch->dev[dev_idx].type == 1) return -4; //невозможно прочитать CD-ROM через PIO
	if (!ata_wait(ch, ATA_STATUS_DRDY, ATA_STATUS_DRDY, 100000)) return -1;

	uint8_t drive_bits = (dev_idx == 0) ? ATA_MASTER : ATA_SLAVE;
	outb(ch->cmd_base + ATA_REG_DRIVE_SELECT, 0x40 | drive_bits | ((lba >> 24) & 0x0F));
	ata_delay(ch);

	outb(ch->cmd_base + ATA_REG_SEC_COUNT, 1);
	outb(ch->cmd_base + ATA_REG_LBA_LOW, (uint8_t)lba);
	outb(ch->cmd_base + ATA_REG_LBA_MID, (uint8_t)(lba >> 8));
	outb(ch->cmd_base + ATA_REG_LBA_HIGH, (uint8_t)(lba >> 16));

	outb(ch->cmd_base + ATA_REG_COMMAND, ATA_CMD_READ_PIO);
	ata_delay(ch);

	uint8_t status = ata_wait(ch, ATA_STATUS_DRQ, ATA_STATUS_DRQ, 100000);
	if (!status || (status & ATA_STATUS_ERR)) return -2;

	for (int i = 0; i < 256; i++) buf[i] = inw(ch->cmd_base + ATA_REG_DATA);

	return 0;
}

int ata_write_sector(struct ata_channel *ch, uint8_t dev_idx, uint32_t lba, const uint16_t *buf)
{
	if (dev_idx > 1 || !ch->dev[dev_idx].present) return -3;
	if (ch->dev[dev_idx].type == 1) return -4; //невозможно прочитать CD-ROM через PIO
	if (!ata_wait(ch, ATA_STATUS_DRDY, ATA_STATUS_DRDY, 100000)) return -1;

	uint8_t drive_bits = (dev_idx == 0) ? ATA_MASTER : ATA_SLAVE;
	outb(ch->cmd_base + ATA_REG_DRIVE_SELECT, 0x40 | drive_bits | ((lba >> 24) & 0x0F));
	ata_delay(ch);

	outb(ch->cmd_base + ATA_REG_SEC_COUNT, 1);
	outb(ch->cmd_base + ATA_REG_LBA_LOW,   (uint8_t)lba);
	outb(ch->cmd_base + ATA_REG_LBA_MID,   (uint8_t)(lba >> 8));
	outb(ch->cmd_base + ATA_REG_LBA_HIGH,  (uint8_t)(lba >> 16));

	outb(ch->cmd_base + ATA_REG_COMMAND, ATA_CMD_WRITE_PIO);
	ata_delay(ch);

	uint8_t status = ata_wait(ch, ATA_STATUS_DRQ, ATA_STATUS_DRQ, 100000);
	if (!status || (status & ATA_STATUS_ERR)) return -2;

	for (int i = 0; i < 256; i++) outw(ch->cmd_base + ATA_REG_DATA, buf[i]);

	ata_wait(ch, 0, 0, 100000);

	return 0;
}