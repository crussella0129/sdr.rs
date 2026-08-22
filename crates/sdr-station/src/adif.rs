//! Minimal ADIF (Amateur Data Interchange Format) record serialization.
//!
//! ADIF encodes each field as `<NAME:LENGTH>VALUE` and terminates a record with
//! `<EOR>`. Only the subset needed to log a contact to Cloudlog is implemented.

/// A single logged contact (QSO) to be serialized as an ADIF record.
#[derive(Debug, Clone, PartialEq)]
pub struct Contact {
    /// Worked station callsign (e.g. `W1AW`).
    pub call: String,
    /// UTC date in ADIF `YYYYMMDD` form (e.g. `20260822`).
    pub qso_date: String,
    /// UTC start time in ADIF `HHMMSS` form (e.g. `001530`).
    pub time_on: String,
    /// Band designator (e.g. `20m`).
    pub band: String,
    /// Mode (e.g. `SSB`, `FT8`, `CW`).
    pub mode: String,
    /// Optional frequency in MHz.
    pub freq_mhz: Option<f64>,
    /// Optional signal report sent.
    pub rst_sent: Option<String>,
    /// Optional signal report received.
    pub rst_rcvd: Option<String>,
    /// Optional free-text comment.
    pub comment: Option<String>,
}

impl Contact {
    /// Construct a contact from the required ADIF fields.
    pub fn new(
        call: impl Into<String>,
        qso_date: impl Into<String>,
        time_on: impl Into<String>,
        band: impl Into<String>,
        mode: impl Into<String>,
    ) -> Self {
        Self {
            call: call.into(),
            qso_date: qso_date.into(),
            time_on: time_on.into(),
            band: band.into(),
            mode: mode.into(),
            freq_mhz: None,
            rst_sent: None,
            rst_rcvd: None,
            comment: None,
        }
    }
}

/// Encode a single ADIF field as `<NAME:LEN>VALUE`.
///
/// `LEN` is the byte length of `value`, which matches the ADIF character count
/// for the ASCII fields used in a log entry.
fn field(name: &str, value: &str) -> String {
    format!("<{}:{}>{}", name.to_uppercase(), value.len(), value)
}

/// Serialize a [`Contact`] into a single-record ADIF string terminated by `<EOR>`.
pub fn to_adif_record(contact: &Contact) -> String {
    let mut out = String::new();
    out.push_str(&field("CALL", &contact.call));
    out.push_str(&field("QSO_DATE", &contact.qso_date));
    out.push_str(&field("TIME_ON", &contact.time_on));
    out.push_str(&field("BAND", &contact.band));
    out.push_str(&field("MODE", &contact.mode));
    if let Some(freq) = contact.freq_mhz {
        out.push_str(&field("FREQ", &format!("{:.6}", freq)));
    }
    if let Some(rst) = &contact.rst_sent {
        out.push_str(&field("RST_SENT", rst));
    }
    if let Some(rst) = &contact.rst_rcvd {
        out.push_str(&field("RST_RCVD", rst));
    }
    if let Some(comment) = &contact.comment {
        out.push_str(&field("COMMENT", comment));
    }
    out.push_str("<EOR>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adif_record_format() {
        let mut c = Contact::new("W1AW", "20260822", "001530", "20m", "SSB");
        c.freq_mhz = Some(14.074);
        c.rst_sent = Some("59".to_string());
        c.rst_rcvd = Some("57".to_string());

        let adif = to_adif_record(&c);

        // Required fields with correct length prefixes.
        assert!(adif.contains("<CALL:4>W1AW"), "got: {adif}");
        assert!(adif.contains("<QSO_DATE:8>20260822"), "got: {adif}");
        assert!(adif.contains("<TIME_ON:6>001530"), "got: {adif}");
        assert!(adif.contains("<BAND:3>20m"), "got: {adif}");
        assert!(adif.contains("<MODE:3>SSB"), "got: {adif}");
        assert!(adif.contains("<RST_SENT:2>59"), "got: {adif}");
        assert!(adif.contains("<RST_RCVD:2>57"), "got: {adif}");
        // Optional freq present and formatted.
        assert!(adif.contains("<FREQ:9>14.074000"), "got: {adif}");
        // Record terminator.
        assert!(adif.ends_with("<EOR>"), "got: {adif}");
    }

    #[test]
    fn test_adif_omits_absent_optionals() {
        let c = Contact::new("K2ABC", "20260822", "010000", "40m", "FT8");
        let adif = to_adif_record(&c);
        assert!(!adif.contains("RST_SENT"));
        assert!(!adif.contains("FREQ"));
        assert!(adif.ends_with("<EOR>"));
    }
}
