//use std::error::Error;
use std::io;
use std::fmt;
use serde::ser::Impossible;
use serde::{ser, Serialize};
//use write::Writer;
//use parse::Item;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Copy, Clone, Debug)]
pub enum UnsupportedType {
    Map,
    NewtypeStruct,
    NewtypeVariant,
    Unit,
    UnitStruct,
    UnitVariant,
}

#[derive(Debug)]
pub enum Error {
    /// Serialization error
    ///
    /// Passed through error message from the type being serialized.
    Custom(String),

    /// Attempted to serialize a type not supported by the cs2 format
    UnsupportedType(UnsupportedType),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Custom(e.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Custom(msg) => write!(f, "{}", msg),
            Error::UnsupportedType(ty) => write!(f, "{:?} cannot be serialized into cs2", ty),
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        "cs2 serialization error"
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

pub struct Serializer {
    level: usize,
    output: String,
}

impl Default for Serializer{
    fn default() -> Self {
        Self { level: Default::default(), output: Default::default() }
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), Error>;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to cs2 by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output += if v { "1" } else { "0" };
        Ok(())
    }

    // cs2 does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea.
    fn serialize_str(self, v: &str) -> Result<()> {
        self.output += v;
        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    // This is used for hexadecimal values
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    // An absent optional is is empty
    fn serialize_none(self) -> Result<()> {
        Ok(())
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data.
    // Unit is not used in cs2
    fn serialize_unit(self) -> Result<()> {
        Err(Error::UnsupportedType(UnsupportedType::Unit))
    }

    // Unit struct means a named value containing no data.
    // Unit struct is not used in cs2
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::UnsupportedType(UnsupportedType::UnitStruct))
    }

    // Unit variant is not used in cs2
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        Err(Error::UnsupportedType(UnsupportedType::UnitVariant))
    }

    // Tuple newtype struct is not used in cs2
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType(UnsupportedType::NewtypeStruct))
    }

    // Tuple newtype variant is not used in cs2
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType(UnsupportedType::NewtypeVariant))
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in cs2 is the struct name.
    //
    // The length of the sequence is not known ahead of time.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    // Tuples are arrays in the cs2 format. The values are blank separated in
    // one line.
    // Some formats may be able to represent tuples more efficiently by omitting
    // the length, since tuple  means that the corresponding `Deserialize implementation
    // will know the length without needing to look at the serialized data.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    // Tuple structs are not used in cs2
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unimplemented!()
    }

    // Tuple variants are not used in cs2 format.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!()
    }

    // Maps are not used in cs2 format
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!()
    }

    // Structs in cs2 start with just the struct name in one line, followed
    // by all fields in separate lines.
    // each struct starts a new '.' indention level to be able to map the
    // fields to the correct struct.
    // Each field has name, separator '=', value, '\n'
    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        if self.output.ends_with("=") {
            //remove the = at the end since this is a nested struct name
            let _ = self.output.pop();
        }
        if !self.output.ends_with(name) {
            if self.level > 0 {
                self.output += "\n ";
                for _ in 0..self.level {
                    self.output += ".";
                }
            } else if !self.output.is_empty() && !self.output.ends_with("\n") {
                self.output += "\n";
            }
            if !(self.level == 0 && name.starts_with("[") && name.ends_with("]")) {
                self.level +=1;
            }
            self.output += name;
        } else {
            if !self.output.is_empty() && !self.output.ends_with("\n") {
                self.output += "\n";
            }
            // we wrote the own name already as part of the field name
            // increase only indention level
            self.level +=1;
        }
        Ok(self)
    }

    // struct variants are not used in cs2 format
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!()
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a> ser::SerializeSeq for &'a mut Serializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('=') {
            // array separator
            self.output += " ";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Structs are newline separated fields indented by '.'
// values are separated by '='
impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with("\n") {
            self.output += "\n";
        }

        // indent based on the current level
        if self.level > 0 {
            self.output += " ";
            for _ in 0..self.level {
                self.output += ".";
            }
        }
        key.serialize(&mut **self)?;
        self.output += "=";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        // end of the struct, decrease level
        self.level = self.level.saturating_sub(1);
        Ok(())
    }
}

// By convention, the public API of a Serde serializer is one or more `to_abc`
// functions such as `to_string`, `to_bytes`, or `to_writer` depending on what
// Rust types the serializer is able to produce as output.
//
// This basic serializer supports only `to_string`.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let mut serializer = Serializer::default();

    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}
