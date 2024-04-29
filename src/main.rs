
use std::{thread, time::Duration};

use sensor::Sensores;


pub mod comm;
pub mod sensor;

fn main() {
    let sensores = Sensores::new();

    let mut buffer:Vec<Vec<f32>> = Vec::new();
    loop {
        println!("");
        println!("");
        println!("{:?}",sensores.obter_sensores_ativos());
        sensores.obter_valores(&mut buffer);
        for s in &buffer {
            for valor in s {
                print!("\t{}",valor);
            }
        }
        println!("");
        thread::sleep(Duration::from_millis(200));
    }

    // let mut line = String::new();
    // let _ = std::io::stdin().read_line(&mut line).unwrap();
}
