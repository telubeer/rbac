extern crate json;
use std;

pub struct Deserializer<'de> {
    input: &'de str,
}


impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input }
    }

    // Look at the first character in the input without consuming it.
    pub fn peek_char(&mut self) -> Result<char> {
        self.input.chars().next().ok_or(String::from("a"))
    }

    // Consume the first character in the input.
    pub fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    fn read_until(&mut self, stop: char) -> String {
        match self.input.find(stop) {
            Some(len) => {
                let s = &self.input[..len];
                self.input = &self.input[len + 1..];
                String::from(s)
            }
            None => String::from(self.input.clone())
        }
    }

    fn read_chrs(&mut self, count: u8) -> String {
        let mut buf = String::from("");
        for _n in 0..count {
            let ch = self.next_char().unwrap();
            buf.push(ch);
        }
        return buf;
    }

    fn numeric_array(&mut self, size: u8) -> json::JsonValue {
        let mut ar = json::JsonValue::new_array();
        for _n in 0..size {
            self.read_until(';');
            let mut delimiter = ';';
            if self.peek_char().unwrap() == 'a' {
                delimiter = '}'
            }
            let mut value = self.read_until(delimiter);
            value.push(';');
            let mut d = Deserializer::from_str(&value);
            let v = d.parse();
            ar.push(v);
        }
        self.next_char();
        return ar;
    }

    fn parse_array(&mut self, size: u8) -> json::JsonValue {
        self.next_char();
        if self.peek_char().unwrap() == 'i' {
            return self.numeric_array(size);
        }
        let mut hash = json::JsonValue::new_object();
        for _n in 0..size {
            let keyval = self.read_until(';');
            let mut key = "".to_string();
            key.push_str(Deserializer::from_str(&keyval).parse().as_str().unwrap());
            let mut delimiter = ';';
            if self.peek_char().unwrap() == 'a' {
                delimiter = '}'
            }
            let mut value = self.read_until(delimiter);
            value.push(';');
            let mut d = Deserializer::from_str(&value);
            let v = d.parse();
            hash[key] = v;
        }
        self.next_char();
        return hash;
    }

    pub fn parse(&mut self) -> json::JsonValue {
        let t = self.next_char().unwrap();
        self.next_char();
        match t {
            's' => {
                let size: u8 = self.read_until(':').parse().unwrap();
                self.next_char();
                let mut val = "\"".to_string();
                val.push_str(self.read_chrs(size).as_ref());
                val.push_str("\"");
                json::parse(&val).unwrap()
            }
            'i' => {
                let mut val = "\"".to_string();
                val.push_str(self.read_until(';').as_ref());
                val.push_str("\"");
                json::parse(&val).unwrap()
            }
            'a' => {
                let size: u8 = self.read_until(':').parse().unwrap();
                self.parse_array(size)
            }
            _ => json::Null
        }
    }
}

type Result<T> = std::result::Result<T, String>;
