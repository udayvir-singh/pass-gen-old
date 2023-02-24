mod data;
use rand::Rng;
use std::{
    env::args,
    fs::File,
    io::{BufRead, BufReader},
    ops::Range,
    process::{exit, Command, Stdio},
};

/* -------------------- *
 *        UTILS         *
 * -------------------- */
const MINUTE: f64  = 60.0;
const HOUR: f64    = MINUTE * 60.0;
const DAY: f64     = HOUR * 24.0;
const YEAR: f64    = DAY * 365.25;
const CENTURY: f64 = YEAR * 100.0;

macro_rules! error {
    ($($x:expr),*) => {{
        eprintln!("pass-gen: {}", format!($($x,)*));
        exit(1);
    }}
}


/* -------------------- *
 *      TOKEN DATA      *
 * -------------------- */
#[derive(Debug)]
enum TokenData {
    Static(&'static [&'static str]),
    Owned(Vec<String>),
}

impl TokenData {
    fn get(&self, idx: usize) -> &str {
        match self {
            TokenData::Static(x) => x[idx],
            TokenData::Owned(x) => &x[idx],
        }
    }

    fn len(&self) -> usize {
        match self {
            TokenData::Static(x) => x.len(),
            TokenData::Owned(x) => x.len(),
        }
    }

    fn range(&self) -> Range<usize> {
        0..self.len()
    }
}


/* -------------------- *
 *        CONFIG        *
 * -------------------- */
#[derive(Debug)]
struct Config<'a> {
    report: bool,
    token_count: u32,
    token_sep: &'a str,
    token_data: TokenData,
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Config {
            report: false,
            token_count: data::word::TOKEN_COUNT,
            token_sep: data::word::TOKEN_SEP,
            token_data: TokenData::Static(&data::word::TOKEN_DATA),
        }
    }
}

impl<'a> Config<'a> {
    fn new(args: &'a [String]) -> Self {
        let mut config = Config::default();

        let mut idx = 1;
        while let Some(flag) = args.get(idx).map(String::as_str) {
            idx += 1;

            match flag {
                "-r" | "--report" => {
                    config.report = true;
                }
                "-c" | "--count" => {
                    config.token_count = Self::get_number(flag, args, &mut idx);
                }
                "-s" | "--sep" => {
                    config.token_sep = Self::get_string(flag, args, &mut idx);
                }
                "-f" | "--file" => {
                    let path = Self::get_string(flag, args, &mut idx);

                    config.token_data = match File::open(path) {
                        Ok(f) => {
                            TokenData::Owned(BufReader::new(f).lines().map(Result::unwrap).collect())
                        },
                        Err(e) => {
                            error!("error while reading token file: {}", e)
                        },
                    }
                }
                "-p" | "--preset" => {
                    let preset = Self::get_string(flag, args, &mut idx);

                    config = match preset {
                        "ascii" => Self {
                            report: config.report,
                            token_count: data::ascii::TOKEN_COUNT,
                            token_sep: data::ascii::TOKEN_SEP,
                            token_data: TokenData::Static(&data::ascii::TOKEN_DATA),
                        },
                        "number" => Self {
                            report: config.report,
                            token_count: data::number::TOKEN_COUNT,
                            token_sep: data::number::TOKEN_SEP,
                            token_data: TokenData::Static(&data::number::TOKEN_DATA),
                        },
                        "word" => Self {
                            report: config.report,
                            token_count: data::word::TOKEN_COUNT,
                            token_sep: data::word::TOKEN_SEP,
                            token_data: TokenData::Static(&data::word::TOKEN_DATA),
                        },
                        _ => error!("invalid preset {:?}", preset),
                    }
                },
                _ => error!("invalid option {:?}", flag),
            }
        }

        config
    }

    fn get_string(flag: &str, args: &'a [String], idx: &mut usize) -> &'a str {
        if let Some(str) = args.get(*idx) {
            *idx += 1;
            str
        } else {
            error!("missing argument to {}", flag)
        }
    }

    fn get_number(flag: &str, args: &'a [String], idx: &mut usize) -> u32 {
        let str = Self::get_string(flag, args, idx);
        let int = str.parse();

        if matches!(int, Ok(i) if i > 0) {
            int.unwrap()
        } else {
            error!("invalid argument to {:?}, expected positve number got {:?}", flag, str);
        }
    }
}


/* -------------------- *
 *       REPORTER       *
 * -------------------- */
struct Reporter {
    pool_size: f64,
    token_count: f64,
}

impl Reporter {
    fn new(pool_size: f64, token_count: f64) -> Self {
        Self { pool_size, token_count }
    }

    fn print_report(&self) {
        let entropy = self.pool_size.log2();
        let total_entropy = entropy * self.token_count;

        eprintln!("entropy per word:           {:.1} bits", entropy);
        eprintln!("total entropy:              {:.0} bits", total_entropy);
        eprintln!("guess times:");
        eprintln!("  1 billion / second:       {}", Self::format_time((total_entropy - 31.0).exp2()));
        eprintln!("  1 quadrillion / second:   {}", Self::format_time((total_entropy - 51.0).exp2()));
        eprintln!("  1 sextillion / second:    {}", Self::format_time((total_entropy - 71.0).exp2()));
        eprintln!("{}", "-".repeat(Self::get_term_width()));
    }

    fn format_time(t: f64) -> String {
        match () {
            _ if t < 1.0     => String::from("less than a second"),
            _ if t < MINUTE  => Self::format_unit(t, "seconds"),
            _ if t < HOUR    => Self::format_unit(t / MINUTE, "minutes"),
            _ if t < DAY     => Self::format_unit(t / HOUR, "hours"),
            _ if t < YEAR    => Self::format_unit(t / DAY, "days"),
            _ if t < CENTURY => Self::format_unit(t / YEAR, "years"),
            _ => Self::format_unit(t / CENTURY, "centuries"),
        }
    }

    fn format_unit(x: f64, unit: &str) -> String {
        if x < 1e6 {
            format!("{:.0} {}", x, unit)
        } else {
            format!("{} {}", format!("{:.0e}", x).replace('e', "e+"), unit)
        }
    }

    fn get_term_width() -> usize {
        Command::new("tput")
            .arg("cols")
            .stderr(Stdio::inherit())
            .output()
            .map(|o| String::from_utf8(o.stdout).unwrap().trim().parse().unwrap())
            .unwrap_or(80)
    }
}


/* -------------------- *
 *         MAIN         *
 * -------------------- */
fn main() {
    // parse config
    let args: Vec<String> = args().collect();
    let config = Config::new(&args);

    // print report
    if config.report {
        let reporter = Reporter::new(
            config.token_data.len() as f64,
            config.token_count as f64,
        );

        reporter.print_report();
    }

    // generate password
    let mut rng = rand::thread_rng();

    for i in 1..=config.token_count {
        let idx = rng.gen_range(config.token_data.range());

        print!("{}", config.token_data.get(idx));

        if i != config.token_count {
            print!("{}", config.token_sep);
        };
    }
}
