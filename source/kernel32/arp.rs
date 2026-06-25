use crate::vga;

/// ARP-запрос: "Кто имеет IP 10.0.2.2? Скажи свой MAC!"
pub fn send_arp_request(target_ip: [u8; 4]) {
    vga::print_str("[ARP] Sending ARP request...\n", vga::Color::Cyan);
    
    // Собираем Ethernet-фрейм вручную
    let mut packet: [u8; 42] = [0; 42];
    
    // Ethernet: destination MAC = broadcast
    packet[0] = 0xFF; packet[1] = 0xFF; packet[2] = 0xFF;
    packet[3] = 0xFF; packet[4] = 0xFF; packet[5] = 0xFF;
    // Ethernet: source MAC (заглушка, заменим на реальный)
    packet[6] = 0x52; packet[7] = 0x54; packet[8] = 0x00;
    packet[9] = 0x12; packet[10] = 0x34; packet[11] = 0x56;
    // Ethernet: тип = ARP (0x0806)
    packet[12] = 0x08; packet[13] = 0x06;
    
    // ARP: hardware type = Ethernet (0x0001)
    packet[14] = 0x00; packet[15] = 0x01;
    // ARP: protocol type = IPv4 (0x0800)
    packet[16] = 0x08; packet[17] = 0x00;
    // ARP: hardware size = 6, protocol size = 4
    packet[18] = 0x06; packet[19] = 0x04;
    // ARP: operation = Request (0x0001)
    packet[20] = 0x00; packet[21] = 0x01;
    // ARP: sender MAC
    packet[22] = 0x52; packet[23] = 0x54; packet[24] = 0x00;
    packet[25] = 0x12; packet[26] = 0x34; packet[27] = 0x56;
    // ARP: sender IP = 10.0.2.15 (QEMU default)
    packet[28] = 10; packet[29] = 0; packet[30] = 2; packet[31] = 15;
    // ARP: target MAC = 00:00:00:00:00:00
    // (уже нули)
    // ARP: target IP
    packet[38] = target_ip[0];
    packet[39] = target_ip[1];
    packet[40] = target_ip[2];
    packet[41] = target_ip[3];
    
    // Отправляем через RTL8139
    crate::rtl8139::send_packet(&packet);
    
    vga::print_str("[ARP] Request sent!\n", vga::Color::Green);
}
