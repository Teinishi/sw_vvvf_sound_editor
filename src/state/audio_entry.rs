use super::AudioEntryId;
use crate::func_edit::{EditableFunc, EditablePositiveFunc, EditableZeroOneFunc};
use anyhow::Context as _;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AudioFunctions {
    pub volume: EditableZeroOneFunc,
    pub pitch: EditablePositiveFunc,
}

impl Default for AudioFunctions {
    fn default() -> Self {
        Self {
            volume: EditableFunc::with_points(vec![(40.0, 0.5)]).into(),
            pitch: EditableFunc::with_points(vec![(40.0, 1.0)]).into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundType {
    Accel,
    Brake,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AudioFunctionMode {
    Common(AudioFunctions),
    Separate {
        accel: AudioFunctions,
        brake: Box<AudioFunctions>,
    },
    AccelOnly(AudioFunctions),
    BrakeOnly(AudioFunctions),
}

impl AudioFunctionMode {
    pub const TEXT_COMMON: &str = "Common";
    pub const TEXT_SEPARATE: &str = "Separate";
    pub const TEXT_ACCEL_ONLY: &str = "Accel Only";
    pub const TEXT_BRAKE_ONLY: &str = "Brake Only";

    pub fn is_common(&self) -> bool {
        matches!(self, Self::Common(_))
    }

    pub fn is_separate(&self) -> bool {
        matches!(self, Self::Separate { .. })
    }

    pub fn is_accel_only(&self) -> bool {
        matches!(self, Self::AccelOnly(_))
    }

    pub fn is_brake_only(&self) -> bool {
        matches!(self, Self::BrakeOnly(_))
    }

    pub fn get_by_type(&self, audio_type: SoundType) -> Option<&AudioFunctions> {
        match audio_type {
            SoundType::Accel => self.accel(),
            SoundType::Brake => self.brake(),
        }
    }

    pub fn get_by_type_mut(&mut self, audio_type: SoundType) -> Option<&mut AudioFunctions> {
        match audio_type {
            SoundType::Accel => self.accel_mut(),
            SoundType::Brake => self.brake_mut(),
        }
    }

    pub fn accel(&self) -> Option<&AudioFunctions> {
        match self {
            Self::Separate { accel, .. } => Some(accel),
            Self::AccelOnly(funcs) | Self::Common(funcs) => Some(funcs),
            Self::BrakeOnly(_) => None,
        }
    }

    pub fn accel_mut(&mut self) -> Option<&mut AudioFunctions> {
        match self {
            Self::Separate { accel, .. } => Some(accel),
            Self::AccelOnly(funcs) | Self::Common(funcs) => Some(funcs),
            Self::BrakeOnly(_) => None,
        }
    }

    pub fn brake(&self) -> Option<&AudioFunctions> {
        match self {
            Self::Separate { brake, .. } => Some(brake),
            Self::AccelOnly(_) => None,
            Self::BrakeOnly(funcs) | Self::Common(funcs) => Some(funcs),
        }
    }

    pub fn brake_mut(&mut self) -> Option<&mut AudioFunctions> {
        match self {
            Self::Separate { brake, .. } => Some(brake),
            Self::AccelOnly(_) => None,
            Self::BrakeOnly(funcs) | Self::Common(funcs) => Some(funcs),
        }
    }

    pub fn label_text(&self) -> &str {
        match self {
            Self::Common(_) => Self::TEXT_COMMON,
            Self::Separate { .. } => Self::TEXT_SEPARATE,
            Self::AccelOnly(_) => Self::TEXT_ACCEL_ONLY,
            Self::BrakeOnly(_) => Self::TEXT_BRAKE_ONLY,
        }
    }

    pub fn to_common(&self) -> Self {
        match self {
            Self::Separate { accel, .. } => Self::Common(accel.clone()),
            Self::AccelOnly(funcs) | Self::BrakeOnly(funcs) | Self::Common(funcs) => {
                Self::Common(funcs.clone())
            }
        }
    }

    pub fn to_separate(&self) -> Self {
        match self {
            Self::Separate { accel, brake } => Self::Separate {
                accel: accel.clone(),
                brake: brake.clone(),
            },
            Self::AccelOnly(funcs) | Self::Common(funcs) | Self::BrakeOnly(funcs) => {
                Self::Separate {
                    accel: funcs.clone(),
                    brake: Box::new(funcs.clone()),
                }
            }
        }
    }

    pub fn to_accel_only(&self) -> Self {
        match self {
            Self::Separate { accel, .. } => Self::AccelOnly(accel.clone()),
            Self::AccelOnly(funcs) | Self::BrakeOnly(funcs) | Self::Common(funcs) => {
                Self::AccelOnly(funcs.clone())
            }
        }
    }

    pub fn to_brake_only(&self) -> Self {
        match self {
            Self::Separate { brake, .. } => Self::BrakeOnly(*brake.clone()),
            Self::AccelOnly(funcs) | Self::Common(funcs) | Self::BrakeOnly(funcs) => {
                Self::BrakeOnly(funcs.clone())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AudioEntry {
    #[serde(skip)]
    pub(super) id: AudioEntryId,
    pub(super) path: Option<PathBuf>,
    name: String,
    mode: AudioFunctionMode,
}

impl AudioEntry {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            path: None,
            name: name.to_owned(),
            mode: AudioFunctionMode::Common(AudioFunctions::default()),
        }
    }

    pub fn with_path(id: u32, path: PathBuf) -> anyhow::Result<Self> {
        let without_extension = path.with_extension("");
        let name = without_extension
            .file_name()
            .and_then(|n| n.to_str())
            .context("Unable to get name of file")?;
        Ok(Self {
            id,
            path: Some(path),
            name: name.to_owned(),
            mode: AudioFunctionMode::Common(AudioFunctions::default()),
        })
    }

    pub fn id(&self) -> &AudioEntryId {
        &self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn mode(&self) -> &AudioFunctionMode {
        &self.mode
    }

    pub fn mode_mut(&mut self) -> &mut AudioFunctionMode {
        &mut self.mode
    }

    pub fn funcs_by_type(&self, sound_type: SoundType) -> Option<&AudioFunctions> {
        self.mode.get_by_type(sound_type)
    }

    pub fn funcs_by_type_mut(&mut self, sound_type: SoundType) -> Option<&mut AudioFunctions> {
        self.mode.get_by_type_mut(sound_type)
    }
}
