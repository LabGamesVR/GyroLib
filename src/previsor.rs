use std::{
    sync::{Arc, Mutex}, thread
};
use include_dir::{include_dir, Dir};

use crate::dispositivo::{ Dispositivo, DISPOSITIVOS_REGISTRADOS};

const DIR_REDES: Dir = include_dir!("redes");

pub struct Previsor {
    resposta_atual: Arc<Mutex<&'static str>>,
    neural: Option<Arc<Mutex<neuroflow::FeedForward>>>,
    prevendo: Arc<Mutex<bool>>,
    dispositivo: &'static Dispositivo,
}

impl Previsor {
    pub fn obter_movimento(&self, dados: Vec<f64>) -> &'static str {
        if !*self.prevendo.lock().unwrap() {
            //inicia thread para prever
            if let Some(nn) = &self.neural {
                let nn_ref = Arc::clone(nn);
                let resposta_atual_ref:Arc<Mutex<&'static str>> = Arc::clone(&self.resposta_atual);
                let prevendo_ref= Arc::clone(&self.prevendo);
                let dispositivo_ref:&'static Dispositivo = self.dispositivo;
            thread::spawn(move || {
                    let d = dados;
                    if let Ok(mut v) = nn_ref.lock(){
                        if d.len()>=2{
                            let d = &d[0..2];
                            println!("L1");
                            println!("{:?}",d);
                            let res = v.calc(&d);
                            println!("L2");
                            println!("{:?}",res);
                            println!("L3");
                            let max_i = res
                            .iter()
                            .enumerate()
                            .fold(
                                (0, 0.0),
                                |max, (ind, &val)| if val > max.1 { (ind, val) } else { max },
                            )
                            .0;
                            println!("L4");
                            if let Some(mov) = dispositivo_ref.movimentos.get(max_i) {
                                *resposta_atual_ref.lock().unwrap() = mov.0;
                                *prevendo_ref.lock().unwrap() = false;
                                
                            }
                        }
                    }
                });
            }
        }

        return &self.resposta_atual.lock().unwrap();
    }
    pub fn new(id_dispositivo:&str) -> Option<Previsor>{
        use std::io::Write;

        let glob = id_dispositivo.to_owned() + ".flow";
        for entry in DIR_REDES.files() {
            let str_name = entry.path().file_name().unwrap().to_str().unwrap();
            if str_name == &glob {
                println!("Achou!");
                let mut addr = std::env::temp_dir();
                addr.push(str_name);

                //escreve o arquivo incluido durante runtime
                match std::fs::OpenOptions::new().write(true).create(true).open(&addr) {
                    Ok(mut file) => {
                        if let Err(err) = file.write_all(entry.contents()) {
                            eprintln!("Falha ao escrever em arquivo: {}", err);
                        }else {
                            println!("Arquivo escrito em {}",addr.to_str().unwrap());
                        }
                    }
                    Err(err) => {
                        eprintln!("Falha ao abrir arquivo: {}", err);
                    }
                }

                // carrega rede neural
                let new_nn: Result<neuroflow::FeedForward, neuroflow::ErrorKind> = neuroflow::io::load(addr.to_str().unwrap());

                //agora que já usou, exclui arquivo
                match std::fs::remove_file(&addr) {
                    Ok(_) => println!("File deleted: {}", str_name),
                    Err(err) => eprintln!("Failed to delete file: {}", err),
                }
                if let Ok(nn) = new_nn {
                    if let Some(dispositivo) = DISPOSITIVOS_REGISTRADOS.obter_dispositivo_com_id(id_dispositivo) {
                        return Some(
                            Previsor{
                                resposta_atual:Arc::new(Mutex::new("")),
                                neural: Some(Arc::new(Mutex::new(nn))),
                                prevendo: Arc::new(Mutex::new(false)),
                                dispositivo: dispositivo
                            }
                        );
                    }
                }
            }
        }
        None
    }
}

pub enum LazyPrevisor {
    NaoLido,
    Invalido,
    Lido(Previsor)
}

impl LazyPrevisor {
        


        // let n = Neural::carregar("papete.pt").unwrap();

        // //agora que já usou, exclui arquivo
        // match fs::remove_file(&file_path) {
        //     Ok(_) => println!("File deleted: {}", file_path),
        //     Err(err) => eprintln!("Failed to delete file: {}", err),
        // }
    pub fn prever_movimento(&mut self, dados: &Vec<f32>, id_dispositivo:&str) -> Option<&'static str> {
        let mut retorno = None;
        let novo_tipo: Option<LazyPrevisor> = match self {
            Self::NaoLido => {
                retorno = None;
                //inicia thread para ler
                if let Some(previsor) = Previsor::new(id_dispositivo) {
                    Some(Self::Lido(previsor))
                }
                else {
                    Some(Self::Invalido)
                }
            }
            Self::Invalido => None,
            Self::Lido(previsor) => {
                let v:Vec<f64> = dados.iter().map(|valor32| *valor32 as f64).collect();
                retorno = Some(previsor.obter_movimento(v));
                None
            },
        };
        if let Some(tipo) = novo_tipo {
            *self = tipo;
        }
        retorno
    }
}
