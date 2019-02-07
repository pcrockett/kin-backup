use std::io;
use std::io::Write;

pub fn prompt(question: &str) -> Result<String, failure::Error> {

    print!("{} ", question);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    input.pop().unwrap(); // Remove newline at end

    Ok(input)
}
