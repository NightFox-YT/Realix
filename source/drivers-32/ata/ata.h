#ifndef __realix_ata__
#define __realix_ata__

#include "../../libc/stdint.h"

#define PCI_CLASS_MASS_STORAGE 0x01
#define PCI_SUBCLASS_IDE 0x01

#define ATA_REG_DATA 			0
#define ATA_REG_ERROR 			1
#define ATA_REG_FEATURES 		1
#define ATA_REG_SEC_COUNT 		2
#define ATA_REG_LBA_LOW 		3
#define ATA_REG_LBA_MID			4
#define ATA_REG_LBA_HIGH 		5
#define ATA_REG_DRIVE_SELECT 	6
#define ATA_REG_COMMAND 		7
#define ATA_REG_STATUS			7

#define ATA_CMD_READ_PIO 	0x20
#define ATA_CMD_WRITE_PIO 	0x30
#define ATA_CMD_ID			0xEC
#define ATA_CMD_ID_PACKET 	0xA1

#define ATA_STATUS_ERR 		0x01
#define ATA_STATUS_DRQ		0x08
#define ATA_STATUS_DF		0x20
#define ATA_STATUS_DRDY		0x40
#define ATA_STATUS_BSY 		0x80

#define ATA_MASTER 0xA0 
#define ATA_SLAVE  0xB0

struct ata_device {
    uint8_t present;     /*Флаг: 1 - диск найден, 0 - нет*/
    uint8_t type;        /*Тип: 0 - PATA (HDD/Flash), 1 - PATAPI (CD-ROM)*/
    uint32_t sectors;    /*Общее количество секторов (размер диска)*/
    char model[41];      /*Строка модели диска из ID*/
};

struct ata_channel {
	uint16_t cmd_base;	/*База регистров команд*/
	uint16_t ctrl_base; /*База регистров управления*/
	uint8_t irq;		/*Номер прерывания*/
	struct ata_device dev[2]; /*[0] - Master, [1] - Slave*/
};

void ata_init(void);
int ata_read_sector(struct ata_channel *ch, uint8_t dev_idx, uint32_t lba, uint16_t *buf);
int ata_write_sector(struct ata_channel *ch, uint8_t dev_idx, uint32_t lba, const uint16_t *buf);

extern struct ata_channel *ata_primary_ptr;
extern struct ata_channel *ata_secondary_ptr;

#endif /*__realix_ata__*/