use std::io::Write;

use structopt::StructOpt;
mod mask;
mod dico;

struct MaskDescriptor {
    pub dico: char,
    pub len: u8,
}

impl std::str::FromStr for MaskDescriptor {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            Err("MaskDescriptor must be of len 2!")
        } else {
            let mut iter = s.chars();
            let dico = iter.next().unwrap();

            let len = match iter.next().unwrap() {
                len @ '5'..='9' => Ok(len as u8 - '0' as u8),
                _ => Err("len must be between 5 and 9 included"),
            }?;

            Ok(Self { dico, len })
        }
    }
}

#[derive(StructOpt)]
struct Args {
    pub search: MaskDescriptor,
}

// use https://www.tusmo.xyz/s3da53bb 4 tests
// inventees

fn main() {
    let start = std::time::Instant::now();

    let args = Args::from_args();
    let mask_desc = args.search;
    let start = log_section(start, "Args parsed");

    let dico = match dico::load(mask_desc.dico, mask_desc.len + 1) {
        Ok(dico) => dico,
        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    };

    if dico.len() == 0 {
        print!("No world of len {} found in '{}.txt'", mask_desc.len, mask_desc.dico);
        return;
    }

    let mut mask = mask::Mask::new(mask_desc.dico, mask_desc.len);
    let mut result = mask::ResultState::new(mask_desc.len as usize + 1);
    log_section(start, "Dico loaded");

    while !result.complet() {
        let start = std::time::Instant::now();
        let (word_id, score) = match mask.find_best(&dico) {
            Ok(id) => id,
            Err(err) => {
                eprintln!("{}", err);
                return;
            }
        };

        println!("Guess found in {:.2}s", start.elapsed().as_secs_f32());

        let list = match mask.filter_dico(&dico) {
            Ok(list) => list,
            Err(err) => {
                eprintln!("{}", err);
                return;
            }
        };

        if list.len() <= 5 {
            println!("{:?}", list);

            if list.len() == 1 {
                return;
            }
        }

        println!("Best word: {} ({:.2})", dico[word_id], score);
        let mut buf = String::with_capacity(10);

        result = loop {
            buf.clear();
            print!("Result: ");
            std::io::stdout().flush().unwrap();

            if let Err(err) = std::io::stdin().read_line(&mut buf) {
                eprintln!("{}", err);
                return;
            }

            match buf.trim().try_into() {
                Ok(rs) => break rs,
                Err(err) => eprintln!("{}", err),
            }
        };

        if let Err(err) = mask.update(&dico[word_id], &result) {
            eprintln!("{}", err);
            return;
        }

        println!("{:?}", mask);
    }
}

fn log_section(start: std::time::Instant, msg: &str) -> std::time::Instant {
    println!("{} in {}µs", msg, start.elapsed().as_micros());
    std::time::Instant::now()
}
