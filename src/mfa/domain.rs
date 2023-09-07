/// Represents a multi factor authentication method.
#[derive(Debug, strum_macros::EnumString, strum_macros::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Mfa {
    /// An external application is used.
    App,
    /// The OTP is send via email.
    Email,
}
