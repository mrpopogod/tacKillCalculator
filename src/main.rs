extern crate minidom;

use std::env;
use std::fs::File;
use std::io::BufReader;
use minidom::Element;

enum Crit {
    Engine,
    Cockpit,
    Gyro,
    Ammo,
    Other
}

enum Location {
    Head,
    CenterTorso,
    LeftTorso,
    RightTorso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg
}

struct Mech {
    engine: String,
    cockpit: String,
    gyro: String,
    clan_case: bool,
    head_crits: Vec<Crit>,
    ct_crits: Vec<Crit>,
    lt_crits: Vec<Crit>,
    rt_crits: Vec<Crit>,
    la_crits: Vec<Crit>,
    ra_crits: Vec<Crit>,
    ll_crits: Vec<Crit>,
    rl_crits: Vec<Crit>
}

fn main() {
    let mut filename = String::from("");
    let mut found = false;
    for argument in env::args() {
        if argument == "--file" {
            found = true;
        } else if found {
            filename = argument;
        }
    }

    if filename.is_empty() {
        panic!("No --file found");
    }

    let file = File::open(filename).unwrap();
    let file = BufReader::new(file);
    let mech_element = Element::from_reader(file).unwrap();

    let mut mech = Mech {
        engine: String::from("Fusion Engine"),
        cockpit: String::from("Standard Cockpit"),
        gyro: String::from("Standard Gyro"),
        clan_case: false,
        head_crits: Vec::new(),
        ct_crits: Vec::new(),
        lt_crits: Vec::new(),
        rt_crits: Vec::new(),
        la_crits: Vec::new(),
        ra_crits: Vec::new(),
        ll_crits: Vec::new(),
        rl_crits: Vec::new(),
    };

    for child in mech_element.children() {
        if child.name() == "engine" {
            mech.engine = String::from(child.text());
        } else if child.name() == "cockpit" {
            for cockpit_child in child.children() {
                if cockpit_child.name() == "type" {
                    mech.cockpit = String::from(child.text());
                }
            }
        } else if child.name() == "gyro" {
            mech.gyro = String::from(child.text());
        } else if child.name() == "baseloadout" {
            for loadout_child in child.children() {
                if loadout_child.name() == "clancase" {
                    mech.clan_case = (loadout_child.text() == "TRUE");
                } else if loadout_child.name() == "equipment" {
                    let mut location = Location::CenterTorso;
                    let mut equipment_type = Crit::Other;
                    for equipment_child in loadout_child.children() {
                        if equipment_child.name() == "location" {
                            match equipment_child.text().as_str() {
                                "RT" => location = Location::RightTorso,
                                "LT" => location = Location::LeftTorso,
                                "RA" => location = Location::RightArm,
                                "LA" => location = Location::LeftArm,
                                "RL" => location = Location::RightLeg,
                                "LL" => location = Location::LeftLeg,
                                "CT" => location = Location::CenterTorso,
                                "HD" => location = Location::Head,
                                _ => panic!("Invalid equipment location")
                            }
                        } else if equipment_child.name() == "type" {
                            match equipment_child.text().as_str() {
                                "ammunition" => equipment_type = Crit::Ammo,
                                _ => equipment_type = Crit::Other
                            }
                        }
                    }

                    match location {
                        Location::Head => mech.head_crits.push(equipment_type),
                        Location::CenterTorso => mech.ct_crits.push(equipment_type),
                        Location::LeftTorso => mech.lt_crits.push(equipment_type),
                        Location::RightTorso => mech.rt_crits.push(equipment_type),
                        Location::LeftArm => mech.la_crits.push(equipment_type),
                        Location::RightArm => mech.ra_crits.push(equipment_type),
                        Location::LeftLeg => mech.ll_crits.push(equipment_type),
                        Location::RightLeg => mech.rl_crits.push(equipment_type),
                    }
                }
            }

            // TODO: more matches
            // TODO: refactor elseifs into matches and sub fuctions for the internals
            // TODO: actually run the chance to die simulations
        }
    }
}
