extern crate unidecode;

use std::{
    io::{self, Write},
    path::Path,
    sync::mpsc,
    thread,
    time::Duration,
};

use sensor::Sensores;

pub mod comm;
pub mod dispositivo;
pub mod sensor;

fn coletar() {
    let intervalo = Duration::from_millis(80);
    let rodadas = 1;
    println!(
        "Quando forem conectados todos os dispositivos que você pretende coletar, tecle \"enter\""
    );

    let (transmissor_sensores, receptor_sensores) = mpsc::channel();

    let (transmissor_fim, receptor_fim) = mpsc::channel::<()>();
    thread::spawn(move || {
        let sensores = Sensores::new();

        let mut dispositivos;
        let mut dispositivos_impressos = Vec::new();

        loop {
            dispositivos = sensores.obter_sensores_ativos();
            if dispositivos != dispositivos_impressos {
                println!("{:?}", dispositivos);
                dispositivos_impressos = dispositivos.clone();
            }

            match receptor_fim.try_recv() {
                Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                    transmissor_sensores
                        .send((sensores, dispositivos.len()))
                        .unwrap();
                    thread::sleep(Duration::from_millis(100));
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
    });
    io::stdin().read_line(&mut String::new()).unwrap();
    transmissor_fim.send(()).unwrap();

    loop {
        match receptor_sensores.try_recv() {
            Ok((mut sensores, qtd_dispositivos)) => {
                let mut arquivos = Vec::with_capacity(qtd_dispositivos);
                for identificador in sensores.obter_sensores_ativos() {
                    let pasta = "coletas";
                    let addr_string = format!("{}/{}.csv", pasta, identificador);
                    let addr = Path::new(&addr_string);
                    let arquivo = if addr.is_file() {
                        std::fs::OpenOptions::new().append(true).open(addr).unwrap()
                    } else {
                        std::fs::create_dir_all(pasta).unwrap();
                        let mut a = std::fs::File::create(addr).unwrap();
                        a.write_all("pitch;roll;movimento".as_bytes()).unwrap();
                        a
                    };
                    arquivos.push(arquivo);
                }
                for rodada in 0..rodadas {
                    println!("Rodada de coleta {} / {}", rodada + 1, rodadas);

                    let dispositivos: Vec<&dispositivo::Dispositivo> = sensores
                        .obter_sensores_ativos()
                        .iter()
                        .map(|item| {
                            dispositivo::DISPOSITIVOS_REGISTRADOS.obter_dados_dispositivo(item)
                        })
                        .filter(|e| e.is_some())
                        .map(|e| e.unwrap())
                        .collect();

                    let max_movimentos = dispositivos.iter().fold(0, |acumulador, dispositivo| {
                        std::cmp::max(acumulador, dispositivo.movimentos.len())
                    });

                    for index_movimento in 0..max_movimentos {

                        if sensores.obter_sensores_ativos().len() < qtd_dispositivos {
                            while sensores.obter_sensores_ativos().len() < qtd_dispositivos {
                                println!("Aguardando reconexão");
                                thread::sleep(intervalo);
                            }
                        }

                        let mut dispositivos_com_movimento = 0;
                        for dispositivo in dispositivos.iter() {
                            if let Some(movimento) = dispositivo.movimentos.get(index_movimento) {
                                println!("{}: {} ({})", dispositivo.nome, movimento.0, movimento.1);
                                dispositivos_com_movimento += 1;
                            }
                        }
                        println!("\nMantenha {}, tecle \"enter\" e aguarde mantendo ela{} até a próxima ordem.", if dispositivos_com_movimento<=1{"a posição indicada"}else{"as posições indicadas"}, if dispositivos_com_movimento<=1{""}else{"s"});

                        //aguarda confirmação
                        io::stdin().read_line(&mut String::new()).unwrap();
                        for _ in 0..10 {
                            if sensores.obter_sensores_ativos().len() < qtd_dispositivos {
                                while sensores.obter_sensores_ativos().len() < qtd_dispositivos {
                                    println!("Aguardando reconexão");
                                    thread::sleep(intervalo);
                                }
                            }
                            print!(".");

                            //coletar
                            let valores = sensores.obter_valores();

                            dispositivos
                                .iter()
                                .enumerate()
                                .for_each(|(index_dispositivo, dispositivo)| {
                                    if let Some(movimento) = dispositivo.movimentos.get(index_movimento) {
                                        write!(
                                            &mut arquivos[index_dispositivo],
                                            "\n{};{};{}",
                                            valores[index_dispositivo][0], valores[index_dispositivo][1], movimento.0
                                        )
                                        .unwrap();
                                    }
                                });

                            io::stdout().flush().unwrap();
                            thread::sleep(intervalo);
                        }
                        print!("\n");
                    }
                    for _ in 0..80 {
                        print!("-");
                    }
                    print!("\n");
                }
                break;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                println!("Erro ao receber sensores do loop");
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }
    println!("\nColeta finalizada, agradecemos à participação.\n");
}
fn teste() {
    let mut sensores = Sensores::new();

    loop {
        println!("");
        println!("");
        println!("{:?}", sensores.obter_sensores_ativos());
        let buffer = sensores.obter_valores();
        for s in buffer {
            for valor in s {
                print!("\t{}", valor);
            }
        }
        println!("");
        thread::sleep(Duration::from_millis(200));
    }
}

fn main() {
    coletar();
}
