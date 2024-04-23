use std::{sync::{Arc, Mutex}, thread};
use queue::Queue;
use crate::comm::Comm;

pub struct Sensor{
    pub device: String,
    pub values : Vec<f32>
}

pub struct Sensores{
    pub sensores:Arc<Mutex<Vec<Sensor>>>,
    _comm: Comm
}

fn filtro(msg: &str)->bool{
    /*
    [a-zA-Z]: matches any single letter (uppercase or lowercase).
    [^\t]*: matches any number of symbols except for a tab character.
    \t: matches a tab character.
    .{3,}: matches at least three symbols.
    */
    let re = regex::Regex::new(r"[a-zA-Z][^\t]*\t.{3,}").unwrap();
    if re.is_match(msg){
        true    
    }
    else {
        println!("msg falha: \"{}\"",msg);
        false
    }
}

impl Sensores {
    pub fn new()->Self{
        let queue = Arc::new(Mutex::new(Queue::new()));
        let comm = Comm::filtered(Arc::clone(&queue), &filtro);
        
        let sensores = Arc::new(Mutex::new(Vec::new()));
        let copia = Arc::clone(&sensores);

        thread::spawn(move || Sensores::listener(queue, copia));
        let s = Sensores{
            sensores,
            _comm: comm
        };
        s
    }
    fn listener(queue: Arc<Mutex<Queue<String>>>, sensores: Arc<Mutex<Vec<Sensor>>>){
        loop {
            if let Some(msg)  = queue.lock().unwrap().dequeue() {
                let iterator = msg.split('\t');
                if let Some(device) = iterator.clone().take(1).collect::<Vec<&str>>().get(0){
                    let values: Vec<f32> = iterator.skip(1).map(|v| v.parse::<f32>().unwrap_or(0.0)).collect();
                    if let Ok(mut s) = sensores.lock(){
                        if let Some(index) = s.iter().position(|sensor| &sensor.device == device){
                            s[index].values = values;
                        }
                    }
                }
                println!("{}",msg);
            }
        }
    }
}