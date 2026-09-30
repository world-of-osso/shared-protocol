//! Describes a type's serde wire layout by deserializing it from a tracer. Each pass walks
//! the whole type, visiting one element of every sequence, option and map; an enum yields
//! its next unvisited variant, so passes repeat until every reachable variant was visited.
//! Primitives read as zero or empty, so a `Deserialize` that validates them fails the trace.

use serde::Deserialize;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use std::fmt::{self, Write};

/// Enough for every variant of nested enums; reaching it means the tracer is stuck.
const MAX_PASSES: usize = 10_000;

#[derive(Debug)]
pub struct TraceError(String);

impl fmt::Display for TraceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TraceError {}

impl de::Error for TraceError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

type Result<T> = core::result::Result<T, TraceError>;

/// One enum, told apart from same-named enums by its `'static` variant list.
struct EnumLayout {
    name: &'static str,
    variants_ptr: usize,
    variant_names: &'static [&'static str],
    shapes: Vec<Option<String>>,
    cursor: usize,
}

impl EnumLayout {
    fn next_variant(&mut self) -> usize {
        let unvisited = self.shapes.iter().position(Option::is_none);
        let variant = unvisited.unwrap_or(self.cursor % self.shapes.len());
        self.cursor += 1;
        variant
    }
}

#[derive(Default)]
struct Tracer {
    /// In first-encounter order, which is the same in every build of the same types.
    enums: Vec<EnumLayout>,
    /// Containers being traced, to stop on a recursive type instead of overflowing.
    stack: Vec<(&'static str, usize)>,
}

/// `T`'s layout: its shape with every enum it reaches and each enum's variant shapes.
pub fn trace_layout<'de, T: Deserialize<'de>>() -> Result<String> {
    let mut tracer = Tracer::default();
    let mut shape = String::new();
    for _ in 0..MAX_PASSES {
        shape.clear();
        T::deserialize(ShapeDeserializer {
            tracer: &mut tracer,
            out: &mut shape,
        })?;
        let complete = tracer
            .enums
            .iter()
            .all(|layout| layout.shapes.iter().all(Option::is_some));
        if complete {
            return Ok(describe(&shape, &tracer.enums));
        }
    }
    Err(TraceError(format!(
        "enum variants still unvisited after {MAX_PASSES} passes"
    )))
}

fn describe(shape: &str, enums: &[EnumLayout]) -> String {
    let mut out = shape.to_owned();
    for (id, layout) in enums.iter().enumerate() {
        let _ = write!(out, "; enum#{id} {}", layout.name);
        for (variant, shape) in layout.variant_names.iter().zip(&layout.shapes) {
            let _ = write!(out, " |{variant}{}", shape.as_deref().unwrap_or("?"));
        }
    }
    out
}

impl Tracer {
    fn enter(&mut self, name: &'static str, id: usize) -> Result<()> {
        if self.stack.contains(&(name, id)) {
            return Err(TraceError(format!(
                "recursive type {name} is not supported"
            )));
        }
        self.stack.push((name, id));
        Ok(())
    }

    fn enum_index(&mut self, name: &'static str, variants: &'static [&'static str]) -> usize {
        let variants_ptr = variants.as_ptr() as usize;
        let found = self
            .enums
            .iter()
            .position(|layout| layout.variants_ptr == variants_ptr && layout.name == name);
        found.unwrap_or_else(|| {
            self.enums.push(EnumLayout {
                name,
                variants_ptr,
                variant_names: variants,
                shapes: vec![None; variants.len()],
                cursor: 0,
            });
            self.enums.len() - 1
        })
    }
}

struct ShapeDeserializer<'a> {
    tracer: &'a mut Tracer,
    out: &'a mut String,
}

impl<'a> ShapeDeserializer<'a> {
    fn reborrow(&mut self) -> ShapeDeserializer<'_> {
        ShapeDeserializer {
            tracer: self.tracer,
            out: self.out,
        }
    }

    fn primitive(self, name: &str) {
        self.out.push_str(name);
    }

    /// Visits the elements, e.g. struct fields written as `field:shape`, inside `open` and `)`.
    fn elements<'de, V: Visitor<'de>>(
        &mut self,
        open: &str,
        labels: Labels,
        visitor: V,
    ) -> Result<V::Value> {
        self.out.push_str(open);
        let value = visitor.visit_seq(Elements {
            tracer: self.tracer,
            out: self.out,
            labels,
            index: 0,
        })?;
        self.out.push(')');
        Ok(value)
    }

    /// A named container, refused when it contains itself.
    fn container<'de, V: Visitor<'de>>(
        mut self,
        name: &'static str,
        id: usize,
        labels: Labels,
        visitor: V,
    ) -> Result<V::Value> {
        self.tracer.enter(name, id)?;
        let value = self.elements(&format!("{name}("), labels, visitor)?;
        self.tracer.stack.pop();
        Ok(value)
    }
}

enum Labels {
    Fields(&'static [&'static str]),
    Count(usize),
}

impl Labels {
    fn len(&self) -> usize {
        match self {
            Self::Fields(fields) => fields.len(),
            Self::Count(len) => *len,
        }
    }
}

struct Elements<'a> {
    tracer: &'a mut Tracer,
    out: &'a mut String,
    labels: Labels,
    index: usize,
}

impl<'de> SeqAccess<'de> for Elements<'_> {
    type Error = TraceError;

    fn next_element_seed<S: DeserializeSeed<'de>>(&mut self, seed: S) -> Result<Option<S::Value>> {
        if self.index == self.labels.len() {
            return Ok(None);
        }
        if self.index > 0 {
            self.out.push(',');
        }
        if let Labels::Fields(fields) = self.labels {
            let _ = write!(self.out, "{}:", fields[self.index]);
        }
        self.index += 1;
        seed.deserialize(ShapeDeserializer {
            tracer: self.tracer,
            out: self.out,
        })
        .map(Some)
    }
}

/// One key and one value.
struct Entry<'a> {
    tracer: &'a mut Tracer,
    out: &'a mut String,
    done: bool,
}

impl<'de> MapAccess<'de> for Entry<'_> {
    type Error = TraceError;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if self.done {
            return Ok(None);
        }
        self.done = true;
        seed.deserialize(ShapeDeserializer {
            tracer: self.tracer,
            out: self.out,
        })
        .map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        self.out.push_str("=>");
        seed.deserialize(ShapeDeserializer {
            tracer: self.tracer,
            out: self.out,
        })
    }
}

struct Variant<'a> {
    tracer: &'a mut Tracer,
    out: &'a mut String,
    index: u32,
}

impl<'de, 'a> EnumAccess<'de> for Variant<'a> {
    type Error = TraceError;
    type Variant = Self;

    fn variant_seed<S: DeserializeSeed<'de>>(self, seed: S) -> Result<(S::Value, Self)> {
        let index: de::value::U32Deserializer<TraceError> = self.index.into_deserializer();
        Ok((seed.deserialize(index)?, self))
    }
}

impl<'de> VariantAccess<'de> for Variant<'_> {
    type Error = TraceError;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<S: DeserializeSeed<'de>>(self, seed: S) -> Result<S::Value> {
        self.out.push('(');
        let value = seed.deserialize(ShapeDeserializer {
            tracer: self.tracer,
            out: self.out,
        })?;
        self.out.push(')');
        Ok(value)
    }

    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        ShapeDeserializer {
            tracer: self.tracer,
            out: self.out,
        }
        .elements("(", Labels::Count(len), visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        ShapeDeserializer {
            tracer: self.tracer,
            out: self.out,
        }
        .elements("(", Labels::Fields(fields), visitor)
    }
}

macro_rules! primitive {
    ($($method:ident => $visit:ident($($value:expr)?), $name:literal;)*) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
                self.primitive($name);
                visitor.$visit($($value)?)
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for ShapeDeserializer<'_> {
    type Error = TraceError;

    primitive! {
        deserialize_bool => visit_bool(false), "bool";
        deserialize_i8 => visit_i8(0), "i8";
        deserialize_i16 => visit_i16(0), "i16";
        deserialize_i32 => visit_i32(0), "i32";
        deserialize_i64 => visit_i64(0), "i64";
        deserialize_i128 => visit_i128(0), "i128";
        deserialize_u8 => visit_u8(0), "u8";
        deserialize_u16 => visit_u16(0), "u16";
        deserialize_u32 => visit_u32(0), "u32";
        deserialize_u64 => visit_u64(0), "u64";
        deserialize_u128 => visit_u128(0), "u128";
        deserialize_f32 => visit_f32(0.0), "f32";
        deserialize_f64 => visit_f64(0.0), "f64";
        deserialize_char => visit_char('a'), "char";
        deserialize_str => visit_str(""), "str";
        deserialize_string => visit_string(String::new()), "str";
        deserialize_bytes => visit_bytes(&[]), "bytes";
        deserialize_byte_buf => visit_byte_buf(Vec::new()), "bytes";
        deserialize_unit => visit_unit(), "()";
    }

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(TraceError(
            "self-describing deserialization (deserialize_any) is not supported".into(),
        ))
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(0)
    }

    fn deserialize_option<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        self.out.push_str("Option(");
        let value = visitor.visit_some(self.reborrow())?;
        self.out.push(')');
        Ok(value)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.out.push_str(name);
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        mut self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.tracer.enter(name, 0)?;
        let _ = write!(self.out, "{name}(");
        let value = visitor.visit_newtype_struct(self.reborrow())?;
        self.out.push(')');
        self.tracer.stack.pop();
        Ok(value)
    }

    fn deserialize_seq<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        self.elements("Seq(", Labels::Count(1), visitor)
    }

    fn deserialize_tuple<V: Visitor<'de>>(mut self, len: usize, visitor: V) -> Result<V::Value> {
        self.elements(&format!("Tuple{len}("), Labels::Count(len), visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.container(name, len, Labels::Count(len), visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.out.push_str("Map(");
        let value = visitor.visit_map(Entry {
            tracer: self.tracer,
            out: self.out,
            done: false,
        })?;
        self.out.push(')');
        Ok(value)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.container(
            name,
            fields.as_ptr() as usize,
            Labels::Fields(fields),
            visitor,
        )
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        if variants.is_empty() {
            return Err(TraceError(format!("enum {name} has no variants")));
        }
        let id = self.tracer.enum_index(name, variants);
        self.tracer.enter(name, variants.as_ptr() as usize)?;
        let _ = write!(self.out, "enum#{id}");
        let variant = self.tracer.enums[id].next_variant();
        let mut shape = String::new();
        let value = visitor.visit_enum(Variant {
            tracer: self.tracer,
            out: &mut shape,
            index: variant as u32,
        })?;
        self.tracer.stack.pop();
        self.tracer.enums[id].shapes[variant].get_or_insert(shape);
        Ok(value)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}
