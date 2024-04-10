mod data;
mod process;
use std::{collections::HashMap, fs};
use crate::{
    data::data::{AchievementId, AchievementContainer},
    process::process::{enum_proc, Process},
};
use serde::Serialize;
use winapi::um::winnt;
use colored::*;

const ADDR_LOWER: usize = 0x0000070000000000;
const ADDR_UPPER: usize = 0x0000400000000000;

fn main() {
    // The first step is finding the Star Rail process.

    let hsr_proc: Process = enum_proc()
        .unwrap()
        .into_iter()
        .filter_map(|pid| Process::open(pid).ok())
        .find(|proc| { proc.name().is_ok_and(|name| name == "StarRail.exe") })
        .ok_or_else(|| {
            panic!(
                "\n 
                {}: Star Rail process not found. Please ensure: \n- The game is running \n- You are running this program from an elevated command prompt (Ran as Administrator)
                \n", "ERROR: ".red().bold()
            )
        })
        .unwrap();

    println!("{} Star Rail process found with PID: {}", "SUCCESS: ".bright_green().bold(), &hsr_proc.pid);

    let chives: AchievementContainer = AchievementContainer::get();

    let mem = hsr_proc
        .memory_regions().
        into_iter().
        filter(|p| {
            ((p.Protect & winnt::PAGE_READWRITE) != 0)
            && ((p.BaseAddress as usize) >= ADDR_LOWER && (p.BaseAddress as usize) <= ADDR_UPPER)
        });

    let memsize = mem.clone().map(|p| p.RegionSize).sum::<usize>() as f64 * 0.000001;
    println!("Total memory size: {} MB", memsize);

    /*
    let debug_id: i32 = 4070421;
    let target = debug_id.to_ne_bytes();
    */

    let regions = mem.map(|p| p).collect::<Vec<_>>();

    println!("Scanning {} memory regions", regions.len());

    struct FoundChive {
        count: u32,
        status: u32,
    }

    let mut completed_counter = 0;
    let mut uncompleted_counter = 0;
    let mut found_chives: HashMap<AchievementId, FoundChive> = HashMap::new();

    for region in regions.into_iter() {
        let Ok(memory) = hsr_proc.read_memory(region.BaseAddress as _, region.RegionSize) 
        else { continue; };
        for (offset, window) in memory.windows(4).enumerate() {
            if let Some(chive) = chives
                .achievements
                .get(&data::data::AchievementId(window.try_into().unwrap()))
            {

                let status = hsr_proc.value_at(region.BaseAddress as usize + offset + 12).unwrap();
                let addl_check = hsr_proc.value_at(region.BaseAddress as usize + offset + 8).unwrap(); 
                if status != 3 && status != 1 { continue; }
                println!( "{} {} found at {:#04x} with status {}", "! Achievement".yellow().bold(), chive.name.blue().bold(), region.BaseAddress as usize + offset + 12, status );
                if let Some(found_chive) = found_chives.get_mut(&data::data::AchievementId(window.try_into().unwrap())) 
                {
                    if found_chive.status == status 
                    {
                        found_chive.count += 1
                    }
                    else 
                    {
                        println!("^----------------------DUPLICATE---------------------^");
                        if found_chive.status != 3
                            || (found_chive.status == 3 && status == 1 && addl_check == 1)
                        {
                            found_chive.count += 1;
                            found_chive.status = status;
                        }
                    }
                } 
                else 
                {
                    found_chives.insert(
                        data::data::AchievementId(window.try_into().unwrap()),
                        FoundChive { count: 1, status },
                    );
                }
            }
        }
    }

    println!("\n{} finished logging achievements.", "SUCCESS: ".bright_green().bold());

    #[derive(Serialize)]
    struct Output {
        achievements: Vec<String>,
    }

    let mut reports: Output = Output {
        achievements: Vec::new(),
    };
    for (id, chive) in found_chives.iter() {
        match chive.status {
            3 => {
                reports
                    .achievements
                    .push(unsafe { std::mem::transmute::<[u8; 4], u32>(id.0).to_string() });
                completed_counter += 1
            }
            1 => uncompleted_counter += 1,
            _ => (),
        }
    }
    fs::write(
        "output.json",
        serde_json::to_string_pretty(&reports).unwrap(),
    )
    .expect(&format!("{} Unable to write to achievement file.", "ERROR: ".red().bold()));
    println!("{} Data written to output.json \n", "SUCCESS: ".bright_green().bold());
    println!("Completed: {}", completed_counter.to_string().green().bold());
    println!("Uncompleted: {}", uncompleted_counter.to_string().magenta().bold());
}
