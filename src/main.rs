extern crate unidecode;

use std::{
    ffi::CStr,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::{self, Duration}
};

use sensor::Sensores;

pub mod c_bindings;
pub mod comm;
pub mod dispositivo;
pub mod sensor;
pub mod previsor;

fn treinar() -> std::io::Result<()> {
    std::fs::create_dir_all("redes").unwrap();
    let paths = std::fs::read_dir("coletas").unwrap();
    for e in paths {
        if let Ok(e) = e {
            let addr = e.path();
            println!(
                "addr: {:?}",
                addr.with_extension("")
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            );
            if let Some(dados) = dispositivo::DISPOSITIVOS_REGISTRADOS.obter_dados_dispositivo(
                addr.with_extension("")
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            ) {
                println!("{}", dados.nome);
                let mut dataset: neuroflow::data::DataSet = neuroflow::data::DataSet::new();

                let file = std::fs::File::open(addr)?;
                let mut reader = csv::Reader::from_reader(file);

                for record in reader.records() {
                    let linha = record.unwrap();
                    let linha: Vec<&str> = linha.as_slice().split(';').collect();
                    if let Ok(pitch) = linha[0].parse::<f64>() {
                        if let Ok(roll) = linha[1].parse::<f64>() {
                            if let Some(index_movimento) =
                                dados.movimentos.iter().position(|item| item.0 == linha[2])
                            {
                                let mut saidas = Vec::with_capacity(dados.movimentos.len());
                                for _ in 0..dados.movimentos.len() {
                                    saidas.push(0.0);
                                }
                                saidas[index_movimento] = 1.0;
                                dataset.push(&[pitch, roll], &saidas)
                            }
                        }
                    }
                }

                let mut nn = neuroflow::FeedForward::new(&[
                    2,
                    30,
                    30,
                    dados.movimentos.len().try_into().unwrap(),
                ]);
                nn.activation(neuroflow::activators::Type::Tanh)
                    .learning_rate(0.01)
                    .train(&dataset, 1000);

                let output_addr = PathBuf::from(format!("redes/{}.flow", dados.identificador));
                neuroflow::io::save(&mut nn, output_addr.to_str().unwrap()).unwrap();
            }
        }
    }
    Ok(())
}
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

                            dispositivos.iter().enumerate().for_each(
                                |(index_dispositivo, dispositivo)| {
                                    if let Some(movimento) =
                                        dispositivo.movimentos.get(index_movimento)
                                    {
                                        write!(
                                            &mut arquivos[index_dispositivo],
                                            "\n{};{};{}",
                                            valores[index_dispositivo][0],
                                            valores[index_dispositivo][1],
                                            movimento.0
                                        )
                                        .unwrap();
                                    }
                                },
                            );

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
fn testar() {
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
        println!("{:?}",sensores.obter_movimento_primeiro_sensor());
        thread::sleep(Duration::from_millis(2000));
    }
}

fn teste_c_bindings() {
    // for _ in 0..10 {
    //     let ptr = unsafe { c_bindings::alocar_receptor() };
    //     let mut buffer: [u8; 100] = [0; 100];
    //     unsafe{
    //         c_bindings::sensores_conectados(ptr, buffer.as_mut_ptr())
    //     };
    //     unsafe{ c_bindings::liberar_receptor(ptr)};
    //     thread::sleep(time::Duration::from_millis(200));
    //     println!("");
    // }
    let ptr = unsafe { c_bindings::alocar_receptor() };
    let mut u8buffer: [u8; 100] = [0; 100];
    let mut f32buffer: [f32; 100] = [0.0; 100];
    loop {
        let n_sensores = unsafe { c_bindings::sensores_conectados(ptr, u8buffer.as_mut_ptr()) };
        for float in 0..unsafe { c_bindings::leitura_bruta(ptr, f32buffer.as_mut_ptr()) } {
            println!("== {}", f32buffer[float as usize]);
        }
        println!("{}: {}", n_sensores, unsafe {
            CStr::from_ptr(u8buffer.as_ptr() as *const _)
                .to_str()
                .unwrap()
        });
        thread::sleep(time::Duration::from_millis(200));
    }
}

fn main() {
    testar();
    //teste_c_bindings();
    //treinar().unwrap();
}
