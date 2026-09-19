use core::ptr;
use crate::drivers::vga::{self, Color};
use crate::drivers::keyboard;

const _0:u32=0x46554747;
static mut _1:usize=1536;static mut _2:usize=128;static mut _3:usize=12;
static mut _4:usize=28;static mut _5:usize=8960;static mut _6:usize=512;
static mut _7:bool=true;

fn _8(v:u16)->f32{
    let _9=((v as u32)>>15)<<31;
    let _10=((v as u32)>>10)&0x1F;
    let _11=(v as u32)&0x3FF;
    if _10==0{
        if _11==0{f32::from_bits(_9)}
        else{let _12=_11.leading_zeros()-22;f32::from_bits(_9|(((127-15-_12 as i32)as u32)<<23)|((_11<<(_12+1))&0x7FFFFF))}
    }else if _10==31{f32::from_bits(_9|(0xFF<<23)|(_11<<13))}
    else{f32::from_bits(_9|(((_10+112)as u32)<<23)|(_11<<13))}
}

unsafe fn _13(p:*const u8,_14:usize)->u32{ptr::read_unaligned(p.add(_14)as*const u32)}
unsafe fn _15(p:*const u8,_16:usize)->u64{ptr::read_unaligned(p.add(_16)as*const u64)}
unsafe fn _17(p:*const u8,_18:usize)->u8{ptr::read_volatile(p.add(_18))}
unsafe fn _19(_20:usize)->u16{ptr::read_unaligned(_20 as*const u16)}
unsafe fn _21(_22:usize)->f32{ptr::read_unaligned(_22 as*const f32)}

fn _23(_24:f32)->f32{
    if _24<=0.0{0.0}
    else{let mut _25=f32::from_bits((_24.to_bits()>>1)+(127<<22));for _ in 0..4{_25=0.5*(_25+_24/_25)}_25}
}
fn _26(_27:f32)->f32{
    if _27< -10.0{0.0}else if _27>10.0{f32::MAX}
    else{let mut _28=1.0;let mut _29=1.0;for _30 in 1..=6{_29*= _27/(_30 as f32);_28+=_29}_28}
}
fn _31(_32:f32)->f32{_32/(1.0+_26(-_32))}

unsafe fn _33(_34:usize,_35:usize,_36:usize,_37:&[f32],_38:&mut[f32]){
    let _39=(_36+31)/32;
    let _40=_39*34;
    for _41 in 0.._36{_38[_41]=0.0}
    for _42 in 0.._35{
        let _43=_37[_42] as f64;
        for _44 in 0.._39{
            let _45=_34+_42*_40+_44*34;
            let _46=_8(_19(_45)) as f64;
            for _47 in 0..32{
                let _48=_44*32+_47;
                if _48>=_36{break}
                let _49=_17(_45 as*const u8,2+_47) as i8;
                _38[_48]=(_38[_48] as f64+_49 as f64*_46*_43) as f32;
            }
        }
    }
}
unsafe fn _50(_51:usize,_52:usize,_53:usize,_54:&[f32],_55:&mut[f32]){
    for _56 in 0.._53{let mut _57=0.0f64;for _58 in 0.._52{_57+=_8(_19(_51+(_58*_53+_56)*2))as f64*_54[_58]as f64}_55[_56]=_57 as f32}
}
unsafe fn _59(_60:usize,_61:usize,_62:usize,_63:&[f32],_64:&mut[f32]){
    if _7{_33(_60,_61,_62,_63,_64)}else{_50(_60,_61,_62,_63,_64)}
}

struct _65{_66:usize,_67:usize,_68:[usize;28],_69:[usize;28],_70:[usize;28],_71:[usize;28],_72:[usize;28],_73:[usize;28],_74:[usize;28],_75:[usize;28],_76:[usize;28],_77:usize}
static mut _78:Option<_65>=None;static mut _79:usize=0;static mut _80:bool=false;

struct _81{_82:[f32;1536],_83:[f32;1536],_84:[f32;1536],_85:[f32;1536],_86:[f32;1536],_87:[f32;1536],_88:[f32;8960],_89:[f32;8960]}
static mut _90:_81=_81{
    _82:[0.;1536],_83:[0.;1536],_84:[0.;1536],_85:[0.;1536],_86:[0.;1536],
    _87:[0.;1536],_88:[0.;8960],_89:[0.;8960]
};

unsafe fn _91(_92:&[f32],_93:usize,_94:usize,_95:&mut[f32]){
    let _96=1e-6f32;let mut _97=0.0;for _98 in 0.._94{_97+=_92[_98]*_92[_98]}_97=_97/_94 as f32;let _99=1.0/_23(_97+_96);for _100 in 0.._94{_95[_100]=_92[_100]*_99*_21(_93+_100*4)}
}

unsafe fn _101(_102:&_65,_103:&mut _81,_104:u32,_105:usize)->u32{
    let _106=_1;let _107=_2;
    let _108=_104 as usize*_106;for _109 in 0.._106{_103._82[_109]=_8(_19(_102._66+(_108+_109)*2))}
    for _110 in 0.._4{
        _91(&_103._82,_102._68[_110],_106,&mut _103._83);
        _59(_102._70[_110],_106,_106,&_103._83,&mut _103._84);
        _59(_102._71[_110],_106,_106,&_103._83,&mut _103._85);
        _59(_102._72[_110],_106,_106,&_103._83,&mut _103._86);
        for _111 in 0.._106{_103._87[_111]=0.0}
        for _112 in 0.._3{let _113=_112*_107;let mut _114=0.0;for _115 in 0.._107{_114+=_103._84[_113+_115]*_103._85[_113+_115]}_114/=_23(_107 as f32);let _116=_26(_114);for _115 in 0.._107{_103._87[_113+_115]+=_116*_103._86[_113+_115]}}
        _59(_102._73[_110],_106,_106,&_103._87,&mut _103._84);
        for _111 in 0.._106{_103._82[_111]+=_103._84[_111]}
        _91(&_103._82,_102._69[_110],_106,&mut _103._83);
        _59(_102._74[_110],_106,_5,&_103._83,&mut _103._88);
        _59(_102._75[_110],_106,_5,&_103._83,&mut _103._89);
        for _111 in 0.._5{_103._88[_111]=_31(_103._88[_111])*_103._89[_111]}
        _59(_102._76[_110],_5,_106,&_103._88,&mut _103._84);
        for _111 in 0.._106{_103._82[_111]+=_103._84[_111]}
        vga::print_char(b'.', Color::LightGray);
    }
    _91(&_103._82,_102._77,_106,&mut _103._83);
    let _117=if _102._67!=0{_102._67}else{_102._66};
    let _118=_6;
    let mut _119=0u32;
    let mut _120=0.0f64;
    for _121 in 0.._118{
        let _122=_121*_106;
        let mut _123=0.0f64;
        for _124 in 0.._106{_123+=_8(_19(_117+(_122+_124)*2)) as f64*_103._83[_124] as f64}
        if _121==0||_123>_120{_120=_123;_119=_121 as u32}
    }
    _119
}

unsafe fn _125(_126:usize,_127:usize,_128:u64,_129:&[u8])->Option<usize>{
    let mut _130=_127;
    for _ in 0.._128{
        let _131=_15(_126 as*const u8,_130)as usize;let _132=_130+8;_130+=8+_131;
        let _133=_13(_126 as*const u8,_130)as usize;_130+=4;
        for _ in 0.._133{_130+=8}
        let _134=_13(_126 as*const u8,_130);_130+=4;
        let _135=_15(_126 as*const u8,_130)as usize;_130+=8;
        if _131==_129.len(){
            let mut _136=true;
            for _137 in 0.._131{if _17(_126 as*const u8,_132+_137)!=_129[_137]{_136=false;break}}
            if _136{return Some(_79+_135)}
        }
    }
    None
}

unsafe fn _138(){
    if _80{return}
    for &_139 in &[0x10000000usize,0x2000000,0x4000000,0x8000000]{
        if _13(_139 as*const u8,0)!=_0{continue}
        _79=_139;
        let _140=_13(_139 as*const u8,4);
        let _141=_15(_139 as*const u8,8);
        let _142=_15(_139 as*const u8,16)as usize;
        let mut _143=24usize;
        for _ in 0.._142{
            let _144=_15(_139 as*const u8,_143)as usize;_143+=8;
            _143+=_144;
            let _145=_13(_139 as*const u8,_143);_143+=4;
            match _145{
                0|1=>_143+=1,2|3=>_143+=2,4|5|6=>_143+=4,7=>_143+=1,
                8=>{let _146=_15(_139 as*const u8,_143)as usize;_143+=8+_146}
                9=>{let _147=_13(_139 as*const u8,_143);_143+=4;let _148=_15(_139 as*const u8,_143)as usize;_143+=8;
                    for _ in 0.._148{
                        match _147{
                            0|1=>_143+=1,2|3=>_143+=2,4|5|6=>_143+=4,7=>_143+=1,
                            8=>{let _146=_15(_139 as*const u8,_143)as usize;_143+=8+_146}
                            _=>{_143=0;break}
                        }
                    }
                    if _143==0{break}
                }
                10|11|12=>_143+=8,
                _=>{_143=0;break}
            }
        }
        if _143==0{continue}
        let _149=_143;
        let _150=_125(_139,_149,_141,b"token_embd.weight")
            .or_else(||_125(_139,_149,_141,b"model.embed_tokens.weight"))
            .unwrap_or(0);
        if _150==0{continue}
        _1=1536;_2=128;_3=_1/_2;_4=28;_5=_1*4;_6=512;_7=true;
        let _151=_125(_139,_149,_141,b"lm_head.weight")
            .or_else(||_125(_139,_149,_141,b"model.output.weight"))
            .unwrap_or(0);
        let mut _152=_65{_66:_150,_67:_151,_68:[0;28],_69:[0;28],_70:[0;28],_71:[0;28],_72:[0;28],_73:[0;28],_74:[0;28],_75:[0;28],_76:[0;28],_77:0};
        for _153 in 0.._4{
            let mut _154=[0u8;64];
            let _155=b"model.layers.";
            for _156 in 0..13{_154[_156]=_155[_156]}
            if _153<10{
                _154[13]=b'0'+_153 as u8;_154[14]=b'.';
                macro_rules!A{($L:expr,$R:expr)=>{let _157=15;for _158 in 0..$L.len(){_154[_157+_158]=$L[_158]}$R=_125(_139,_149,_141,&_154[.._157+$L.len()]).unwrap_or(0)}}
                A!(b"self_attn.q_proj.weight", _152._70[_153]);
                A!(b"self_attn.k_proj.weight", _152._71[_153]);
                A!(b"self_attn.v_proj.weight", _152._72[_153]);
                A!(b"self_attn.o_proj.weight", _152._73[_153]);
                A!(b"input_layernorm.weight", _152._68[_153]);
                A!(b"post_attention_layernorm.weight", _152._69[_153]);
                A!(b"mlp.gate_proj.weight", _152._74[_153]);
                A!(b"mlp.up_proj.weight", _152._75[_153]);
                A!(b"mlp.down_proj.weight", _152._76[_153]);
            }else{
                _154[13]=b'0'+(_153/10)as u8;_154[14]=b'0'+(_153%10)as u8;_154[15]=b'.';
                macro_rules!B{($L:expr,$R:expr)=>{let _157=16;for _158 in 0..$L.len(){_154[_157+_158]=$L[_158]}$R=_125(_139,_149,_141,&_154[.._157+$L.len()]).unwrap_or(0)}}
                B!(b"self_attn.q_proj.weight", _152._70[_153]);
                B!(b"self_attn.k_proj.weight", _152._71[_153]);
                B!(b"self_attn.v_proj.weight", _152._72[_153]);
                B!(b"self_attn.o_proj.weight", _152._73[_153]);
                B!(b"input_layernorm.weight", _152._68[_153]);
                B!(b"post_attention_layernorm.weight", _152._69[_153]);
                B!(b"mlp.gate_proj.weight", _152._74[_153]);
                B!(b"mlp.up_proj.weight", _152._75[_153]);
                B!(b"mlp.down_proj.weight", _152._76[_153]);
            }
        }
        _152._77=_125(_139,_149,_141,b"model.norm.weight")
            .or_else(||_125(_139,_149,_141,b"output_norm.weight"))
            .unwrap_or(0);
        _78=Some(_152);_80=true;break
    }
}

fn _159(){
    let _160:[[u8;26];5]=[
        [1,0,0,0,1,0,0,1,1,1,0,0,1,0,0,0,1,0,0,1,1,1,0,0,0,0],
        [1,1,0,0,1,0,1,0,0,0,1,0,1,0,0,0,1,0,1,0,0,0,1,0,0,0],
        [1,0,1,0,1,0,1,0,0,0,1,0,1,0,0,0,1,0,1,1,1,1,1,0,0,0],
        [1,0,0,1,1,0,1,0,0,0,1,0,0,1,0,1,0,0,1,0,0,0,1,0,0,0],
        [1,0,0,0,1,0,0,1,1,1,0,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0],
    ];
    let _161=[Color::Red,Color::Yellow,Color::Green,Color::Cyan];
    let _162=[(0,5),(6,11),(12,17),(18,23)];
    for _163 in 0..30{vga::write_char_at(1,_163,b'=',Color::DarkGray)}
    for(_164,_165)in _160.iter().enumerate(){
        for(_166,(_167,_168))in _162.iter().enumerate(){
            for _169 in *_167..*_168{
                if _169<_165.len()&&_165[_169]==1{
                    vga::write_char_at(_164+2,_169+2,0xDB,_161[_166])
                }
            }
        }
    }
    for _163 in 0..30{vga::write_char_at(7,_163,b'=',Color::DarkGray)}
}

fn _170(){
    vga::clear_screen();
    _159();
    for _ in 0..10{vga::print_new_line();}
    vga::print_line("     Realix X NOVA Pilot v0.1\n",Color::Cyan);
    vga::print_line("     Qwen 2.5 1.5B Q8_K\n",Color::Cyan);
    vga::print_line("______________________________\n\n",Color::DarkGray);
}

pub unsafe fn BC(_172:&str){
    _138();
    match _172.trim(){
        "-a"=>_173(),
        _=>{
            vga::print_line("NOVA v0.1 - Realix X NOVA Pilot\n",Color::Cyan);
            vga::print_line("  nova -a - Qwen 2.5 interactive\n",Color::LightGray);
            if _80{vga::print_line("  [Model loaded]\n",Color::Green)}
        }
    }
}

unsafe fn _173(){
    core::arch::asm!("sti");
    _170();
    vga::print_new_line();
    vga::print_new_line();
    if !_80{
        vga::print_line("[WARN] Model not loaded.\n",Color::Yellow);
        keyboard::read_key();
        return
    }
    vga::print_line("[READY] Type 'exit' or ESC to quit.\n",Color::Green);
    let _174=match &_78{Some(_175)=>_175 as*const _65,None=>{return}};
    let _176=&*_174;
    let _177=&mut _90;
    loop{
        vga::print_line("USER> ",Color::Yellow);
        let mut _178=[0u8;64];
        let mut _179=0usize;
        loop{
            match keyboard::read_key(){
                keyboard::Key::Escape=>{
                    vga::print_line("\nESC\n",Color::Cyan);
                    return
                }
                keyboard::Key::Char(b'\n')=>{
                    _178[_179]=0;
                    vga::print_new_line();
                    break
                }
                keyboard::Key::Char(0x08)=>{
                    if _179>0{_179-=1;vga::print_backspace()}
                }
                keyboard::Key::Char(_180)if _179<63&&_180>=0x20=>{
                    _178[_179]=_180;_179+=1;vga::print_char(_180,Color::LightGray)
                }
                _=>{}
            }
        }
        let _181=core::str::from_utf8(&_178).unwrap_or("");
        let _182=_181.find('\0').unwrap_or(_181.len());
        let _183=_181[.._182].trim();
        if _183.is_empty(){continue}
        if _183=="exit"||_183=="quit"{
            vga::print_line("bye\n",Color::Cyan);
            return
        }
        vga::print_line("NOVA> ",Color::Green);
        let _184=_183.as_bytes();
        let mut _185=0u32;
        for _186 in 0.._184.len(){
            _185=_101(_176,_177,_184[_186]as u32,_186);
        }
        let _187=_101(_176,_177,_185,_184.len());
        vga::print_line(" [done]\n",Color::Green);
        vga::print_new_line();
    }
}