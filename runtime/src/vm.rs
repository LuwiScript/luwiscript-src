use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Struct(StructInstance),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructInstance {
    pub name: String,
    pub fields: HashMap<String, Value>,
}

impl StructInstance {
    pub fn new(name: impl Into<String>, fields: HashMap<String, Value>) -> Self {
        Self {
            name: name.into(),
            fields,
        }
    }

    pub fn get_field(&self, field_name: &str) -> Option<&Value> {
        self.fields.get(field_name)
    }

    pub fn set_field(&mut self, field_name: &str, value: Value) -> Result<(), String> {
        if !self.fields.contains_key(field_name) {
            return Err(format!(
                "struct '{}' has no field '{}'",
                self.name, field_name
            ));
        }

        self.fields.insert(field_name.to_string(), value);
        Ok(())
    }
}

#[derive(Default)]
pub struct Vm;

impl Vm {
    pub fn new() -> Self {
        Self
    }

    pub fn make_struct(
        &self,
        struct_name: impl Into<String>,
        fields: HashMap<String, Value>,
    ) -> Value {
        Value::Struct(StructInstance::new(struct_name, fields))
    }

    pub fn member_get<'a>(&self, target: &'a Value, field_name: &str) -> Result<&'a Value, String> {
        match target {
            Value::Struct(instance) => instance.get_field(field_name).ok_or_else(|| {
                format!("struct '{}' has no field '{}'", instance.name, field_name)
            }),
            other => Err(format!("cannot access field '{}' on {}", field_name, other.type_name())),
        }
    }

    pub fn member_set(
        &self,
        target: &mut Value,
        field_name: &str,
        value: Value,
    ) -> Result<(), String> {
        match target {
            Value::Struct(instance) => instance.set_field(field_name, value),
            other => Err(format!("cannot set field '{}' on {}", field_name, other.type_name())),
        }
    }
}

impl Value {
    fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Struct(_) => "struct",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_struct_and_reads_fields() {
        let vm = Vm::new();
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::String("Lucas".to_string()));
        fields.insert("age".to_string(), Value::Int(23));

        let person = vm.make_struct("Person", fields);
        assert_eq!(vm.member_get(&person, "name"), Ok(&Value::String("Lucas".to_string())));
        assert_eq!(vm.member_get(&person, "age"), Ok(&Value::Int(23)));
    }

    #[test]
    fn updates_existing_field() {
        let vm = Vm::new();
        let mut fields = HashMap::new();
        fields.insert("count".to_string(), Value::Int(1));
        let mut counter = vm.make_struct("Counter", fields);

        vm.member_set(&mut counter, "count", Value::Int(2)).unwrap();

        assert_eq!(vm.member_get(&counter, "count"), Ok(&Value::Int(2)));
    }

    #[test]
    fn errors_when_field_is_missing() {
        let vm = Vm::new();
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), Value::Int(10));
        let mut user = vm.make_struct("User", fields);

        let read_err = vm.member_get(&user, "email").unwrap_err();
        assert!(read_err.contains("has no field 'email'"));

        let write_err = vm.member_set(&mut user, "email", Value::String("x".into())).unwrap_err();
        assert!(write_err.contains("has no field 'email'"));
    }
}
