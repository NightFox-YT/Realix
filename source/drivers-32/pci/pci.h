/* Pci driver for Realix
 * The PCI scanner realization
*/
#ifndef __realix_pci_h__
#define __realix_pci_h__

#include "../../libc/stdint.h"

#define PCI_CONFIG_ADDRESS 0xCF8
#define PCI_CONFIG_DATA 0xCFC

uint32_t pci_read_config(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset);
void pci_init(void);

#endif /*__realix_pci_h__*/