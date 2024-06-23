use anyhow::Error;

pub fn print(err: &Error) {
    eprintln!("{:?}", err);
}
