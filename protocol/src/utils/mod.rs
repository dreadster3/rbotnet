use std::io::{BufRead, Cursor, Read};

pub(crate) fn read_word(cursor: &mut Cursor<String>) -> Result<String, std::io::Error> {
    let mut word = String::new();

    let mut buf = Vec::new();
    let bytes = cursor.read_until(b' ', &mut buf)?;
    if bytes == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Unexpected end of file",
        ));
    }

    word.push_str(std::str::from_utf8(&buf).unwrap());
    word = word.trim().to_string();
    Ok(word)
}
