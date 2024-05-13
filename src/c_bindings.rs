use std::{ffi::CString, mem::transmute};

use crate::sensor::Sensores;


#[no_mangle]
pub unsafe extern "C" fn alocar_receptor() -> *mut Sensores {
    println!("Alocado");
    transmute(Box::new(Sensores::new()))
    // Box::into_raw(s)
}

#[no_mangle]
pub unsafe extern "C" fn liberar_receptor(s: *mut Sensores) -> i32 {
    println!("Liberado");
    let s: Box<Sensores> = transmute(s);
    drop(s);
    return 42;
}


/*
Preenche um array com todos os valores lidos, retorna quantidade de valores
*/
#[no_mangle]
pub unsafe extern "C" fn leitura_bruta(s: *mut Sensores, buffer: *mut f32) ->i32 {
    let mut i = 0;
    for sensor in (*s).obter_valores() {
        for valor in sensor {
            let ptr = buffer.offset(i);
            *ptr = *valor;
            i+=1;
        }
    }
    i as i32
}

/*
Preenche um array com todos os valores lidos
*/
#[no_mangle]
pub unsafe extern "C" fn sensores_conectados(s: *mut Sensores, buffer: *mut u8) -> i32 {
    let mut preenchidos = 0;
    let mut contador = 0;
    for sensor in (*s).obter_sensores_ativos(){
        let c_string = CString::new(sensor.as_str()).unwrap();
        for byte in c_string.as_bytes() {
            *(buffer.offset(preenchidos)) = *byte;
            preenchidos += 1;
        }
        *(buffer.offset(preenchidos)) = b',';
        preenchidos += 1;
        contador += 1;
    }
    //null-terminator
    *(buffer.offset(preenchidos - 1)) = 0;
    contador
}

#[no_mangle]
pub unsafe extern "C" fn teste()->i32{
    return 42;
}