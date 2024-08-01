use serde_cs2;
use serde_derive::{Deserialize, Serialize};
use serde_hex::{SerHex, SerHexOpt, StrictPfx, CompactPfx};


#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "funktionen_2")]
pub struct Funktionen2 {
    pub nr: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typ: Option<u16>,
    pub dauer: i8,
    pub wert: u8,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "funktionen")]
pub struct Funktionen {
    pub nr: u8,
    pub typ: u16,
    pub dauer: i8,
    pub wert: u8,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "lokomotive")]
struct Lokomotive {
    name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    vorname: String,
    #[serde(with = "SerHex::<StrictPfx>")]
    uid: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "SerHexOpt::<StrictPfx>")]
    mfxuid: Option<u32>,
    #[serde(with = "SerHex::<CompactPfx>")]
    adresse: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    funktionen: Vec<Funktionen>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    funktionen_2: Vec<Funktionen2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blocks: Option<[u8; 16]>
}

#[test]
fn lokomotive_serialize() {

    let lokomotive = Lokomotive {
        name: "Lok".to_owned(),
        vorname: "".to_owned(),
        uid: 0x4001,
        mfxuid: None,
        adresse: 5,
        funktionen: vec![],
        funktionen_2: vec![],
        blocks: None,
    };

    let str = r#"lokomotive
 .name=Lok
 .uid=0x4001
 .adresse=0x5
"#;

    let serialized = serde_cs2::to_string(&lokomotive).unwrap();
    assert_eq!(str, serialized);

    let lokomotive = Lokomotive {
        name: "Lok".to_owned(),
        vorname: "".to_owned(),
        uid: 0x4001,
        mfxuid: None,
        adresse: 5,
        funktionen: [
            Funktionen {
                nr: 1,
                typ: 1,
                dauer: -1,
                wert: 0,
            },
            Funktionen {
                nr: 2,
                typ: 2,
                dauer: 0,
                wert: 0,
            },
        ].to_vec(),
        funktionen_2: [
            Funktionen2 {
                nr: 16,
                typ: Some(16),
                dauer: 0,
                wert: 0,
            },
            Funktionen2 {
                nr: 17,
                typ: Some(17),
                dauer: 0,
                wert: 0,
            },
        ].to_vec(),
        blocks: Some([0; 16]),
    };

    let serialized = serde_cs2::to_string(&lokomotive).unwrap();

    let str = r#"lokomotive
 .name=Lok
 .uid=0x4001
 .adresse=0x5
 .funktionen
 ..nr=1
 ..typ=1
 ..dauer=-1
 ..wert=0
 .funktionen
 ..nr=2
 ..typ=2
 ..dauer=0
 ..wert=0
 .funktionen_2
 ..nr=16
 ..typ=16
 ..dauer=0
 ..wert=0
 .funktionen_2
 ..nr=17
 ..typ=17
 ..dauer=0
 ..wert=0
 .blocks=0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
"#;

    assert_eq!(str, serialized);
}


#[test]
fn lokomotive_deserialize() {

    let expected = Lokomotive {
        name: "Lok".to_owned(),
        vorname: "".to_owned(),
        uid: 0x4001,
        mfxuid: None,
        adresse: 5,
        funktionen: vec![],
        funktionen_2: vec![],
        blocks: None,
    };

    let cs2 = r#"lokomotive
     .name=Lok
     .uid=0x4001
     .adresse=0x5"#;
    assert_eq!(expected, serde_cs2::from_str(cs2).unwrap());

    let cs2 = r#"
    lokomotive
     .name=Lok
     .uid=0x4001
     .adresse=0x5"#;
    assert_eq!(expected, serde_cs2::from_str(cs2).unwrap());

    let cs2 = r#"
    lokomotive
     .name=Lok
     .uid=0x4001
     .adresse=0x5
    "#;
    assert_eq!(expected, serde_cs2::from_str(cs2).unwrap());

    let expected = Lokomotive {
        name: "Lok".to_owned(),
        vorname: "".to_owned(),
        uid: 0x4001,
        mfxuid: Some(0xffcd995d),
        adresse: 5,
        funktionen: [
            Funktionen {
                nr: 1,
                typ: 1,
                dauer: -1,
                wert: 0,
            },
            Funktionen {
                nr: 2,
                typ: 2,
                dauer: 0,
                wert: 0,
            },
        ].to_vec(),
        funktionen_2: [
            Funktionen2 {
                nr: 16,
                typ: Some(16),
                dauer: 0,
                wert: 0,
            },
            Funktionen2 {
                nr: 17,
                typ: Some(17),
                dauer: 0,
                wert: 0,
            },
        ].to_vec(),
        blocks: Some([0; 16]),
    };

    let cs2 = r#"
        lokomotive
         .name=Lok
         .uid=0x4001
         .mfxuid=0xffcd995d
         .adresse=0x5
         .funktionen
         ..nr=1
         ..typ=1
         ..dauer=-1
         ..wert=0
         .funktionen
         ..nr=2
         ..typ=2
         ..dauer=0
         ..wert=0
         .funktionen_2
         ..nr=16
         ..typ=16
         ..dauer=0
         ..wert=0
         .funktionen_2
         ..nr=17
         ..typ=17
         ..dauer=0
         ..wert=0
         .blocks=0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
        "#;

    assert_eq!(expected, serde_cs2::from_str(cs2).unwrap());

    assert_eq!(expected, serde_cs2::from_str(&serde_cs2::to_string(&expected).unwrap()).unwrap());
}
