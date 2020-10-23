use postgres::{Client, NoTls};
use regex::Regex;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead};
use structopt::StructOpt;

// This structure defines all the possibles arguments the script
// can accept. We could add a date format to use with other files
// or an option to accept the type of insert that is going to be 
// done and use this option for a different database
#[derive(StructOpt)]
struct Options {
    #[structopt(short = "f", long = "file")]
    /// File name
    file: String,

    #[structopt(default_value = ",", short = "s", long = "separator")]
    /// Value separator
    separator: char,

    #[structopt(short = "h", long = "header")]
    /// Value separator
    header: bool,

    #[structopt(short = "v", long = "verbose")]
    /// Value separator
    verbose: bool,

    #[structopt(default_value = "localhost", long = "host")]
    /// Database location
    host: String,

    #[structopt(default_value = "postgres", long = "user")]
    /// User name
    user: String,

    #[structopt(default_value = "root", long = "password")]
    /// User password
    password: String,

    #[structopt(default_value = "testdb", long = "dbname")]
    /// Database name
    dbname: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::from_args();

    let skip = match options.header {
        true => 1,
        false => 0,
    };

    let parameters = format!(
        "host={} user={} password={} dbname={}",
        options.host, options.user, options.password, options.dbname
    );

    let mut client = Client::connect(parameters.as_str(), NoTls)?;

    // Base format for the dates in the file
    let fmt = "%Y-%m-%d";

    // This base date will be used when no date was found
    let base_date = chrono::NaiveDate::from_ymd(1900, 1, 1);

    // Regex expression to remove all non digits from the date
    let re = Regex::new(r"[^\d-]+").unwrap();

    let mut lines = 0;
    let mut errors = 0;

    // Opening the file to be read
    let file = fs::File::open(options.file)?;
    for line in io::BufReader::new(file).lines().skip(skip) {
        let line = line?;
        let values: Vec<&str> = line.split(options.separator).collect();

        // The date has to be cleaned in order to be parsed as date
        let clean_date = re.replace(values[5], "").to_string();
        let created_at =
            chrono::NaiveDate::parse_from_str(clean_date.as_str(), fmt).unwrap_or(base_date);

        let clean_date = re.replace(values[6], "").to_string();
        let paid_at =
            chrono::NaiveDate::parse_from_str(clean_date.as_str(), fmt).unwrap_or(base_date);

        if options.verbose {
            println!("{}", line);
        }

        match client.execute(
            "INSERT INTO data (id, name, company, amount, status, created_at, paid_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[&values[0],
              &values[1],
              &values[2],
              &values[3].parse::<f32>().unwrap_or(0.0),
              &values[4],
              &created_at,
              &paid_at]
        ) {
            Ok(_) => { lines += 1 },
            Err(e) => {
                errors += 1;
                eprintln!("{:?}", e);
            }
        }
    }

    println!("Lines stored: {}\tErrors: {}", lines, errors); 

    Ok(())
}
