use serde::Deserialize;

pub mod data 
{
    use std::{collections::{HashMap, HashSet}, fs::File, io::BufReader};

    use serde::Deserialize;


    #[derive(Deserialize)]
    pub struct Achievement 
    {
        pub id: u32,
        pub series: u32,
        pub series_name: String,
        pub name: String,
        pub description: String,
        pub jades: u32,
        pub hidden: bool,
        pub version: String,
        pub gacha: bool,
        pub impossible: bool,
        pub percent: f32,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct AchievementId(pub [u8; 4]);

    #[derive(Debug, Clone)]
    pub struct MemAchievement {
        pub name: String,
    }

    pub struct AchievementContainer
    {
        pub achievements: HashMap<AchievementId, MemAchievement>
    }

    const ACHIEVEMENTS: &str = include_str!("achievements.json");

    impl AchievementContainer
    {
        pub fn get() -> AchievementContainer
        {

            // Read the JSON contents of the file as an instance of `User`.
            let achievements: Vec<Achievement> = serde_json::from_str(ACHIEVEMENTS).unwrap();
            let achievements = achievements.into_iter().map(|a| {
                let id = a.id.to_ne_bytes();
                let id = AchievementId(id);
                let mem_achievement = MemAchievement { name: a.name };
                (id, mem_achievement)
            }).collect();
            
            
            AchievementContainer { achievements }
        }
    }

}