void kernel_main(){
    volatile char* video_memory = (volatile char*) 0xB8000;
    char* name_os = "RealixOS";
    int i = 0; //Счётчик для букв в слове
    int vga = 0; //Счётчик для позиций на экране
    
    while (name_os[i] != '\0') //работает пока не встретит конец строки
    {
        video_memory[vga] = name_os[i]; //буква
        video_memory[vga + 1] = 0x01; //цвет
        i++; //Переход к следующей букве 
        vga = vga + 2; 
    }
    
}