use std::io;
use std::io::Write;
use std::iter::Iterator;

pub fn prompt(question: &str) -> Result<String, failure::Error> {
    print!("{} ", question);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // input has a newline at the end. On Linux or MacOS that could be an \n.
    // On Windows it could be an \r\n. `input.lines()` is smart enough to
    // take care of that for us.
    let input = input.lines().nth(0).unwrap();

    Ok(String::from(input))
}
