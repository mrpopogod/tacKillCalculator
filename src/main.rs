use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};
use std::io::BufReader;
use std::{env, io::BufRead};
use std::{fs::File, ops::ControlFlow};
use std::cmp;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Crit {
    Engine,
    Cockpit,
    Gyro,
    Ammo,
    CASE,
    CASEII,
    Other,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Config {
    Biped,
    Quad,
    Tripod,
}

#[derive(Debug)]
struct Mech {
    clan_case: bool,
    hardened: bool,
    heavy_duty_gyro: bool,
    config: Config,
    head_crits: Vec<Crit>,
    ct_crits: Vec<Crit>,
    lt_crits: Vec<Crit>,
    rt_crits: Vec<Crit>,
    la_crits: Vec<Crit>,
    ra_crits: Vec<Crit>,
    ll_crits: Vec<Crit>,
    rl_crits: Vec<Crit>,
    cl_crits: Vec<Crit>,
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
    } else if !filename.ends_with(".mtf") {
        panic!("Only MTF files are supported");
    }

    let file = File::open(filename).unwrap();
    let file = BufReader::new(file);

    let mut mech = Mech {
        clan_case: false,
        hardened: false,
        heavy_duty_gyro: false,
        config: Config::Biped,
        head_crits: Vec::new(),
        ct_crits: Vec::new(),
        lt_crits: Vec::new(),
        rt_crits: Vec::new(),
        la_crits: Vec::new(),
        ra_crits: Vec::new(),
        ll_crits: Vec::new(),
        rl_crits: Vec::new(),
        cl_crits: Vec::new(),
    };

    let mut lines = file.lines();
    while let Some(line) = lines.next() {
        let line = line.unwrap();
        if line.starts_with("Config") {
            if line.contains("Quad") {
                mech.config = Config::Quad;
            } else if line.contains("Tripod") {
                mech.config = Config::Tripod;
            }
        }

        if line.starts_with("TechBase") && line.contains("Clan") {
            mech.clan_case = true;
        }

        if line.starts_with("Armor") && line.contains("Hardened") {
            mech.hardened = true;
        }

        if line.starts_with("Gyro") && line.contains("Heavy Duty") {
            mech.heavy_duty_gyro = true;
        }

        if line.starts_with("Left Arm") || line.starts_with("Left Front Leg") {
            while let Some(line) = lines.next() {
                let line = line.unwrap();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.la_crits) {
                    break;
                }
            }
        }

        if line.starts_with("Right Arm") || line.starts_with("Right Front Leg") {
            while let Some(line) = lines.next() {
                let line = line.unwrap();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.ra_crits) {
                    break;
                }
            }
        }

        if line.starts_with("Left Leg") || line.starts_with("Left Rear Leg") {
            while let Some(line) = lines.next() {
                let line = line.unwrap();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.ll_crits) {
                    break;
                }
            }
        }

        if line.starts_with("Right Leg") || line.starts_with("Right Rear Leg") {
            while let Some(line) = lines.next() {
                let line = line.unwrap();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.rl_crits) {
                    break;
                }
            }
        }

        if line.starts_with("Center Leg") {
            while let Some(line) = lines.next() {
                let line = line.unwrap();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.cl_crits) {
                    break;
                }
            }
        }

        if line.starts_with("Left Torso") {
            while let Some(line) = lines.next() {
                let line = line.unwrap();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.lt_crits) {
                    break;
                }
            }
        }

        if line.starts_with("Right Torso") {
            while let Some(line) = lines.next() {
                let line = line.unwrap();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.rt_crits) {
                    break;
                }
            }
        }

        if line.starts_with("Center Torso") {
            while let Some(line) = lines.next() {
                let line = line.unwrap();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.ct_crits) {
                    break;
                }
            }
        }

        if line.starts_with("Head") {
            while let Some(line) = lines.next() {
                let line = line.unwrap();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.head_crits) {
                    break;
                }
            }
        }
    }

    // TODO: for full correctness should check for crits not exceeding the legal amount in each location.

    let mut deaths = 0;
    for _i in 0..1000000 {
        let mut result = two_d_six();
        if mech.hardened {
            result = cmp::max(result - 2, 2);
        }

        let location_crits: Vec<Crit> = mech.ct_crits.to_vec();
        check_crit_roll(result, mech.clan_case, location_crits, &mut deaths);
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

        let mut result = two_d_six();
        if mech.hardened {
            result = cmp::max(result - 2, 2);
        }

        check_crit_roll(result, mech.clan_case, location_crits, &mut floating_deaths);
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

fn check_crit_roll(result: u8, clan_case: bool, location_crits: Vec<Crit>, deaths: &mut i32) {
    match result {
        2..=7 => (),
        8..=9 => {
            check_single_crit(clan_case, location_crits, deaths);
        }
        10..=11 => {
            check_double_crit(clan_case, location_crits, deaths);
        }
        12 => {
            check_triple_crit(clan_case, location_crits, deaths);
        }
        _ => panic!("2d6 somehow was not between 2 and 12"),
    }
}

fn parse_crits(line: String, crits: &mut Vec<Crit>) -> ControlFlow<()> {
    match line.as_str() {
        "-Empty-" => (),
        "" => return ControlFlow::Break(()),
        _ => {
            if line.contains("Ammo") {
                crits.push(Crit::Ammo);
            } else if line.contains("Cockpit") {
                crits.push(Crit::Cockpit);
            } else if line.contains("Engine") {
                crits.push(Crit::Engine);
            } else if line.contains("Gyro") {
                crits.push(Crit::Gyro);
            } else if line.contains("CASEII") {
                crits.push(Crit::CASEII);
            } else if line.contains("CASE") {
                crits.push(Crit::CASE);
            } else {
                crits.push(Crit::Other);
            }
        }
    }
    ControlFlow::Continue(())
}

fn check_single_crit(clan_case: bool, location_crits: Vec<Crit>, deaths: &mut i32) {
    if location_crits.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    let chosen_crit = location_crits.choose(&mut rng).unwrap();
    let chosen_crit = *chosen_crit;
    if chosen_crit == Crit::Cockpit {
        *deaths += 1;
        return;
    }

    if chosen_crit == Crit::Ammo && !clan_case {
        if location_crits.contains(&Crit::CASEII) {
            let result = two_d_six();
            check_crit_roll(result, clan_case, location_crits, deaths);
            return;
        } else if location_crits.contains(&Crit::CASE) {
            let mut engines = 0;
            let mut gyros = 0;
            for crit in location_crits.as_slice() {
                match crit {
                    Crit::Engine => engines += 1,
                    Crit::Gyro => gyros += 1,
                    _ => (),
                }
            }

            if engines >= 3 || gyros >= 2 {
                *deaths += 1;
                return;
            }
        } else {
            *deaths += 1;
            return;
        }
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

    if first_crit == Crit::Cockpit {
        *deaths += 1;
        return;
    }

    if first_crit == Crit::Ammo && !clan_case {
        if location_crits.contains(&Crit::CASEII) {
            let result = two_d_six();
            check_crit_roll(result, clan_case, location_crits, deaths);
            return;
        } else if location_crits.contains(&Crit::CASE) {
            let mut engines = 0;
            let mut gyros = 0;
            for crit in location_crits.as_slice() {
                match crit {
                    Crit::Engine => engines += 1,
                    Crit::Gyro => gyros += 1,
                    _ => (),
                }
            }

            if engines >= 3 || gyros >= 2 {
                *deaths += 1;
                return;
            }
        } else {
            *deaths += 1;
            return;
        }
    }

    let mut gyros = 0;
    let mut engines = 0;
    match first_crit {
        Crit::Engine => engines += 1,
        Crit::Gyro => gyros += 1,
        _ => (),
    }

    let (i, second_crit) = location_crits
        .iter_mut()
        .enumerate()
        .choose(&mut rng)
        .unwrap();
    let second_crit = *second_crit;
    location_crits.remove(i);

    if second_crit == Crit::Cockpit {
        *deaths += 1;
        return;
    }

    if second_crit == Crit::Ammo && !clan_case {
        if location_crits.contains(&Crit::CASEII) {
            let result = two_d_six();
            check_crit_roll(result, clan_case, location_crits, deaths);
            return;
        } else if location_crits.contains(&Crit::CASE) {
            let mut engines = 0;
            let mut gyros = 0;
            for crit in location_crits.as_slice() {
                match crit {
                    Crit::Engine => engines += 1,
                    Crit::Gyro => gyros += 1,
                    _ => (),
                }
            }

            if engines >= 3 || gyros >= 2 {
                *deaths += 1;
                return;
            }
        } else {
            *deaths += 1;
            return;
        }
    }

    match second_crit {
        Crit::Engine => engines += 1,
        Crit::Gyro => gyros += 1,
        _ => (),
    }

    if engines >= 3 || gyros >= 2 {
        *deaths += 1;
        return;
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

    if first_crit == Crit::Cockpit {
        *deaths += 1;
        return;
    }

    // TODO: Figure out how to refactor this block while following the rust borrowing rules
    if first_crit == Crit::Ammo && !clan_case {
        if location_crits.contains(&Crit::CASEII) {
            let result = two_d_six();
            check_crit_roll(result, clan_case, location_crits, deaths);
            return;
        } else if location_crits.contains(&Crit::CASE) {
            let mut engines = 0;
            let mut gyros = 0;
            for crit in location_crits.as_slice() {
                match crit {
                    Crit::Engine => engines += 1,
                    Crit::Gyro => gyros += 1,
                    _ => (),
                }
            }

            if engines >= 3 || gyros >= 2 {
                *deaths += 1;
                return;
            }
        } else {
            // TODO: need to account for torso CASE somehow if this is an arm or leg
            *deaths += 1;
            return;
        }
    }

    let mut gyros = 0;
    let mut engines = 0;
    match first_crit {
        Crit::Engine => engines += 1,
        Crit::Gyro => gyros += 1,
        _ => (),
    }

    let (i, second_crit) = location_crits
        .iter_mut()
        .enumerate()
        .choose(&mut rng)
        .unwrap();
    let second_crit = *second_crit;
    location_crits.remove(i);

    if second_crit == Crit::Cockpit {
        *deaths += 1;
        return;
    }

    if second_crit == Crit::Ammo && !clan_case {
        if location_crits.contains(&Crit::CASEII) {
            let result = two_d_six();
            check_crit_roll(result, clan_case, location_crits, deaths);
            return;
        } else if location_crits.contains(&Crit::CASE) {
            let mut engines = 0;
            let mut gyros = 0;
            for crit in location_crits.as_slice() {
                match crit {
                    Crit::Engine => engines += 1,
                    Crit::Gyro => gyros += 1,
                    _ => (),
                }
            }

            if engines >= 3 || gyros >= 2 {
                *deaths += 1;
                return;
            }
        } else {
            *deaths += 1;
            return;
        }
    }

    match second_crit {
        Crit::Engine => engines += 1,
        Crit::Gyro => gyros += 1,
        _ => (),
    }

    let (i, third_crit) = location_crits
        .iter_mut()
        .enumerate()
        .choose(&mut rng)
        .unwrap();
    let third_crit = *third_crit;
    location_crits.remove(i);

    if third_crit == Crit::Cockpit {
        *deaths += 1;
        return;
    }

    if third_crit == Crit::Ammo && !clan_case {
        if location_crits.contains(&Crit::CASEII) {
            let result = two_d_six();
            check_crit_roll(result, clan_case, location_crits, deaths);
            return;
        } else if location_crits.contains(&Crit::CASE) {
            let mut engines = 0;
            let mut gyros = 0;
            for crit in location_crits.as_slice() {
                match crit {
                    Crit::Engine => engines += 1,
                    Crit::Gyro => gyros += 1,
                    _ => (),
                }
            }

            if engines >= 3 || gyros >= 2 {
                *deaths += 1;
                return;
            }
        } else {
            *deaths += 1;
            return;
        }
    }

    match third_crit {
        Crit::Engine => engines += 1,
        Crit::Gyro => gyros += 1,
        _ => (),
    }

    if engines >= 3 || gyros >= 2 {
        *deaths += 1;
        return;
    }
}

pub fn two_d_six() -> u8 {
    let mut rng = rand::thread_rng();
    let d1 = rng.gen_range(1..=6);
    let d2 = rng.gen_range(1..=6);

    d1 + d2
}
