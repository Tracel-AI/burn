use super::*;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{de::DeserializeOwned, Serialize};
use std::{fs::File, path::PathBuf};

/// Recorder trait specialized to save and load data to and from files.
pub trait FileRecorder:
    Recorder<RecordArgs = PathBuf, RecordOutput = (), LoadArgs = PathBuf>
{
}

/// File recorder using the [bincode format](bincode).
pub struct FileBinRecorder;
/// File recorder using the [bincode format](bincode) compressed with gzip.
pub struct FileBinGzRecorder;
/// File recorder using the json format compressed with gzip.
pub struct FileJsonGzRecorder;

#[cfg(feature = "msgpack")]
/// File recorder using the [message pack](rmp_serde) format compressed with gzip.
pub struct FileMpkGzRecorder;

impl FileRecorder for FileBinGzRecorder {}
impl FileRecorder for FileBinRecorder {}
impl FileRecorder for FileJsonGzRecorder {}

#[cfg(feature = "msgpack")]
impl FileRecorder for FileMpkGzRecorder {}

macro_rules! str2reader {
    (
        $file:expr,
        $ext:expr
    ) => {{
        $file.set_extension($ext);
        let path = $file.as_path();

        File::open(path).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => RecordError::FileNotFound(err.to_string()),
            _ => RecordError::Unknown(err.to_string()),
        })
    }};
}

macro_rules! str2writer {
    (
        $file:expr,
        $ext:expr
    ) => {{
        $file.set_extension($ext);
        let path = $file.as_path();

        if path.exists() {
            log::info!("File exists, replacing");
            std::fs::remove_file(path).unwrap();
        }

        File::create(path).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => RecordError::FileNotFound(err.to_string()),
            _ => RecordError::Unknown(err.to_string()),
        })
    }};
}

impl Recorder for FileBinGzRecorder {
    type RecordArgs = PathBuf;
    type RecordOutput = ();
    type LoadArgs = PathBuf;

    fn record<Obj: Serialize + DeserializeOwned>(
        obj: Obj,
        mut file: PathBuf,
    ) -> Result<(), RecordError> {
        let config = bin_config();
        let writer = str2writer!(file, "bin.gz")?;
        let mut writer = GzEncoder::new(writer, Compression::default());

        bincode::serde::encode_into_std_write(&obj, &mut writer, config).unwrap();

        Ok(())
    }

    fn load<Obj: Serialize + DeserializeOwned>(mut file: PathBuf) -> Result<Obj, RecordError> {
        let reader = str2reader!(file, "bin.gz")?;
        let mut reader = GzDecoder::new(reader);
        let state = bincode::serde::decode_from_std_read(&mut reader, bin_config()).unwrap();

        Ok(state)
    }
}

impl Recorder for FileBinRecorder {
    type RecordArgs = PathBuf;
    type RecordOutput = ();
    type LoadArgs = PathBuf;

    fn record<Obj: Serialize + DeserializeOwned>(
        obj: Obj,
        mut file: PathBuf,
    ) -> Result<(), RecordError> {
        let config = bin_config();
        let mut writer = str2writer!(file, "bin")?;
        bincode::serde::encode_into_std_write(&obj, &mut writer, config).unwrap();
        Ok(())
    }

    fn load<Obj: Serialize + DeserializeOwned>(mut file: PathBuf) -> Result<Obj, RecordError> {
        let mut reader = str2reader!(file, "bin")?;
        let state = bincode::serde::decode_from_std_read(&mut reader, bin_config()).unwrap();
        Ok(state)
    }
}

impl Recorder for FileJsonGzRecorder {
    type RecordArgs = PathBuf;
    type RecordOutput = ();
    type LoadArgs = PathBuf;

    fn record<Obj: Serialize + DeserializeOwned>(
        obj: Obj,
        mut file: PathBuf,
    ) -> Result<(), RecordError> {
        let writer = str2writer!(file, "json.gz")?;
        let writer = GzEncoder::new(writer, Compression::default());
        serde_json::to_writer(writer, &obj).unwrap();

        Ok(())
    }

    fn load<Obj: Serialize + DeserializeOwned>(mut file: PathBuf) -> Result<Obj, RecordError> {
        let reader = str2reader!(file, "json.gz")?;
        let reader = GzDecoder::new(reader);
        let state = serde_json::from_reader(reader).unwrap();

        Ok(state)
    }
}

#[cfg(feature = "msgpack")]
impl Recorder for FileMpkGzRecorder {
    type SaveArgs = PathBuf;
    type SaveOutput = ();
    type LoadArgs = PathBuf;

    fn save<Obj: Serialize + DeserializeOwned>(
        obj: Obj,
        mut file: PathBuf,
    ) -> Result<(), RecordError> {
        let writer = str2writer!(file, "mpk.gz")?;
        let mut writer = GzEncoder::new(writer, Compression::default());
        rmp_serde::encode::write(&mut writer, &obj).unwrap();

        Ok(())
    }

    fn load<Obj: Serialize + DeserializeOwned>(mut file: PathBuf) -> Result<Obj, RecordError> {
        let reader = str2reader!(file, "mpk.gz")?;
        let reader = GzDecoder::new(reader);
        let state = rmp_serde::decode::from_read(reader).unwrap();

        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{module::Module, nn, TestBackend};

    static FILE_PATH: &str = "/tmp/test_state";

    #[test]
    fn test_can_save_and_load_jsongz_format() {
        test_can_save_and_load::<FileJsonGzRecorder>()
    }

    #[test]
    fn test_can_save_and_load_bin_format() {
        test_can_save_and_load::<FileBinRecorder>()
    }

    #[test]
    fn test_can_save_and_load_bingz_format() {
        test_can_save_and_load::<FileBinGzRecorder>()
    }

    #[cfg(feature = "msgpack")]
    #[test]
    fn test_can_save_and_load_mpkgz_format() {
        test_can_save_and_load::<FileMpkGzRecorder>()
    }

    fn test_can_save_and_load<Recorder: FileRecorder>() {
        let model_before = create_model();
        let state_before = model_before.state();
        Recorder::record(state_before.clone(), FILE_PATH.into()).unwrap();

        let model_after = create_model()
            .load(&Recorder::load(FILE_PATH.into()).unwrap())
            .unwrap();

        let state_after = model_after.state();
        assert_eq!(state_before, state_after);
    }

    pub fn create_model() -> nn::Linear<TestBackend> {
        nn::LinearConfig::new(32, 32).with_bias(true).init()
    }
}
