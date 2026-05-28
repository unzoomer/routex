use sysinfo::{System, ProcessesToUpdate};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Game {
    pub name: &'static str,
    pub process: &'static str,
    pub ports: &'static [u16],
}

pub const GAMES: &[Game] = &[
    Game {
        name: "CS2",
        process: "cs2.exe",
        ports: &[27015, 27016, 27017, 27018, 27019, 27020],
    },
    Game {
        name: "Dota 2",
        process: "dota2.exe",
        ports: &[27015, 27016, 27017, 27018, 27019, 27020],
    },
    Game {
        name: "Valorant",
        process: "VALORANT-Win64-Shipping.exe",
        ports: &[7086, 8088],
    },
];

pub struct GameDetector {
    system: System,
}

impl GameDetector {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_processes(ProcessesToUpdate::All, true);
    }

    pub fn detect_running(&mut self) -> Vec<&'static Game> {
        self.refresh();
        let mut running = Vec::new();

        for game in GAMES {
            for (_pid, process) in self.system.processes() {
                if process.name().to_string_lossy()
                    .to_lowercase()
                    .contains(&game.process.to_lowercase())
                {
                    running.push(game);
                    break;
                }
            }
        }

        running
    }

    pub fn find_pid(&mut self, process_name: &str) -> Option<u32> {
        self.refresh();
        for (pid, process) in self.system.processes() {
            if process.name().to_string_lossy()
                .to_lowercase()
                .contains(&process_name.to_lowercase())
            {
                return Some(pid.as_u32());
            }
        }
        None
    }
}