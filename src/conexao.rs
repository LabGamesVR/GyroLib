#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Conexao {
    Desconectado,
    USB(String),
    WiFi,
}
