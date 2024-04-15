use std::ops::{AddAssign, MulAssign, Neg};

use serde::Deserialize;
use serde::de::{
    self, DeserializeSeed, MapAccess, SeqAccess, Visitor,
};

use super::error::{Error, Result};

#[derive(Debug)]
pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de str,
    keys: Vec<&'de str>,
}

impl<'de> Deserializer<'de> {
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_cs2::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_cs2::Deserializer::from_str(...)`.
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input, keys: vec![] }
    }
}

// By convention, the public API of a Serde deserializer is one or more
// `from_xyz` methods such as `from_str`, `from_bytes`, or `from_reader`
// depending on what Rust types the deserializer is able to consume as input.
//
// This basic deserializer supports only `from_str`.
pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

// SERDE IS NOT A PARSING LIBRARY. This impl block defines a few basic parsing
// functions from scratch. More complicated formats may wish to use a dedicated
// parsing library to help implement their Serde deserializer.
impl<'de> Deserializer<'de> {
    // Look at the first character in the input without consuming it.
    fn peek_char(&mut self) -> Result<char> {
        self.input.chars().next().ok_or(Error::Eof)
    }

    // Consume the first character in the input.
    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    // Parse the identifier `true` or `false`.
    fn parse_bool(&mut self) -> Result<bool> {
        if self.next_char()? == '1' {
            Ok(true)
        } else if self.next_char()? == '0' {
            Ok(false)
        } else {
            Err(Error::ExpectedBoolean)
        }
    }

    // Parse a group of decimal digits as an unsigned integer of type T.
    //
    // This implementation is a bit too lenient, for example `001` is not
    // allowed in cs2. Also the various arithmetic operations can overflow and
    // panic or return bogus data. But it is good enough for example code!
    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        let mut int = match self.next_char()? {
            ch @ '0'..='9' => T::from(ch as u8 - b'0'),
            _ => {
                return Err(Error::ExpectedInteger);
            }
        };
        loop {
            match self.input.chars().next() {
                Some(ch @ '0'..='9') => {
                    self.input = &self.input[1..];
                    int *= T::from(10);
                    int += T::from(ch as u8 - b'0');
                }
                _ => {
                    return Ok(int);
                }
            }
        }
    }

    // Parse bytes is used by serde_hex to deserialize hexadecimal values
    // in cs2 data
    //
    fn parse_bytes(&mut self) -> Result<&'de [u8]> {
        if self.input.starts_with("0x") {
            self.input = &self.input[2..];
        }
        if let Some(len) = self.input.find('\n') {
            let bytes = self.input[..len].as_bytes();
            self.input = &self.input[len..];
            return Ok(bytes);
        }
        let len = self.input.len();
        if len > 0 {
            let bytes = self.input[..len].as_bytes();
            self.input = &self.input[len..];
            return Ok(bytes);
        }
        return Err(Error::ExpectedNewline);
    }

    // Parse a possible minus sign followed by a group of decimal digits as a
    // signed integer of type T.
    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i8>,
    {
        let is_negative = match self.peek_char() {
            Ok('-') => {
                let _ = self.next_char();
                true
            },
            _ => false,
        };
        let mut int = match self.next_char()? {
            ch @ '0'..='9' => T::from((ch as u8 - b'0') as i8),
            _ => return Err(Error::ExpectedInteger),
        };
        loop {
            match self.input.chars().next() {
                Some(ch @ '0'..='9') => {
                    self.input = &self.input[1..];
                    int *= T::from(10);
                    int += T::from((ch as u8 - b'0') as i8);
                }
                _ => {
                    if is_negative {
                        int = -int;
                    }
                    return Ok(int);
                }
            }
        }
    }

    // Parse a string until the next newline or '=' character.
    //
    fn parse_string(&mut self) -> Result<&'de str> {
        if self.input.starts_with('.') {
            // this is a key values since it started with an .
            self.input = self.input.trim_start_matches('.');
            let line = self.input.lines().next().unwrap();
            if let Some(len) = line.find('=') {
                let s = &self.input[..len];
                self.input = &self.input[len..];
                Ok(s)
            } else {
                // this must be a sequence start, we just return the key value
                // without moving the string forward since the key is also the
                // name of the struct inside the sequence
                Ok(line)
            }
        } else if let Some(len) = self.input.find('\n') {
                let s = &self.input[..len];
                self.input = &self.input[len..];
                Ok(s)
        } else if self.input.len() > 0 {
                let len = self.input.len();
                let s = &self.input[..len];
                self.input = &self.input[len..];
                Ok(s)
        } else {
            Err(Error::ExpectedString)
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_any() {:?}", self.peek_char());
    }

    // Uses the `parse_bool` parsing function defined above to read the 
    // identifier `true` or `false` from the input.
    //
    // Parsing refers to looking at the input and deciding that it contains the
    // value `true` or `false`.
    //
    // Deserialization refers to mapping that cs2 value into Serde's data
    // model by invoking one of the `Visitor` methods. In the case of cs2 and
    // bool that mapping is straightforward so the distinction may seem silly,
    // but in other cases Deserializers sometimes perform non-obvious mappings.
    // For example the TOML format has a Datetime type and Serde's data model
    // does not. In the `toml` crate, a Datetime in the input is deserialized by
    // mapping it to a Serde data model "struct" type with a special name and a
    // single field containing the Datetime represented as a string.
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_signed()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_signed()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_signed()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_signed()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_unsigned()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_unsigned()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_unsigned()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_unsigned()?)
    }

    // Float parsing is stupidly hard.
    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Float parsing is stupidly hard.
    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Parse a string, check that it is one character, call `visit_char`.
        unimplemented!()
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as cs2 arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.parse_bytes()?)
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // As commented in `Serializer` implementation, this is a lossy
    // representation.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    // In Serde, unit means an anonymous value containing no data.
    // Units are not used in cs2
    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Unit struct means a named value containing no data.
    // Unit structs are not used in cs2
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Give the visitor access to each element of the sequence.
        let value = visitor.visit_seq(NewlineSeparated::new(self))?;
        self.keys.pop();
        Ok(value)
    }

    // Tuples look just like arrays in cs2.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // this is an array in the cs2 data
        // Give the visitor access to each element of the sequence.
        self.input = &self.input.trim_start();
        let value = visitor.visit_seq(SpaceSeparated::new(self))?;
        Ok(value)
    }

    // Tuple structs are not used in cs2.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = visitor.visit_map(NewlineSeparated::new(self))?;
        return Ok(value);
    }

    // Structs start with the struct name in the first line.
    // Followed by all fields in a new line and indented by one level
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // skip new lines in front
        if self.peek_char() == Ok('\n') {
            let _ = self.next_char();
        }
        if !self.input.trim_start().starts_with(name) {
            return Err(Error::ExpectedStructName);
        }
        if let Some(len) = self.input.find('\n') {
            self.input = &self.input[len + 1..];
            // we deserialize the full structure map
            if !self.keys.last().is_some_and(|key| *key == name) {
                self.keys.push(name);
            }
            return self.deserialize_map(visitor);
        }
        Err(Error::ExpectedNewline)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In cs2, struct fields are represented as strings.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

// In order to handle arrays correctly when deserializing a cs2 struct
// we need to track whether we are on the first element or past the first
// element.
struct SpaceSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> SpaceSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        SpaceSeparated {
            de,
            first: true,
        }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for SpaceSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.input.is_empty() {
            return Ok(None);
        }
        // Check if there are no more elements.
        if self.de.peek_char()? == '\n' {
            return Ok(None);
        }
        // Space is required before every element except the first.
        if !self.first && self.de.next_char()? != ' ' {
            return Err(Error::ExpectedArraySeperator);
        }
        self.first = false;
        // Deserialize an array element.
        seed.deserialize(&mut *self.de).map(Some)
    }
}

// In order to handle struct collections correctly when deserializing a cs2 struct
// we need to track whether we are on the first element or past the first
// element.
struct NewlineSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> NewlineSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        NewlineSeparated {
            de,
            first: true,
        }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for NewlineSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.input.is_empty() {
            return Ok(None);
        }
        let str = self.de.input.trim_start_matches('\n').trim_start();

        // Check if there are no more elements.
        let level = str.chars().take_while(|c| *c == '.').count();
        if !self.first && level >= self.de.keys.len() {
            return Err(Error::WrongLevel);
        }

        let str = str.trim_start_matches('.');
        // line is not the expected key for the current sequence so we are at the end of
        // this sequence
        let key = str.split_whitespace().next().unwrap_or("").split('=').next().unwrap_or("");
        if !self.first && key != *self.de.keys.last().unwrap() {
            return Ok(None);
        }
        // remove leading ' ' from the value string
        self.de.input = str.trim_start();
        self.first = false;
        // Deserialize an array element.
        seed.deserialize(&mut *self.de).map(Some)
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for NewlineSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        if self.de.peek_char() == Err(Error::Eof) {
            return Ok(None);
        }

        let str = self.de.input.trim_start_matches('\n').trim_start();
        if str.is_empty() {
            self.de.input = str;
            return Ok(None);
        }

        // Check if we are in the correct level
        let level = str.chars().take_while(|c| *c == '.').count();
        if level < self.de.keys.len() {
            return Ok(None)
        }

        // Newline is required before every entry except the first.
        if !self.first && self.de.next_char()? != '\n' {
            return Err(Error::ExpectedNewline);
        }
        self.first = false;

        // skip white spaces
        self.de.input = self.de.input.trim_start();

        //check for empty strings
        if self.de.input.is_empty() {
            return Ok(None);
        }

        // Deserialize a map key.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // It doesn't make a difference whether the colon is parsed at the end
        // of `next_key_seed` or at the beginning of `next_value_seed`. In this
        // case the code is a bit simpler having it here.
        // value is separated by a '='
        if self.de.peek_char() != Ok('=') {
            // TODO: check if this is a array start
            // we can do this by keeping the. In front of the struct name?
            return seed.deserialize(&mut *self.de);
        }
        if self.de.next_char()? != '=' {
            return Err(Error::ExpectedValueSeperator);
        }
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }
}
