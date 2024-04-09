
mod process;
mod data;
use std::{collections::HashMap, fs};

use data::data::AchievementContainer;
use serde::Serialize;
use winapi::um::winnt;
use crate::{data::data::AchievementId, process::process::{enum_proc, Process}};

const ADDR_LOWER: usize = 0x0000070000000000;
const ADDR_UPPER: usize = 0x0000400000000000;


fn main() 
{
    // The first step is finding the Star Rail process.
    let mut hsr_proc: Option<Process> = None;

    enum_proc()
        .unwrap()
        .into_iter()
        .for_each(|pid| match Process::open(pid) {
            Ok(proc) => match proc.name() {
                Ok(name) => {
                    if name == "StarRail.exe" {
                        hsr_proc = Some(proc);
                    }
                }
                Err(e) => ()
            },
            Err(e) => (),
        });
    
    let Some(hsr_proc) = hsr_proc
    else {
        println!("Star Rail process not found.");
        return;
    };
    println!("Star Rail process found with PID: {}", hsr_proc.pid);

    let mut chives: AchievementContainer = AchievementContainer::get();

    let mask = winnt::PAGE_READWRITE;

    let mem_size = hsr_proc // process memory size in bytes
        .memory_regions()
        .into_iter()
        .filter(|p| (p.Protect & mask) != 0)
        .filter(|p| (p.BaseAddress as usize) >= ADDR_LOWER && (p.BaseAddress as usize) <= ADDR_UPPER )
        .map(|p| p.RegionSize)
        .sum::<usize>();

    // convert bytes to MB
    println!("Total memory size: {}MB", mem_size as f64 * 0.000001);

    

    let debug_id: i32 = 4070421; // debug achievement, this is the id for Trauma Team Platinum Bundle
    let target = debug_id.to_ne_bytes();

    let regions = hsr_proc
    .memory_regions()
    .into_iter()
    .filter(|p| (p.Protect & mask) != 0)
    .filter(|p| (p.BaseAddress as usize) >= ADDR_LOWER && (p.BaseAddress as usize) <= ADDR_UPPER )
    .map(|p| p )
    .collect::<Vec<_>>();


    println!("Scanning {} memory regions", regions.len());

    struct FoundChive {
        count: u32,
        status: u32,
    }
    
    let mut completed_counter = 0;
    let mut uncompleted_counter = 0;
    let mut all_counter = 0;
    let mut found_chives: HashMap<AchievementId, FoundChive> = HashMap::new();
    regions.into_iter().for_each(|region| {
        match hsr_proc.read_memory(region.BaseAddress as _, region.RegionSize) {
            Ok(memory) => memory
            .windows(target.len())
            .enumerate() // achievements are 4 bytes long
            .for_each(|(offset, window)| {
                
                if let Some(chive) = chives.achievements.get(&data::data::AchievementId(window.try_into().unwrap())) {
                    let status_offset_addr = unsafe { region.BaseAddress.offset(offset as isize).offset(12) };
                    let chive_offset_addr = unsafe { region.BaseAddress.offset(offset as isize).offset(8) };
                    let mut status_buffer: [u8; 4] = [0; 4];
                    let mut chive_buffer: [u8; 4] = [0; 4];
                    let mut status_number_read: usize = 0;
                    let mut chive_number_read: usize = 0;
                    unsafe { winapi::um::memoryapi::ReadProcessMemory(hsr_proc.handle.as_ptr(), status_offset_addr, status_buffer.as_mut_ptr().cast(), 4, &mut status_number_read) };
                    unsafe { winapi::um::memoryapi::ReadProcessMemory(hsr_proc.handle.as_ptr(), chive_offset_addr, chive_buffer.as_mut_ptr().cast(), 4, &mut chive_number_read) };
                    let status = u32::from_ne_bytes(status_buffer);
                    let is_chive = u32::from_ne_bytes(chive_buffer); // this is an additional check. A couple of chives are false positives, but this only occurs on real chives.
                        match status
                        {
                            3 | 1 => { 
                                all_counter += 1;
                                unsafe { println!("Found {} at {:?} with status {}", chive.name, region.BaseAddress.offset(offset as isize).offset(12), status) }
                                if let Some(found_chive) = found_chives.get_mut(&data::data::AchievementId(window.try_into().unwrap())) {
                                    if found_chive.status != status  {
                                        println!("^----------------------DUPLICATE---------------------^");
                                        if found_chive.status != 3
                                        || found_chive.status == 3 && status == 1 && is_chive == 1 {
                                            found_chive.count += 1;
                                            found_chive.status = status;
                                        }
                                    }
                                    else {
                                        found_chive.count += 1;
                                    }
                                } else {
                                    found_chives.insert(data::data::AchievementId(window.try_into().unwrap()), FoundChive { count: 1, status });
                                }
                            }
                            _ => (),
                        }
                }
            }),
            Err(_) => (),
            //Err(err) => eprintln!( "Failed to read {} bytes at {:?}: {}", region.RegionSize, region.BaseAddress, err, ),
        }
    });

    #[derive(Serialize)]
    struct Output
    {
        achievements: Vec<String>
    }
    
    let mut reports: Output = Output { achievements: Vec::new() };
    for (id, chive) in found_chives.iter() {
        match chive.status
        {
            3 => {reports.achievements.push(unsafe { std::mem::transmute::<[u8; 4], u32>(id.0).to_string()} ); completed_counter += 1 },
            1 => {uncompleted_counter += 1},
            _ => (),
        }
    }
    fs::write("output.json", serde_json::to_string_pretty(&reports).unwrap()).expect("Unable to write to achievement file.");
    println!("Data written to output.json");
    println!("Completed: {}", completed_counter);
    println!("Uncompleted: {}", uncompleted_counter);



}
