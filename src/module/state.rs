use crate::tensor::{DataSerialize, Element};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::io::{Read, Write};

#[derive(Debug, PartialEq)]
pub struct StateNamed<E> {
    pub values: HashMap<String, State<E>>,
}

#[derive(Debug, PartialEq)]
pub enum State<E> {
    StateNamed(StateNamed<E>),
    Data(DataSerialize<E>),
}

#[derive(Debug)]
pub enum StateError {
    InvalidFormat(String),
    FileNotFound(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut message = "State error => ".to_string();

        match self {
            Self::InvalidFormat(err) => {
                message += format!("Invalid format: {}", err).as_str();
            }
            Self::FileNotFound(err) => {
                message += format!("File not found: {}", err).as_str();
            }
        };

        f.write_str(message.as_str())
    }
}
impl std::error::Error for StateError {}

impl<E: Element> Into<serde_json::Value> for State<E>
where
    E: serde::de::DeserializeOwned,
    E: serde::Serialize,
{
    fn into(self) -> serde_json::Value {
        match self {
            Self::StateNamed(state) => state.into(),
            Self::Data(data) => serde_json::to_value(data).unwrap(),
        }
    }
}

impl<E: Element> Into<serde_json::Value> for StateNamed<E>
where
    E: serde::de::DeserializeOwned,
    E: serde::Serialize,
{
    fn into(self) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        for (key, state) in self.values {
            map.insert(key, state.into());
        }

        serde_json::Value::Object(map)
    }
}

impl<E> TryFrom<serde_json::Value> for State<E>
where
    E: serde::de::DeserializeOwned,
    E: serde::Serialize,
{
    type Error = StateError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        match serde_json::from_value(value.clone()) {
            Ok(data) => Ok(State::Data(data)),
            Err(_) => Ok(State::StateNamed(StateNamed::try_from(value)?)),
        }
    }
}

impl<E> TryFrom<serde_json::Value> for StateNamed<E>
where
    E: serde::de::DeserializeOwned,
    E: serde::Serialize,
{
    type Error = StateError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let map = match value {
            serde_json::Value::Object(map) => map,
            _ => {
                return Err(StateError::InvalidFormat(format!(
                    "Invalid value {:?}",
                    value
                )))
            }
        };

        let mut values = HashMap::new();
        for (key, value) in map {
            values.insert(key, State::try_from(value)?);
        }

        Ok(Self { values })
    }
}

impl<E: Element> StateNamed<E> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn register_state(&mut self, name: &str, state: State<E>) {
        self.values.insert(name.to_string(), state);
    }
}

impl<E: Element> StateNamed<E> {
    pub fn get(&self, name: &str) -> Option<&State<E>> {
        self.values.get(name)
    }

    pub fn convert<O: Element>(self) -> StateNamed<O> {
        let mut values = HashMap::with_capacity(self.values.len());

        for (key, value) in self.values {
            values.insert(key, value.convert());
        }

        StateNamed { values }
    }
}

impl<E: Element> State<E> {
    pub fn get(&self, name: &str) -> Option<&Self> {
        match self {
            State::StateNamed(named) => named.get(name),
            _ => None,
        }
    }

    pub fn convert<O: Element>(self) -> State<O> {
        match self {
            State::StateNamed(named) => State::StateNamed(named.convert()),
            State::Data(data) => State::Data(data.convert()),
        }
    }
}

impl<E: Element> State<E>
where
    E: serde::de::DeserializeOwned,
    E: serde::Serialize,
{
    pub fn save(self, file: &str) -> std::io::Result<()> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        let value: serde_json::Value = self.into();

        let content = value.to_string();
        encoder.write_all(content.as_bytes()).unwrap();
        let content_compressed = encoder.finish().unwrap();

        std::fs::write(file, content_compressed)
    }

    pub fn load(file: &str) -> Result<Self, StateError> {
        let content_compressed =
            std::fs::read(file).map_err(|err| StateError::FileNotFound(format!("{:?}", err)))?;

        let mut decoder = GzDecoder::new(content_compressed.as_slice());
        let mut content = String::new();
        decoder.read_to_string(&mut content).unwrap();

        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|err| StateError::InvalidFormat(format!("{:?}", err)))?;
        Self::try_from(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::Module;
    use crate::nn;
    use crate::tensor::back;

    #[test]
    fn test_state_to_from_value() {
        let linear = nn::Linear::<crate::TestBackend>::new(&nn::LinearConfig {
            d_input: 32,
            d_output: 32,
            bias: true,
        });

        let state = linear.state();
        let value: serde_json::Value = state.into();
        let state_from: State<<crate::TestBackend as back::Backend>::Elem> =
            State::try_from(value.clone()).unwrap();
        let value_from: serde_json::Value = state_from.into();

        assert_eq!(value, value_from);
    }

    #[test]
    fn test_can_save_and_load_from_file() {
        let mut linear = nn::Linear::<crate::TestBackend>::new(&nn::LinearConfig {
            d_input: 32,
            d_output: 32,
            bias: true,
        });
        linear.state().save("/tmp/test.json").unwrap();
        linear
            .load(&State::load("/tmp/test.json").unwrap())
            .unwrap();
    }
}
