use rand::seq::IteratorRandom;
use rand::Rng;
use std::cmp;
use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::ops::ControlFlow;
use std::panic;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Crit {
    Engine,
    Cockpit,
    Gyro,
    Ammo,
    Explosive(i32),
    Case,
    CaseII,
    Other,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Config {
    Biped,
    Quad,
    Tripod,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Location {
    Head,
    CenterTorso,
    LeftTorso,
    RightTorso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
    CenterLeg,
}

#[derive(Clone, Debug)]
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
    panic::set_hook(Box::new(|info| {
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            println!("{}", s);
        } else {
            println!("Unknown error occurred");
        }
    }));

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
        panic!("No --file specified");
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
        let line = line.unwrap().to_lowercase();
        if line.starts_with("config") {
            if line.contains("quad") {
                mech.config = Config::Quad;
            } else if line.contains("tripod") {
                mech.config = Config::Tripod;
            }
        }

        if line.starts_with("techbase") && line.contains("clan") {
            mech.clan_case = true;
        }

        if line.starts_with("armor") && line.contains("hardened") {
            mech.hardened = true;
        }

        if line.starts_with("gyro") && line.contains("heavy duty") {
            mech.heavy_duty_gyro = true;
        }

        if line.starts_with("left arm") || line.starts_with("front left leg") {
            for line in lines.by_ref() {
                let line = line.unwrap().to_lowercase();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.la_crits) {
                    break;
                }
            }
        }

        if line.starts_with("right arm") || line.starts_with("front right leg") {
            for line in lines.by_ref() {
                let line = line.unwrap().to_lowercase();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.ra_crits) {
                    break;
                }
            }
        }

        if line.starts_with("left leg") || line.starts_with("rear left leg") {
            for line in lines.by_ref() {
                let line = line.unwrap().to_lowercase();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.ll_crits) {
                    break;
                }
            }
        }

        if line.starts_with("right leg") || line.starts_with("rear right leg") {
            for line in lines.by_ref() {
                let line = line.unwrap().to_lowercase();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.rl_crits) {
                    break;
                }
            }
        }

        if line.starts_with("center leg") {
            for line in lines.by_ref() {
                let line = line.unwrap().to_lowercase();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.cl_crits) {
                    break;
                }
            }
        }

        if line.starts_with("left torso") {
            for line in lines.by_ref() {
                let line = line.unwrap().to_lowercase();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.lt_crits) {
                    break;
                }
            }
        }

        if line.starts_with("right torso") {
            for line in lines.by_ref() {
                let line = line.unwrap().to_lowercase();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.rt_crits) {
                    break;
                }
            }
        }

        if line.starts_with("center torso") {
            for line in lines.by_ref() {
                let line = line.unwrap().to_lowercase();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.ct_crits) {
                    break;
                }
            }
        }

        if line.starts_with("head") {
            for line in lines.by_ref() {
                let line = line.unwrap().to_lowercase();
                if let ControlFlow::Break(_) = parse_crits(line, &mut mech.head_crits) {
                    break;
                }
            }
        }
    }

    assert!(mech.head_crits.len() <= 6, "Too many head crits");
    assert!(mech.ct_crits.len() <= 12, "Too many center torso crits");
    assert!(mech.lt_crits.len() <= 12, "Too many left torso crits");
    assert!(mech.rt_crits.len() <= 12, "Too many right torso crits");
    assert!(mech.ll_crits.len() <= 6, "Too many left leg crits");
    assert!(mech.rl_crits.len() <= 6, "Too many right leg crits");
    match mech.config {
        Config::Biped => {
            assert!(mech.ra_crits.len() <= 12, "Too many right arm crits");
            assert!(mech.la_crits.len() <= 12, "Too many left arm crits");
            assert!(
                mech.cl_crits.len() == 0,
                "Biped mechs cannot have center leg crits"
            );
        }
        Config::Quad => {
            assert!(mech.ra_crits.len() <= 6, "Too many forward right leg crits");
            assert!(mech.la_crits.len() <= 6, "Too many forward left leg crits");
            assert!(
                mech.cl_crits.len() == 0,
                "Quad mechs cannot have center leg crits"
            );
        }
        Config::Tripod => {
            assert!(mech.ra_crits.len() <= 12, "Too many right arm crits");
            assert!(mech.la_crits.len() <= 12, "Too many left arm crits");
            assert!(mech.cl_crits.len() <= 6, "Too many center leg crits");
        }
    }

    let mut rng = rand::thread_rng();
    let required_gyro_hits = if mech.heavy_duty_gyro { 3 } else { 2 };
    let mut deaths = 0;
    'iteration: for _i in 0..1_000_000 {
        let mut trialmech = mech.clone();
        let mut numcrits = roll_num_crits(trialmech.hardened);
        let mut engine_hits = 0;
        let mut gyro_hits = 0;
        while numcrits > 0 {
            let crits = &mut trialmech.ct_crits;

            let (i, chosen_crit) = crits.iter_mut().enumerate().choose(&mut rng).unwrap();
            let chosen_crit = *chosen_crit;
            crits.remove(i);

            match chosen_crit {
                Crit::Engine => engine_hits += 1,
                Crit::Cockpit => {
                    deaths += 1;
                    continue 'iteration;
                }
                Crit::Gyro => gyro_hits += 1,
                Crit::Ammo => {
                    if crits.contains(&Crit::CaseII) {
                        // TODO: track the one poitn of damage for when we handle explosive equipment
                        numcrits += roll_num_crits(false); // hardened doesn't apply for internal damage
                    } else if crits.contains(&Crit::Case) {
                        engine_hits += crits.iter().filter(|&c| *c == Crit::Engine).count();
                        gyro_hits += crits.iter().filter(|&c| *c == Crit::Gyro).count();
                        if engine_hits >= 3 || gyro_hits >= required_gyro_hits {
                            deaths += 1;
                        }

                        continue 'iteration;
                    } else {
                        deaths += 1;
                        continue 'iteration;
                    }
                }
                Crit::Explosive(_) => todo!(),
                _ => (),
            }

            numcrits -= 1;
        }

        if engine_hits >= 3 || gyro_hits >= required_gyro_hits {
            deaths += 1;
        }
    }

    let mut floating_deaths = 0;
    'iteration: for _i in 0..1_000_000 {
        let mut trialmech = mech.clone();
        let mut numcrits = roll_num_crits(trialmech.hardened);
        let mut engine_hits = 0;
        let mut gyro_hits = 0;
        while numcrits > 0 {
            let location = get_location(trialmech.config == Config::Tripod);
            let crits = match location {
                Location::Head => &mut trialmech.head_crits,
                Location::CenterTorso => &mut trialmech.ct_crits,
                Location::LeftTorso => &mut trialmech.lt_crits,
                Location::RightTorso => &mut trialmech.rt_crits,
                Location::LeftArm => &mut trialmech.la_crits,
                Location::RightArm => &mut trialmech.ra_crits,
                Location::LeftLeg => &mut trialmech.ll_crits,
                Location::RightLeg => &mut trialmech.rl_crits,
                Location::CenterLeg => &mut trialmech.cl_crits,
            };

            let (i, chosen_crit) = match crits.iter_mut().enumerate().choose(&mut rng) {
                Some(x) => x,
                None => break, // if we run out of crits then we're done checking, break out and see if we hit the engine/gyro threshold
            };

            let chosen_crit = *chosen_crit;
            crits.remove(i);

            match chosen_crit {
                Crit::Engine => engine_hits += 1,
                Crit::Cockpit => {
                    floating_deaths += 1;
                    continue 'iteration;
                }
                Crit::Gyro => gyro_hits += 1,
                Crit::Ammo => {
                    if crits.contains(&Crit::CaseII) {
                        // TODO: track the one poitn of damage for when we handle explosive equipment
                        numcrits += roll_num_crits(false); // hardened doesn't apply for internal damage
                    } else if crits.contains(&Crit::Case) {
                        engine_hits += crits.iter().filter(|&c| *c == Crit::Engine).count();
                        gyro_hits += crits.iter().filter(|&c| *c == Crit::Gyro).count();
                        if engine_hits >= 3 || gyro_hits >= required_gyro_hits {
                            floating_deaths += 1;
                        }

                        continue 'iteration;
                    } else {
                        match location {
                            Location::LeftArm | Location::LeftLeg => {
                                if trialmech.lt_crits.contains(&Crit::CaseII) {
                                    // TODO: track the one point of damage for when we handle explosive equipment
                                    numcrits += roll_num_crits(false); // hardened doesn't apply for internal damage
                                } else if trialmech.lt_crits.contains(&Crit::Case) {
                                    engine_hits += trialmech
                                        .lt_crits
                                        .iter()
                                        .filter(|&c| *c == Crit::Engine)
                                        .count();
                                    gyro_hits += trialmech
                                        .lt_crits
                                        .iter()
                                        .filter(|&c| *c == Crit::Gyro)
                                        .count();
                                    if engine_hits >= 3 || gyro_hits >= required_gyro_hits {
                                        floating_deaths += 1;
                                    }

                                    continue 'iteration;
                                } else {
                                    floating_deaths += 1;
                                    continue 'iteration;
                                }
                            }
                            Location::RightArm | Location::RightLeg => {
                                if trialmech.rt_crits.contains(&Crit::CaseII) {
                                    // TODO: track the one point of damage for when we handle explosive equipment
                                    numcrits += roll_num_crits(false); // hardened doesn't apply for internal damage
                                } else if trialmech.lt_crits.contains(&Crit::Case) {
                                    let torso_engine_hits = trialmech
                                        .rt_crits
                                        .iter()
                                        .filter(|&c| *c == Crit::Engine)
                                        .count();
                                    let torso_gyro_hits = trialmech
                                        .rt_crits
                                        .iter()
                                        .filter(|&c| *c == Crit::Gyro)
                                        .count();
                                    if torso_engine_hits + engine_hits >= 3
                                        || torso_gyro_hits + gyro_hits >= required_gyro_hits
                                    {
                                        floating_deaths += 1;
                                    }

                                    continue 'iteration;
                                } else {
                                    floating_deaths += 1;
                                    continue 'iteration;
                                }
                            }
                            _ => {
                                floating_deaths += 1;
                                continue 'iteration;
                            }
                        }
                    }
                }
                Crit::Explosive(_) => todo!(),
                _ => (),
            }

            numcrits -= 1;
        }

        if engine_hits >= 3 || gyro_hits >= required_gyro_hits {
            deaths += 1;
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

fn parse_crits(line: String, crits: &mut Vec<Crit>) -> ControlFlow<()> {
    match line.as_str() {
        "-empty-" => (), // TODO: need a mechanism to consider endo and the like to also be non-crits
        "" => return ControlFlow::Break(()),
        _ => {
            if line.contains("ammo") {
                crits.push(Crit::Ammo);
            } else if line.contains("cockpit") {
                crits.push(Crit::Cockpit);
            } else if line.contains("engine") {
                crits.push(Crit::Engine);
            } else if line.contains("gyro") {
                crits.push(Crit::Gyro);
            } else if line.contains("caseii") {
                crits.push(Crit::CaseII);
            } else if line.contains("case") {
                crits.push(Crit::Case);
            } else if line.contains("heavygaussrifle") {
                crits.push(Crit::Explosive(25));
            } else if line.contains("lightgaussrifle") {
                crits.push(Crit::Explosive(10));
            } else if line.contains("gaussrifle") {
                crits.push(Crit::Explosive(20));
            } else {
                crits.push(Crit::Other);
            }
        }
    }

    // TODO: need to fill out this more for all the explosive equipment and double check the right parsing order.
    // Also need to account for non-explosive ammo.  And need to not count Endo-Steel, Ferro-Fibrous, and the like
    // as crittable.

    ControlFlow::Continue(())
}

fn roll_num_crits(is_hardened: bool) -> i32 {
    let mut result = two_d_six();
    if is_hardened {
        result = cmp::max(2, result - 2);
    }

    return match result {
        2..=7 => 0,
        8 | 9 => 1,
        10 | 11 => 2,
        12 => 3,
        _ => panic!("2d66 somehow was not between 2 and 12"),
    };
}

fn get_location(is_tripod: bool) -> Location {
    let mut location = match two_d_six() {
        2 | 7 => Location::CenterTorso,
        3 | 4 => Location::RightArm,
        5 => Location::RightLeg,
        6 => Location::RightTorso,
        8 => Location::LeftTorso,
        9 => Location::LeftLeg,
        10 | 11 => Location::LeftArm,
        12 => Location::Head,
        _ => panic!("2d6 somehow was not between 2 and 12"),
    };

    // TODO: double check this mapping
    if is_tripod && (location == Location::LeftLeg || location == Location::RightLeg) {
        match one_d_six() {
            1 | 2 => location = Location::RightLeg,
            3 | 4 => location = Location::CenterLeg,
            5 | 6 => location = Location::LeftLeg,
            _ => panic!("1d6 somehow was not between 1 and 6"),
        }
    }

    location
}

fn one_d_six() -> u8 {
    let mut rng = rand::thread_rng();
    let d1 = rng.gen_range(1..=6);

    d1
}

fn two_d_six() -> u8 {
    let mut rng = rand::thread_rng();
    let d1 = rng.gen_range(1..=6);
    let d2 = rng.gen_range(1..=6);

    d1 + d2
}
