use std::usize;

/*
Caso você vá usar essa biblioteca com outros dispositivos, registre eles aqui!
Na lista DISPOSITIVOS_REGISTRADOS siga o exemplo dos casos já implementados, dando
ao seu dispositivo um nome como mostrado ao usuário, um identificador único (tal
como registrado na variável "identificador" de gyro.ino) e uma lista de movimentos
& descrições.
*/
pub struct Dispositivo {
    pub nome: &'static str,
    pub identificador: &'static str,
    pub movimentos: &'static [(&'static str,&'static str)],
}
impl Dispositivo {
    pub fn obter_identificador_movimento(&self, index:usize)->String{
        unidecode::unidecode(self.movimentos[index].0)
    }
}
pub struct Dispositivos {
    lista: &'static [Dispositivo],
}

impl Dispositivos {
    pub fn contem_id(&self, identificador: &str) -> bool {
        for elemento in self.lista {
            if elemento.identificador == identificador {
                return true;
            }
        }
        return false;
    }
    pub fn obter_dados_dispositivo(&self, identificador: &str) -> Option<&'static Dispositivo> {
        self.lista
            .iter()
            .find(|item| item.identificador == identificador)
    }
}

pub const DISPOSITIVOS_REGISTRADOS: Dispositivos = Dispositivos {
    lista: &[
        Dispositivo {
            nome: "Papete Esquerda",
            identificador: "papE",
            movimentos: &[
                ("Flexão Plantar","Levante a ponta do pé"),
                ("Dorsilexão Plantar","Levante o calcanhar"),
                ("Repouso","Apoie o pé"),
                ("Eversão","Levante o lado do pé do mindinho, apoiando no lado oposto"),
                ("Inversão","Levante o lado do pé do dedão, apoiando no lado oposto")
            ],
        },
        Dispositivo {
            nome: "Papete Direita",
            identificador: "papD",
            movimentos: &[
                ("Flexão Plantar","Levante a ponta do pé"),
                ("Dorsilexão Plantar","Levante o calcanhar"),
                ("Repouso","Apoie o pé"),
                ("Eversão","Levante o lado do pé do mindinho, apoiando no lado oposto"),
                ("Inversão","Levante o lado do pé do dedão, apoiando no lado oposto")
            ],
        },
        Dispositivo {
            nome: "Luva Esquerda",
            identificador: "luvaE",
            movimentos: &[
                ("Flexão","Abaixe a mão, mantendo o pulso reto"),
                ("Dorsilexão","Levante a mão, mantendo o pulso reto"),
                ("Repouso","Deixe a mão alinhada com o pulso"),
                ("Supinação","Gire a mão levantando o dedão, mantendo o braço reto"),
                ("Pronação","Gire a mão levantando o mindinho, mantendo o braço reto"),
            ],
        },
        Dispositivo {
            nome: "Luva Direita",
            identificador: "luvaD",
            movimentos: &[
                ("Flexão","Abaixe a mão, mantendo o pulso reto"),
                ("Dorsilexão","Levante a mão, mantendo o pulso reto"),
                ("Repouso","Deixe a mão alinhada com o pulso"),
                ("Supinação","Gire a mão levantando o dedão, mantendo o braço reto"),
                ("Pronação","Gire a mão levantando o mindinho, mantendo o braço reto"),
            ],
        },
        // Dispositivo{
        //     nome:  "Novo Dispositivo",
        //     identificador: "novoDisp",
        //     movimentos:&[("Lá", "Gire para lá"), ("Cá", "Rode para cá")]
        // },
    ],
};
