use serde_cs2;
use serde_derive::{Deserialize, Serialize};


pub struct TrainFunction {
    pub id: u8,
    pub typ: u8,
    pub duration: i8,
    pub value: u8,
}

pub struct TrainDecoder {
    pub name: String,
    pub speed: u16,
    pub direction: u8,
    pub functions: Vec<TrainFunction>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "version")]
pub struct Version {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    major: Option<u8>,
    minor: u8,
}

impl Version {
    pub fn new (major: u8, minor: u8) -> Self {
        if major == 0 {
            Self { major: None, minor }
        } else {
            Self { major: Some(major), minor }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "[lokomotive]")]
pub struct LokstatFile {
    version: Version,
    lokomotive: Vec<Lokstat>,
}

impl Default for LokstatFile {
    fn default() -> Self {
        Self {
            version: Version::new(0, 3),
            lokomotive: vec![],
        }
    }
}

impl LokstatFile {
    pub fn add_train(&mut self, decoder: &TrainDecoder) {
        let lokomotive = Lokstat::from(decoder);
        self.lokomotive.push(lokomotive);
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "funktionen")]
pub struct FunktionStatus {
    nr: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    wert: Option<u8>,
}

impl FunktionStatus {
    pub fn get_id(&self) -> u8 {
        self.nr
    }

    pub fn get_value(&self) -> Option<u8> {
        self.wert
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "lokomotive")]
pub struct Lokstat {
    name: String,
    #[serde(default, rename = "velocity")]
    speed: u16,
    #[serde(default, rename = "richtung")]
    direction: u8,
    #[serde(default)]
    funktionen: Vec<FunktionStatus>,
}

impl Lokstat {
    pub fn get_direction(&self) -> u8 {
        self.direction
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_speed(&self) -> u16 {
        self.speed
    }

    pub fn iter_functions(&self) -> std::slice::Iter<'_, FunktionStatus> {
        self.funktionen.iter()
    }
}

impl From<&TrainDecoder> for Lokstat {
    fn from(decoder: &TrainDecoder) -> Self {
        let mut funktionen = vec![];
        for function in &decoder.functions {
            funktionen.push(FunktionStatus {
                nr: function.id,
                wert: Some(function.value),
            });
        }
        while funktionen.len() < 32 {
            funktionen.push(FunktionStatus {
                nr: funktionen.len() as u8,
                wert: None,
            });
        }
        Self {
            name: decoder.name.to_owned(),
            speed: decoder.speed,
            direction: decoder.direction,
            funktionen,
        }
    }
}

#[test]
fn lokstat() {

    let mut train = TrainDecoder {
        name: "01 133 DB".to_string(),
        speed: 255,
        direction: 1,
        functions: vec![],
    };
    train.functions.push(TrainFunction{ id: 0, duration: 1, typ: 1, value: 1 });

    let lokomotive = Lokstat::from(&train);
    let serialized = serde_cs2::to_string(&lokomotive).unwrap();
    println!("{}", serialized);

    let cs2_str = r#"lokomotive
 .name=01 133 DB
 .velocity=255
 .richtung=1
 .funktionen
 ..nr=0
 ..wert=1
 .funktionen
 ..nr=1
 .funktionen
 ..nr=2
 .funktionen
 ..nr=3
 .funktionen
 ..nr=4
 .funktionen
 ..nr=5
 .funktionen
 ..nr=6
 .funktionen
 ..nr=7
 .funktionen
 ..nr=8
 .funktionen
 ..nr=9
 .funktionen
 ..nr=10
 .funktionen
 ..nr=11
 .funktionen
 ..nr=12
 .funktionen
 ..nr=13
 .funktionen
 ..nr=14
 .funktionen
 ..nr=15
 .funktionen
 ..nr=16
 .funktionen
 ..nr=17
 .funktionen
 ..nr=18
 .funktionen
 ..nr=19
 .funktionen
 ..nr=20
 .funktionen
 ..nr=21
 .funktionen
 ..nr=22
 .funktionen
 ..nr=23
 .funktionen
 ..nr=24
 .funktionen
 ..nr=25
 .funktionen
 ..nr=26
 .funktionen
 ..nr=27
 .funktionen
 ..nr=28
 .funktionen
 ..nr=29
 .funktionen
 ..nr=30
 .funktionen
 ..nr=31"#;

    assert_eq!(serialized, cs2_str);

    let lokstat: Lokstat = serde_cs2::from_str(serialized.as_str()).unwrap();
    assert_eq!(lokstat.get_direction(), 1);
    assert_eq!(lokstat.get_speed(), 255);

    for (index, function) in lokstat.iter_functions().enumerate() {
        assert_eq!(function.get_id(), index as u8);
        if index == 0 {
            assert_eq!(function.get_value(), Some(1));
        } else {
            assert_eq!(function.get_value(), None);
        }
    }
}

#[test]
fn lokstat_file_simple() {
    let mut train = TrainDecoder {
        name: "01 133 DB".to_string(),
        speed: 255,
        direction: 1,
        functions: vec![],
    };
    train.functions.push(TrainFunction{ id: 0, duration: 1, typ: 1, value: 1 });

    let mut lokstat = LokstatFile::default();
    lokstat.add_train(&train);
    let serialized = serde_cs2::to_string(&lokstat).unwrap();

    let cs2_str = r#"[lokomotive]
version
 .minor=3
lokomotive
 .name=01 133 DB
 .velocity=255
 .richtung=1
 .funktionen
 ..nr=0
 ..wert=1
 .funktionen
 ..nr=1
 .funktionen
 ..nr=2
 .funktionen
 ..nr=3
 .funktionen
 ..nr=4
 .funktionen
 ..nr=5
 .funktionen
 ..nr=6
 .funktionen
 ..nr=7
 .funktionen
 ..nr=8
 .funktionen
 ..nr=9
 .funktionen
 ..nr=10
 .funktionen
 ..nr=11
 .funktionen
 ..nr=12
 .funktionen
 ..nr=13
 .funktionen
 ..nr=14
 .funktionen
 ..nr=15
 .funktionen
 ..nr=16
 .funktionen
 ..nr=17
 .funktionen
 ..nr=18
 .funktionen
 ..nr=19
 .funktionen
 ..nr=20
 .funktionen
 ..nr=21
 .funktionen
 ..nr=22
 .funktionen
 ..nr=23
 .funktionen
 ..nr=24
 .funktionen
 ..nr=25
 .funktionen
 ..nr=26
 .funktionen
 ..nr=27
 .funktionen
 ..nr=28
 .funktionen
 ..nr=29
 .funktionen
 ..nr=30
 .funktionen
 ..nr=31"#;

    assert_eq!(serialized, cs2_str);

    let lokstat_file: LokstatFile = serde_cs2::from_str(serialized.as_str()).unwrap();
    assert_eq!(lokstat_file.lokomotive.len(), 1);

    let lokstat = lokstat_file.lokomotive.first().unwrap();
    assert_eq!(lokstat.get_direction(), 1);
    assert_eq!(lokstat.get_speed(), 255);
}

#[test]
fn lokstat_file_full() {
    let mut lokstat = LokstatFile::default();

    let mut train = TrainDecoder {
        name: "01 133 DB".to_string(),
        speed: 255,
        direction: 1,
        functions: vec![],
    };
    train.functions.push(TrainFunction{ id: 0, duration: 1, typ: 1, value: 1 });
    lokstat.add_train(&train);

    let mut train = TrainDecoder {
        name: "02".to_string(),
        speed: 200,
        direction: 0,
        functions: vec![],
    };
    train.functions.push(TrainFunction{ id: 0, duration: 1, typ: 1, value: 1 });
    lokstat.add_train(&train);

    let serialized = serde_cs2::to_string(&lokstat).unwrap();

    let cs2_str = r#"[lokomotive]
version
 .minor=3
lokomotive
 .name=01 133 DB
 .velocity=255
 .richtung=1
 .funktionen
 ..nr=0
 ..wert=1
 .funktionen
 ..nr=1
 .funktionen
 ..nr=2
 .funktionen
 ..nr=3
 .funktionen
 ..nr=4
 .funktionen
 ..nr=5
 .funktionen
 ..nr=6
 .funktionen
 ..nr=7
 .funktionen
 ..nr=8
 .funktionen
 ..nr=9
 .funktionen
 ..nr=10
 .funktionen
 ..nr=11
 .funktionen
 ..nr=12
 .funktionen
 ..nr=13
 .funktionen
 ..nr=14
 .funktionen
 ..nr=15
 .funktionen
 ..nr=16
 .funktionen
 ..nr=17
 .funktionen
 ..nr=18
 .funktionen
 ..nr=19
 .funktionen
 ..nr=20
 .funktionen
 ..nr=21
 .funktionen
 ..nr=22
 .funktionen
 ..nr=23
 .funktionen
 ..nr=24
 .funktionen
 ..nr=25
 .funktionen
 ..nr=26
 .funktionen
 ..nr=27
 .funktionen
 ..nr=28
 .funktionen
 ..nr=29
 .funktionen
 ..nr=30
 .funktionen
 ..nr=31
lokomotive
 .name=02
 .velocity=200
 .richtung=0
 .funktionen
 ..nr=0
 ..wert=1
 .funktionen
 ..nr=1
 .funktionen
 ..nr=2
 .funktionen
 ..nr=3
 .funktionen
 ..nr=4
 .funktionen
 ..nr=5
 .funktionen
 ..nr=6
 .funktionen
 ..nr=7
 .funktionen
 ..nr=8
 .funktionen
 ..nr=9
 .funktionen
 ..nr=10
 .funktionen
 ..nr=11
 .funktionen
 ..nr=12
 .funktionen
 ..nr=13
 .funktionen
 ..nr=14
 .funktionen
 ..nr=15
 .funktionen
 ..nr=16
 .funktionen
 ..nr=17
 .funktionen
 ..nr=18
 .funktionen
 ..nr=19
 .funktionen
 ..nr=20
 .funktionen
 ..nr=21
 .funktionen
 ..nr=22
 .funktionen
 ..nr=23
 .funktionen
 ..nr=24
 .funktionen
 ..nr=25
 .funktionen
 ..nr=26
 .funktionen
 ..nr=27
 .funktionen
 ..nr=28
 .funktionen
 ..nr=29
 .funktionen
 ..nr=30
 .funktionen
 ..nr=31"#;

    assert_eq!(serialized, cs2_str);

    let lokstat_file: LokstatFile = serde_cs2::from_str(serialized.as_str()).unwrap();
    assert_eq!(lokstat_file.lokomotive.len(), 2);

    let lokstat = lokstat_file.lokomotive.first().unwrap();
    assert_eq!(lokstat.get_direction(), 1);
    assert_eq!(lokstat.get_speed(), 255);

    let lokstat = lokstat_file.lokomotive.last().unwrap();
    assert_eq!(lokstat.get_direction(), 0);
    assert_eq!(lokstat.get_speed(), 200);
}
