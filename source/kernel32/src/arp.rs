use crate::vga;

pub fn send_arp_request(target_ip: [u8; 4]) {
    if !crate::rtl8139::is_ready() {
        vga::print_str("[ARP] No network card!\n", vga::Color::Red);
        return;
    }
    
    vga::print_str("[ARP] Sending...\n", vga::Color::Cyan);
    
    // Минимум 60 байт для Ethernet!
    let mut packet: [u8; 60] = [0; 60];
    
    // Ethernet заголовок (14 байт)
    packet[0] = 0xFF; packet[1] = 0xFF; packet[2] = 0xFF;
    packet[3] = 0xFF; packet[4] = 0xFF; packet[5] = 0xFF; // Broadcast
    packet[6] = 0x52; packet[7] = 0x54; packet[8] = 0x00;
    packet[9] = 0x12; packet[10] = 0x34; packet[11] = 0x56; // Source MAC
    packet[12] = 0x08; packet[13] = 0x06; // ARP type
    
    // ARP заголовок (28 байт)
    packet[14] = 0x00; packet[15] = 0x01; // Hardware: Ethernet
    packet[16] = 0x08; packet[17] = 0x00; // Protocol: IPv4
    packet[18] = 0x06; // HW size
    packet[19] = 0x04; // Proto size
    packet[20] = 0x00; packet[21] = 0x01; // Operation: Request
    
    // Sender MAC
    packet[22] = 0x52; packet[23] = 0x54; packet[24] = 0x00;
    packet[25] = 0x12; packet[26] = 0x34; packet[27] = 0x56;
    
    // Sender IP: 10.0.2.15
    packet[28] = 10; packet[29] = 0; packet[30] = 2; packet[31] = 15;
    
    // Target MAC: нули (уже)
    
    // Target IP
    packet[38] = target_ip[0];
    packet[39] = target_ip[1];
    packet[40] = target_ip[2];
    packet[41] = target_ip[3];
    
    // Байты 42-59 — padding (уже нули)
    
    crate::rtl8139::send_packet(&packet);
    vga::print_str("[ARP] Sent, waiting...\n", vga::Color::Green);
    
    let mut buf: [u8; 2048] = [0; 2048];
    for _ in 0..5000000 {
        if let Some(len) = crate::rtl8139::receive_packet(&mut buf) {
            if len >= 14 && buf[12] == 0x08 && buf[13] == 0x06 {
                vga::print_str("[ARP] Reply! MAC=", vga::Color::Green);
                for i in 22..28 {
                    let hex = b"0123456789ABCDEF";
                    vga::put_char(hex[(buf[i]>>4) as usize], vga::Color::White);
                    vga::put_char(hex[(buf[i]&0xF) as usize], vga::Color::White);
                    if i < 27 { vga::put_char(b':', vga::Color::White); }
                }
                vga::put_char(b'\n', vga::Color::LightGray);
                return;
            }
        }
    }
    vga::print_str("[ARP] Timeout\n", vga::Color::Red);
}
