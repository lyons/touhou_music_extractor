use colored::*;
use rust_embed::RustEmbed;
use std::{
  io::{Cursor, Read},
};

#[derive(RustEmbed)]
#[folder = "bgminfo/"]
pub struct BgmStore;

impl BgmStore {
  pub fn get_from_token(token: &str) -> Option<String> {
    let token = token.to_ascii_lowercase();
    let filename = match token.as_str() {
      "th6"  | "th06"   | "eosd"  | "embodiment of scarlet devil"     => "th06.bgm",
      "th7"  | "th07"   | "pcb"   | "perfect cherry blossom"          => "th07.bgm",
      "th8"  | "th08"   | "in"    | "imperishable night"              => "th08.bgm",
      "th9"  | "th09"   | "pofv"  | "phantasmagoria of flower view"   => "th09.bgm",
      "th9.5"| "th09.5" | "stb"   | "shoot the bullet"                => "th09.5.bgm",
               "th10"   | "mof"   | "mountain of faith"               => "th10.bgm",
               "th11"   | "sa"    | "subterranean animism"            => "th11.bgm",
               "th12"   | "ufo"   | "undefined fantastic object"      => "th12.bgm",
               "th12.5" | "ds"    | "double spoiler"                  => "th12.5.bgm",
               "th12.8" | "fw"    | "fairy wars"                      => "th12.8.bgm",
               "th13"   | "td"    | "ten desires"                     => "th13.bgm",
               "th14"   | "ddc"   | "double dealing character"        => "th14.bgm",
               "th14.3" | "isc"   | "impossible spell card"           => "th14.3.bgm",
               "th15"   | "lolk"  | "legacy of lunatic kingdom"       => "th15.bgm",
               "th16"   | "hsifs" | "hidden star in four seasons"     => "th16.bgm",
               "th16.5" | "vd"    | "violet detector"                 => "th16.5.bgm",
               "th17"   | "wbawc" | "wily beast and weakest creature" => "th17.bgm",
      _ => "",
    };

    if let Some(data) = BgmStore::get(filename) {
      let mut c = Cursor::new(data);
      let mut s = String::new();

      match c.read_to_string(&mut s) {
        Ok(_)  => Some(s),
        Err(_) => None,
      }
    }
    else {
      None
    }
  }
}

pub fn print_command_line_help() {
  line("TH6",   "TH06",    "EoSD",  "\"Embodiment of Scarlet Devil\"");
  line("TH7",   "TH07",    "PCB",   "\"Perfect Cherry Blossom\"");
  line("TH8",   "TH08",    "IN",    "\"Imperishable Night\"");
  line("TH9",   "TH09",    "PoFV",  "\"Phantasmagoria of Flower View\"");
  line("TH9.5",  "TH09.5", "StB",   "\"Shoot the Bullet\"");
  line("",       "TH10",   "MoF",   "\"Mountain of Faith\"");
  line("",       "TH11",   "SA",    "\"Subterranean Animism\"");
  line("",       "TH12",   "UFO",   "\"Undefined Fantastic Object\"");
  line("",       "TH12.5", "DS",    "\"Double Spoiler\"");
  line("",       "TH12.8", "FW",    "\"Fairy Wars\"");
  line("",       "TH13",   "TD",    "\"Ten Desires\"");
  line("",       "TH14",   "DDC",   "\"Double Dealing Character\"");
  line("",       "TH14.3", "ISC",   "\"Impossible Spell Card\"");
  line("",       "TH15",   "LoLK",  "\"Legacy of Lunatic Kingdom\"");
  line("",       "TH16",   "HSiFS", "\"Hidden Star in Four Seasons\"");
  line("",       "TH16.5", "VD",    "\"Violet Detector\"");
  line("",       "TH17",   "WBaWC", "\"Wily Beast and Weakest Creature\"");
}

fn line(a: &str, b: &str, c: &str, d: &str) {
  println!("        {:8} {:8} {:8} {}", a.red(), b.magenta(), c.blue(), d.cyan());
}