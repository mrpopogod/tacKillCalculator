extern crate minidom;

use minidom::Element;
use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};
use std::env;
use std::fs::File;
use std::io::BufReader;

#[derive(Clone, Copy, PartialEq)]
enum Crit {
    Engine,
    Cockpit,
    Gyro,
    Ammo,
    Other,
}

#[derive(Clone, Copy, PartialEq)]
enum Location {
    Head,
    CenterTorso,
    LeftTorso,
    RightTorso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
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
    rl_crits: Vec<Crit>,
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

    // TODO: minidom needs an xmlns and ssw files don't have one
    // Swap to https://github.com/RazrFalcon/roxmltree instead (since we're only reading)

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
        match child.name().to_string().as_str() {
            "engine" => mech.engine = child.text(),
            "cockpit" => handle_cockpit(child, &mut mech),
            "gyro" => mech.gyro = child.text(),
            "baseloadout" => handle_loadout(child, &mut mech),
            // TODO: "loadout" - probably want to take as an option and match the right one
            _ => (),
        }
    }

    handle_cockpit_type(&mut mech);
    handle_engine_type(&mut mech);
    handle_gyro_type(&mut mech);
    populate_leg_crits(&mut mech);
    // TODO: for full correctness should check for crits not exceeding the legal amount in each limb.

    // TODO: hardened armor modification to crit result

    let mut deaths = 0;
    for _i in 0..1000000 {
        match two_d_six() {
            2..=7 => (),
            8..=9 => {
                let location_crits: Vec<Crit> = mech.ct_crits.to_vec();
                check_single_crit(mech.clan_case, location_crits, &mut deaths);
            }
            10..=11 => {
                let location_crits: Vec<Crit> = mech.ct_crits.to_vec();
                check_double_crit(mech.clan_case, location_crits, &mut deaths);
            }
            12 => {
                let location_crits: Vec<Crit> = mech.ct_crits.to_vec();
                check_triple_crit(mech.clan_case, location_crits, &mut deaths);
            }
            _ => panic!("2d6 somehow was not between 2 and 12"),
        }
    }

    let mut floating_deaths = 0;
    for _i in 0..1000000 {
        let location_crits = match two_d_six() {
            2 | 7 => mech.ct_crits.to_vec(),
            3 | 4 => mech.ra_crits.to_vec(),
            5 => mech.rl_crits.to_vec(),
            6 => mech.rt_crits.to_vec(),
            8 => mech.lt_crits.to_vec(),
            9 => mech.ll_crits.to_vec(),
            10 | 11 => mech.la_crits.to_vec(),
            12 => mech.head_crits.to_vec(),
            _ => panic!("2d6 somehow was not between 2 and 12"),
        };

        match two_d_six() {
            2..=7 => (),
            8..=9 => {
                check_single_crit(mech.clan_case, location_crits, &mut floating_deaths);
            }
            10..=11 => {
                check_double_crit(mech.clan_case, location_crits, &mut floating_deaths);
            }
            12 => {
                check_triple_crit(mech.clan_case, location_crits, &mut floating_deaths);
            }
            _ => panic!("2d6 somehow was not between 2 and 12"),
        }
    }

    let regular_percentage = deaths as f32 / 1000000.0 * 100.0;
    let floating_percentage = floating_deaths as f32 / 1000000.0 * 100.0;

    println!(
        "Regular rules had {} deaths in 1,000,000 runs, floating crits had {} deaths in 1,000,000 runs", 
        deaths, floating_deaths
    );
    println!(
        "Regular death percentage {} floating crit death percentage {}",
        regular_percentage, floating_percentage
    );
}

fn check_single_crit(clan_case: bool, location_crits: Vec<Crit>, deaths: &mut i32) {
    if location_crits.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    let chosen_crit = location_crits.choose(&mut rng).unwrap();
    if (*chosen_crit == Crit::Ammo && !clan_case) || *chosen_crit == Crit::Cockpit {
        *deaths += 1;
    }
}

fn check_double_crit(clan_case: bool, mut location_crits: Vec<Crit>, deaths: &mut i32) {
    if location_crits.len() < 2 {
        check_single_crit(clan_case, location_crits, deaths);
        return;
    }

    let mut rng = rand::thread_rng();
    let (i, first_crit) = location_crits
        .iter_mut()
        .enumerate()
        .choose(&mut rng)
        .unwrap();
    let first_crit = *first_crit;
    location_crits.remove(i);
    let (i, second_crit) = location_crits
        .iter_mut()
        .enumerate()
        .choose(&mut rng)
        .unwrap();
    let second_crit = *second_crit;
    location_crits.remove(i);

    if ((first_crit == Crit::Ammo || second_crit == Crit::Ammo) && !clan_case)
        || first_crit == Crit::Cockpit
        || second_crit == Crit::Cockpit
    {
        *deaths += 1;
    } else if first_crit == Crit::Gyro && second_crit == Crit::Gyro {
        *deaths += 1;
    }
}

fn check_triple_crit(clan_case: bool, mut location_crits: Vec<Crit>, deaths: &mut i32) {
    if location_crits.len() < 3 {
        check_double_crit(clan_case, location_crits, deaths);
        return;
    }

    let mut rng = rand::thread_rng();
    let (i, first_crit) = location_crits
        .iter_mut()
        .enumerate()
        .choose(&mut rng)
        .unwrap();
    let first_crit = *first_crit;
    location_crits.remove(i);
    let (i, second_crit) = location_crits
        .iter_mut()
        .enumerate()
        .choose(&mut rng)
        .unwrap();
    let second_crit = *second_crit;
    location_crits.remove(i);
    let (i, third_crit) = location_crits
        .iter_mut()
        .enumerate()
        .choose(&mut rng)
        .unwrap();
    let third_crit = *third_crit;
    location_crits.remove(i);

    // TODO: refactor
    // 1. after we remove a crit, check for head, kill if appropriate
    // 2. from that same crit, check for ammo, if ammo check for case, no case then kill
    // 3. if regular case check for gyro, cockpit, or enough engine crits for death
    // 4. if case ii roll another crit
    // 5. when remove engine/gyro add to a running tally and check at end (and reference as needed in step 3)
    if ((first_crit == Crit::Ammo || second_crit == Crit::Ammo || third_crit == Crit::Ammo)
        && !clan_case)
        || first_crit == Crit::Cockpit
        || second_crit == Crit::Cockpit
        || third_crit == Crit::Cockpit
    {
        *deaths += 1;
    } else if first_crit == Crit::Engine
        && second_crit == Crit::Engine
        && third_crit == Crit::Engine
    {
        *deaths += 1;
    } else if (first_crit == Crit::Gyro && second_crit == Crit::Gyro)
        || (first_crit == Crit::Gyro && third_crit == Crit::Gyro)
        || (second_crit == Crit::Gyro && third_crit == Crit::Gyro)
    {
        *deaths += 1;
    }
}

fn populate_leg_crits(mech: &mut Mech) {
    // TODO: handle quads
    // hip, upper, lower, foot
    mech.ll_crits.push(Crit::Other);
    mech.ll_crits.push(Crit::Other);
    mech.ll_crits.push(Crit::Other);
    mech.ll_crits.push(Crit::Other);
    mech.rl_crits.push(Crit::Other);
    mech.rl_crits.push(Crit::Other);
    mech.rl_crits.push(Crit::Other);
    mech.rl_crits.push(Crit::Other);
}

fn handle_cockpit_type(mech: &mut Mech) {
    match mech.cockpit.as_str() {
        "Standard Cockpit" => {
            mech.head_crits.push(Crit::Cockpit);
            mech.head_crits.push(Crit::Other);
            mech.head_crits.push(Crit::Other);
            mech.head_crits.push(Crit::Other);
            mech.head_crits.push(Crit::Other);
        }
        "Small Cockpit" => {
            mech.head_crits.push(Crit::Cockpit);
            mech.head_crits.push(Crit::Other);
            mech.head_crits.push(Crit::Other);
            mech.head_crits.push(Crit::Other);
        }
        "Torso-Mounted Cockpit" => {
            mech.ct_crits.push(Crit::Cockpit);
            mech.ct_crits.push(Crit::Other);
            mech.head_crits.push(Crit::Other);
            mech.head_crits.push(Crit::Other);
            mech.lt_crits.push(Crit::Other);
            mech.rt_crits.push(Crit::Other);
        }
        _ => panic!("Invalid cockpit type {}", mech.cockpit),
    }
}

fn handle_engine_type(mech: &mut Mech) {
    match mech.engine.as_str() {
        "Fusion Engine" => {
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
        }
        "Compact Fusion Engine" => {
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
        }
        "XL Engine" => {
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.lt_crits.push(Crit::Engine);
            mech.lt_crits.push(Crit::Engine);
            mech.lt_crits.push(Crit::Engine);
            mech.rt_crits.push(Crit::Engine);
            mech.rt_crits.push(Crit::Engine);
            mech.rt_crits.push(Crit::Engine);
        }
        "XXL Engine" => {
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.ct_crits.push(Crit::Engine);
            mech.lt_crits.push(Crit::Engine);
            mech.lt_crits.push(Crit::Engine);
            mech.lt_crits.push(Crit::Engine);
            mech.rt_crits.push(Crit::Engine);
            mech.rt_crits.push(Crit::Engine);
            mech.rt_crits.push(Crit::Engine);
            mech.lt_crits.push(Crit::Engine);
            mech.lt_crits.push(Crit::Engine);
            mech.lt_crits.push(Crit::Engine);
            mech.rt_crits.push(Crit::Engine);
            mech.rt_crits.push(Crit::Engine);
            mech.rt_crits.push(Crit::Engine);
        }
        _ => panic!("Invalid engine type {}", mech.engine),
    }
}

fn handle_gyro_type(mech: &mut Mech) {
    match mech.gyro.as_str() {
        "Standard Gyro" => {
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
        }
        "Heavy Duty Gyro" => {
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
        }
        "XL Gyro" => {
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
        }
        "Compact Gyro" => {
            mech.ct_crits.push(Crit::Gyro);
            mech.ct_crits.push(Crit::Gyro);
        }
        _ => panic!("Invalid gyro type {}", mech.gyro),
    }
}

fn handle_cockpit(child: &Element, mech: &mut Mech) {
    for cockpit_child in child.children() {
        if cockpit_child.name() == "type" {
            mech.cockpit = cockpit_child.text();
        }
    }
}

fn handle_loadout(child: &Element, mech: &mut Mech) {
    for loadout_child in child.children() {
        match loadout_child.name().to_string().as_str() {
            "clancase" => mech.clan_case = loadout_child.text() == "TRUE",
            "equipment" => handle_equipment(loadout_child, mech),
            "actuators" => handle_actuators(loadout_child, mech),
            "heatsinks" => {
                let mut hs_type = String::from("");
                let mut hs_locations: Vec<Location> = Vec::new();
                for heatsink_child in loadout_child.children() {
                    if heatsink_child.name() == "type" {
                        hs_type = heatsink_child.text();
                    } else if heatsink_child.name() == "location" {
                        match heatsink_child.text().as_str() {
                            "RT" => hs_locations.push(Location::RightTorso),
                            "LT" => hs_locations.push(Location::LeftTorso),
                            "RA" => hs_locations.push(Location::RightArm),
                            "LA" => hs_locations.push(Location::LeftArm),
                            "RL" => hs_locations.push(Location::RightLeg),
                            "LL" => hs_locations.push(Location::LeftLeg),
                            "CT" => hs_locations.push(Location::CenterTorso),
                            "HD" => hs_locations.push(Location::Head),
                            _ => panic!("Invalid heatsink location {}", heatsink_child.text()),
                        }
                    }
                }

                // TODO: IS vs Clan DHS
                for hs_location in hs_locations {
                    match hs_type.as_str() {
                        "Single Heat Sink" => add_crit_to_location(Crit::Other, hs_location, mech),
                        "Double Heat Sink" => {
                            add_crit_to_location(Crit::Other, hs_location, mech);
                            add_crit_to_location(Crit::Other, hs_location, mech);
                            add_crit_to_location(Crit::Other, hs_location, mech);
                        }
                        _ => panic!("Invalid heat sink type {}", hs_type),
                    }
                }
            }
            _ => (),
        }
    }
}

fn add_crit_to_location(crit: Crit, location: Location, mech: &mut Mech) {
    match location {
        Location::Head => mech.head_crits.push(crit),
        Location::CenterTorso => mech.ct_crits.push(crit),
        Location::LeftTorso => mech.lt_crits.push(crit),
        Location::RightTorso => mech.rt_crits.push(crit),
        Location::LeftArm => mech.la_crits.push(crit),
        Location::RightArm => mech.ra_crits.push(crit),
        Location::LeftLeg => mech.ll_crits.push(crit),
        Location::RightLeg => mech.rl_crits.push(crit),
    }
}

fn handle_actuators(loadout_child: &Element, mech: &mut Mech) {
    // Shoulder and upper arm
    mech.la_crits.push(Crit::Other);
    mech.la_crits.push(Crit::Other);
    mech.ra_crits.push(Crit::Other);
    mech.ra_crits.push(Crit::Other);

    if let Some("TRUE") = loadout_child.attr("lla") {
        mech.la_crits.push(Crit::Other);
    }
    if let Some("TRUE") = loadout_child.attr("lh") {
        mech.la_crits.push(Crit::Other);
    }
    if let Some("TRUE") = loadout_child.attr("rla") {
        mech.ra_crits.push(Crit::Other);
    }
    if let Some("TRUE") = loadout_child.attr("rh") {
        mech.ra_crits.push(Crit::Other);
    }
}

fn handle_equipment(loadout_child: &Element, mech: &mut Mech) {
    let mut location = Location::CenterTorso;
    let mut equipment_type = Crit::Other;
    for equipment_child in loadout_child.children() {
        extract_equipment_type_and_location(equipment_child, &mut location, &mut equipment_type);
    }

    // TODO: biggest miss here is that we don't know the number of crits per piece of gear since that is hardcoded in SSW's code
    add_crit_to_location(equipment_type, location, mech);
}

fn extract_equipment_type_and_location(
    equipment_child: &Element,
    location: &mut Location,
    equipment_type: &mut Crit,
) {
    if equipment_child.name() == "location" {
        match equipment_child.text().as_str() {
            "RT" => *location = Location::RightTorso,
            "LT" => *location = Location::LeftTorso,
            "RA" => *location = Location::RightArm,
            "LA" => *location = Location::LeftArm,
            "RL" => *location = Location::RightLeg,
            "LL" => *location = Location::LeftLeg,
            "CT" => *location = Location::CenterTorso,
            "HD" => *location = Location::Head,
            _ => panic!("Invalid equipment location {}", equipment_child.text()),
        }
    } else if equipment_child.name() == "type" {
        match equipment_child.text().as_str() {
            "ammunition" => *equipment_type = Crit::Ammo,
            _ => *equipment_type = Crit::Other,
        }
    } // TODO: any non-armor/structure equipment that ignores crits?
}

pub fn two_d_six() -> u8 {
    let mut rng = rand::thread_rng();
    let d1 = rng.gen_range(1..=6);
    let d2 = rng.gen_range(1..=6);

    d1 + d2
}
