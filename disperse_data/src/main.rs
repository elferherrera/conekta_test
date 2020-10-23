use postgres::{Client, NoTls};
use std::error::Error;
use structopt::StructOpt;
use std::collections::HashMap;
use regex::Regex;

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


// Creates a map from the company table to be used to reference all the
// transactions that will be used
fn create_map(client: &mut Client) -> Result<HashMap<String, String>, Box<dyn Error>> {

    let mut map: HashMap<String, String> = HashMap::new();

    for row in client.query("SELECT id, name FROM company", &[])? {
        let id: String = row.get(0);
        let name: String = row.get(1);

        map.insert(name, id);
    }

    Ok(map)
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
    let mut ignored = 0;
    let mut errors = 0;

    // Map with a relationship between the names and company ids
    let map = create_map(&mut client)?;

    // Regex expression to check the id matches. It has to have only
    // numbers and letters and have a length of 24
    let re = Regex::new(r"[\w]{24}").unwrap();

    for row in client.query(
        "SELECT id, company_name, company_id, amount, status, created_at, updated_at FROM cargo",
        &[],
    )? {
        let id: String = row.get(0);
        let company_name: String = row.get(1);
        let company_id: String = row.get(2);
        let amount: f32 = row.get(3);
        let status: String = row.get(4);
        let created_at: chrono::NaiveDateTime = row.get(5);
        let updated_at: chrono::NaiveDateTime = row.get(6);

        if options.verbose {
            println!("{} {}", company_id, company_name);
        }

        let final_id = match re.is_match(company_id.as_str()) {
            true => company_id,
            false => {
                match map.get(company_name.as_str()) {
                    Some(id) => id.to_string(),
                    None => "".to_string(),
                }
            }
        };

        // If the ID doesn't match the required parameters then
        // the transaction is ignored and is not stored
        if final_id == "" {
            println!("No company ID found for transaction {}", id);
            ignored += 1;
            continue;
        }

        match client.execute(
            "INSERT INTO charges (id, company_id, amount, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&id,
              &final_id,
              &amount,
              &status,
              &created_at,
              &updated_at,
            ]) {
            Ok(_) => { lines += 1 },
            Err(e) => {
                errors += 1;
                eprintln!("{:?}", e);
            }
        }

    }

    println!("Lines:{}\tErrors:{}\tIgnored:{}", lines, errors, ignored);

    Ok(())
}
