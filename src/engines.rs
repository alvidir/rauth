use base64::{
    alphabet,
    engine::{self, general_purpose},
};

pub const B64: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
