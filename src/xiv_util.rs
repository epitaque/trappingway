use std::str::FromStr;
use std::fmt;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Job {
    Gunbreaker,
    Paladin,
    Gladiator,
    DarkKnight,
    Warrior,
    Marauder,
    Scholar,
    Arcanist,
    Sage,
    Astrologian,
    WhiteMage,
    Conjurer,
    Samurai,
    Dragoon,
    Ninja,
    Monk,
    Reaper,
    Bard,
    Machinist,
    Dancer,
    BlackMage,
    BlueMage,
    Summoner,
    RedMage,
    Lancer,
    Pugilist,
    Rogue,
    Thaumaturge,
    Archer
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Role {
    Tank,
    DPS,
    Healer
}

#[derive(Debug)]
#[derive(Clone)]
pub struct PFListing {
    pub title: String,
    pub author: String,
    pub flags: String,
    pub description: String,
    pub slots: Vec<Slot>,
    pub last_updated: String,
    pub expires_in: String,
    pub min_ilvl: String,
    pub data_center: String,
    pub pf_category: String
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Slot {
    pub available_jobs: Vec<Job>,
    pub filled: bool,
}

#[allow(dead_code)]
impl Slot {   
    pub fn to_string(&self) -> String {
        // We don't want to disclose the secret
        format!("Slot({:#?}, {})", &self.available_jobs, &self.filled)
    }
    pub fn get_emoji_string(&self) -> String {
        if self.filled {
            match self.available_jobs.first() {
                Some(x) => x.get_emoji_string(),
                None => "".to_string()
            }
        } else {
            let contains_tank = self.available_jobs.iter().any(|x| x.get_role() == Role::Tank);
            let contains_healer = self.available_jobs.iter().any(|x| x.get_role() == Role::Healer);
            let contains_dps = self.available_jobs.iter().any(|x| x.get_role() == Role::DPS);

            if contains_tank && contains_healer && contains_dps {
                "<:tankhealerdps:985322491398459482>".to_string()
            } else if contains_tank && contains_healer && !contains_dps {
                "<:tankhealer:985322490375049246>".to_string()
            } else if contains_tank && !contains_healer && contains_dps {
                "<:tankdps:985322489422958662>".to_string()
            } else if contains_tank && !contains_healer && !contains_dps {
                "<:tank:985322488332443668>".to_string()
            } else if !contains_tank && contains_healer && contains_dps {
                "<:healerdps:985322474923233390>".to_string()
            } else if !contains_tank && contains_healer && !contains_dps {
                "<:healer:985322474134704138>".to_string()
            } else if !contains_tank && !contains_healer && contains_dps {
                "<:dps:985322470326280213>".to_string()
            } else {
                "".to_string()
            }
        }
    }
}

impl Job {
    pub fn get_emoji_string(&self) -> String {
        match self {
            Job::Gunbreaker => "<:gunbreaker:985322473337782384>".to_string(),
            Job::Paladin => "<:paladin:985322479318892584>".to_string(),
            Job::Gladiator => "<:gladiator:985322472079491152>".to_string(),
            Job::DarkKnight => "<:darkknight:985322469873303624>".to_string(),
            Job::Warrior => "<:warrior:985322493143318578>".to_string(),
            Job::Marauder => "<:marauder:985322476986826782>".to_string(),
            Job::Scholar => "<:scholar:985322486231089212>".to_string(),
            Job::Arcanist => "<:arcanist:985322461866369094>".to_string(),
            Job::Sage => "<:sage:985322483823566908>".to_string(),
            Job::Astrologian => "<:astrologian:985322464127107093>".to_string(),
            Job::WhiteMage => "<:whitemage:985322493919244328>".to_string(),
            Job::Conjurer => "<:conjurer:985322468308811886>".to_string(),
            Job::Samurai => "<:samurai:985322484842758235>".to_string(),
            Job::Dragoon => "<:dragoon:985322471232245860>".to_string(),
            Job::Ninja => "<:ffxivninja:985322478521966612>".to_string(),
            Job::Monk => "<:monk:985322477683089418>".to_string(),
            Job::Reaper => "<:reaper:985322481025966150>".to_string(),
            Job::Bard => "<:bard:985322465733533736>".to_string(),
            Job::Machinist => "<:machinist:985322476244443246>".to_string(),
            Job::Dancer => "<:ffxivdancer:985322469172850728>".to_string(),
            Job::BlackMage => "<:blackmage:985322466723377202>".to_string(),
            Job::BlueMage => "<:bluemage:985322467599974421>".to_string(),
            Job::Summoner => "<:summoner:985322487191584839>".to_string(),
            Job::RedMage => "<:redmage:985322481889996890>".to_string(),
            Job::Lancer => "<:lancer:985322475225219084>".to_string(),
            Job::Pugilist => "<:pugilist:985322480203862056>".to_string(),
            Job::Rogue => "<:rogue:985322482879848458>".to_string(),
            Job::Thaumaturge => "<:thaumaturge:985322492258295818>".to_string(),
            Job::Archer  => "<:archer:985322463552495616>".to_string()
        }
    }

    pub fn get_role(&self) -> Role {
        let tanks = vec![Job::Paladin, Job::Gunbreaker, Job::DarkKnight, Job::Warrior, Job::Marauder, Job::Gladiator];
        let healers = vec![Job::Conjurer, Job::WhiteMage, Job::Scholar, Job::Astrologian, Job::Sage];

        if tanks.contains(self) {
            Role::Tank
        } else if healers.contains(self) {
            Role::Healer
        } else {
            Role::DPS
        }
    }
}

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Job::Gunbreaker => write!(f, "Gunbreaker"),
            Job::Paladin => write!(f, "Paladin"),
            Job::Gladiator => write!(f, "Gladiator"),
            Job::DarkKnight => write!(f, "DarkKnight"),
            Job::Warrior => write!(f, "Warrior"),
            Job::Marauder => write!(f, "Marauder"),
            Job::Scholar => write!(f, "Scholar"),
            Job::Arcanist => write!(f, "Arcanist"),
            Job::Sage => write!(f, "Sage"),
            Job::Astrologian => write!(f, "Astrologian"),
            Job::WhiteMage => write!(f, "WhiteMage"),
            Job::Conjurer => write!(f, "Conjurer"),
            Job::Samurai => write!(f, "Samurai"),
            Job::Dragoon => write!(f, "Dragoon"),
            Job::Ninja => write!(f, "Ninja"),
            Job::Monk => write!(f, "Monk"),
            Job::Reaper => write!(f, "Reaper"),
            Job::Bard => write!(f, "Bard"),
            Job::Machinist => write!(f, "Machinist"),
            Job::Dancer => write!(f, "Dancer"),
            Job::BlackMage => write!(f, "BlackMage"),
            Job::BlueMage => write!(f, "BlueMage"),
            Job::Summoner => write!(f, "Summoner"),
            Job::RedMage => write!(f, "RedMage"),
            Job::Lancer => write!(f, "Lancer"),
            Job::Pugilist => write!(f, "Pugilist"),
            Job::Rogue => write!(f, "Rogue"),
            Job::Thaumaturge => write!(f, "Thaumaturge"),
            Job::Archer => write!(f, "Archer"),
        }
    }
}

impl FromStr for Job {

    type Err = ();

    fn from_str(input: &str) -> Result<Job, Self::Err> {
        match input {
            "PLD"  => Ok(Job::Paladin),
            "WAR"  => Ok(Job::Warrior),
            "DRK"  => Ok(Job::DarkKnight),
            "GNB"  => Ok(Job::Gunbreaker),
            "GLD"  => Ok(Job::Gladiator),
            "MRD"  => Ok(Job::Marauder),
            "WHM"  => Ok(Job::WhiteMage),
            "SCH"  => Ok(Job::Scholar),
            "AST"  => Ok(Job::Astrologian),
            "SGE"  => Ok(Job::Sage),
            "CNJ"  => Ok(Job::Conjurer),
            "ARN"  => Ok(Job::Arcanist),
            "MNK"  => Ok(Job::Monk),
            "PGL"  => Ok(Job::Pugilist),
            "DRG"  => Ok(Job::Dragoon),
            "LNC"  => Ok(Job::Lancer),
            "NIN"  => Ok(Job::Ninja),
            "ROG"  => Ok(Job::Rogue),
            "SAM"  => Ok(Job::Samurai),
            "RPR"  => Ok(Job::Reaper),
            "BRD"  => Ok(Job::Bard),
            "ARC"  => Ok(Job::Archer),
            "MCH"  => Ok(Job::Machinist),
            "DNC"  => Ok(Job::Dancer),
            "BLM"  => Ok(Job::BlackMage),
            "SMN"  => Ok(Job::Summoner),
            "BLU"  => Ok(Job::BlueMage),
            "RDM"  => Ok(Job::RedMage),
            "RGE"  => Ok(Job::Rogue),
            "THM"  => Ok(Job::Thaumaturge),
            "ACN"  => Ok(Job::Arcanist),

            _      => Err(()),
        }
    }
}

pub fn get_color_from_duty(duty_name: &str) -> u32 {
    match duty_name {
        "The Unending Coil of Bahamut (Ultimate)" => 0xfce100,
        "The Weapon's Refrain (Ultimate)" => 0x008bfc,
        "The Epic of Alexander (Ultimate)" => 0xfcaa00,
        "Dragonsong's Reprise (Ultimate)" =>  0xf12916,
        _ =>  0xf0a057
    }
}