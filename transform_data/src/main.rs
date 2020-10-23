use postgres::{Client, NoTls};
use std::error::Error;
use structopt::StructOpt;

// This structure defines all the possibles arguments the script
// can accept. We could add a date format to use with other files
// or an option to accept the type of insert that is going to be
// done and use this option for a different database
#[derive(StructOpt)]
struct Options {
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

// Function to crop strings based on a crop size
fn crop_string(s: &str, size: usize) -> &str {
    if size < s.len() {
        return &s[..size];
    } 

    return s;
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::from_args();

    let parameters = format!(
        "host={} user={} password={} dbname={}",
        options.host, options.user, options.password, options.dbname
    );

    // Client to extract information from DB
    let mut client = Client::connect(parameters.as_str(), NoTls)?;

    let mut lines = 0;
    let mut errors = 0;

    for row in client.query(
        "SELECT id, name, company, amount, status, created_at, paid_at FROM data",
        &[],
    )? {
        let id: String = row.get(0);
        let name: String = row.get(1);
        let company: String = row.get(2);
        let amount: f32 = row.get(3);
        let status: String = row.get(4);
        let created_at: chrono::NaiveDate = row.get(5);
        let paid_at: chrono::NaiveDate = row.get(6);

        if options.verbose {
            println!("{} {}", id, name);
        }

        match client.execute(
            "INSERT INTO cargo (id, company_name, company_id, amount, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)", 
            &[
            &crop_string(id.replace(" ", "").as_str(), 24),
            &name.replace(" ", ""),
            &crop_string(company.replace(" ", "").as_str(), 24),
            &amount,
            &status.replace(" ", ""),
            &created_at.and_hms(0, 0, 0),
            &paid_at.and_hms(0, 0, 0),
            ]) {
            Ok(_) => { lines += 1},
            Err(e) => {
                eprintln!("Error: {}", e); 
                errors += 1;
            }
        }
    }

    println!("Lines:{}\tErrors:{}", lines, errors);

    Ok(())
}
