//! Jurisdictional RF Regulatory Band Advisor and Transmission Compliance Engine.

/// Geographic Jurisdiction for telecommunications and RF spectrum rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jurisdiction {
    US,     // United States (FCC Part 15 / Part 97)
    EU,     // European Union (CEPT ERC Rec 70-03 / ETSI)
    UK,     // United Kingdom (Ofcom IR 2030)
    AU,     // Australia (ACMA LIPD Class License)
    Global, // Worldwide ITU ISM allocations (2.4 GHz, 5.8 GHz)
}

impl Jurisdiction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Jurisdiction::US => "US (FCC)",
            Jurisdiction::EU => "EU (CEPT/ETSI)",
            Jurisdiction::UK => "UK (Ofcom)",
            Jurisdiction::AU => "AU (ACMA)",
            Jurisdiction::Global => "Global (ITU)",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "US" | "USA" | "FCC" => Some(Jurisdiction::US),
            "EU" | "EUROPE" | "CEPT" | "ETSI" => Some(Jurisdiction::EU),
            "UK" | "GB" | "OFCOM" => Some(Jurisdiction::UK),
            "AU" | "AUSTRALIA" | "ACMA" => Some(Jurisdiction::AU),
            "GLOBAL" | "WORLD" | "ITU" => Some(Jurisdiction::Global),
            _ => None,
        }
    }
}

/// Regulatory Band Service Classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandType {
    /// License-exempt Industrial, Scientific, and Medical band (e.g. 915 MHz US, 2.4 GHz).
    Ism,
    /// License-exempt Short Range Device band (e.g. 868 MHz EU, 433 MHz EU).
    Srd,
    /// Amateur Radio Service (e.g. 2m 144 MHz, 70cm 430/440 MHz).
    Amateur,
    /// Commercial or Specialized Licensed Service.
    Licensed,
}

/// A legally classified RF frequency band.
#[derive(Debug, Clone)]
pub struct RegulatoryBand {
    pub name: &'static str,
    pub jurisdiction: Jurisdiction,
    pub band_type: BandType,
    pub start_freq_hz: u64,
    pub end_freq_hz: u64,
    pub max_power_dbm: f32, // Conducted or EIRP limit in dBm
    pub max_bandwidth_hz: Option<u32>,
    pub max_duty_cycle_pct: Option<f32>,
    pub encryption_permitted: bool,
    pub citation: &'static str,
}

/// Verification outcome of an RF transmission plan.
#[derive(Debug, Clone, PartialEq)]
pub enum ComplianceResult {
    Compliant {
        band_name: &'static str,
        citation: &'static str,
        warnings: Vec<String>,
    },
    NonCompliant {
        reasons: Vec<String>,
    },
}

/// Database of national and international RF regulations.
pub struct RegulatoryDatabase;

impl RegulatoryDatabase {
    /// List all cataloged regulatory bands.
    pub fn all_bands() -> &'static [RegulatoryBand] {
        &REGULATORY_BANDS
    }

    /// Query legal frequency bands matching jurisdiction and encryption requirements.
    pub fn query_recommended_bands(
        jurisdiction: Jurisdiction,
        require_encryption: bool,
    ) -> Vec<&'static RegulatoryBand> {
        REGULATORY_BANDS
            .iter()
            .filter(|b| {
                (b.jurisdiction == jurisdiction || b.jurisdiction == Jurisdiction::Global)
                    && (!require_encryption || b.encryption_permitted)
            })
            .collect()
    }

    /// Verify whether a planned transmission is legally compliant.
    pub fn check_compliance(
        jurisdiction: Jurisdiction,
        freq_hz: u64,
        power_dbm: f32,
        is_encrypted: bool,
    ) -> ComplianceResult {
        let matching_bands: Vec<&RegulatoryBand> = REGULATORY_BANDS
            .iter()
            .filter(|b| {
                (b.jurisdiction == jurisdiction || b.jurisdiction == Jurisdiction::Global)
                    && freq_hz >= b.start_freq_hz
                    && freq_hz <= b.end_freq_hz
            })
            .collect();

        if matching_bands.is_empty() {
            return ComplianceResult::NonCompliant {
                reasons: vec![format!(
                    "Frequency {:.3} MHz is not in any cataloged license-exempt or amateur band for {:?}",
                    freq_hz as f64 / 1e6,
                    jurisdiction
                )],
            };
        }

        for band in matching_bands {
            let mut reasons = Vec::new();
            let mut warnings = Vec::new();

            // Check encryption legality (strictly forbidden on Amateur bands by telecommunications law)
            if is_encrypted && !band.encryption_permitted {
                reasons.push(format!(
                    "ENCRYPTION PROHIBITED: Frequency {:.3} MHz is in the {} band ({}) where transmitting encrypted payloads (like SSH/TLS) violates telecommunications law ({})",
                    freq_hz as f64 / 1e6,
                    band.name,
                    if band.band_type == BandType::Amateur { "Amateur Radio Service" } else { "Unpermitted Service" },
                    band.citation
                ));
            }

            // Check power limit
            if power_dbm > band.max_power_dbm {
                reasons.push(format!(
                    "Power {:.1} dBm exceeds legal limit of {:.1} dBm ({})",
                    power_dbm, band.max_power_dbm, band.citation
                ));
            }

            // Check duty cycle warning
            if let Some(dc) = band.max_duty_cycle_pct {
                warnings.push(format!(
                    "Duty cycle restriction: max {:.1}% transmission time ({})",
                    dc, band.citation
                ));
            }

            if reasons.is_empty() {
                return ComplianceResult::Compliant {
                    band_name: band.name,
                    citation: band.citation,
                    warnings,
                };
            } else {
                return ComplianceResult::NonCompliant { reasons };
            }
        }

        ComplianceResult::NonCompliant {
            reasons: vec![
                format!(
                    "Frequency {:.3} MHz violates regulatory rules for jurisdiction {:?}",
                    freq_hz as f64 / 1e6,
                    jurisdiction
                ),
            ],
        }
    }
}

/// Cataloged RF bands.
static REGULATORY_BANDS: [RegulatoryBand; 10] = [
    // US ISM Band (902–928 MHz) - License-free, high power with FHSS/DSSS, encryption fully permitted
    RegulatoryBand {
        name: "US 915 MHz ISM (FCC Part 15.247)",
        jurisdiction: Jurisdiction::US,
        band_type: BandType::Ism,
        start_freq_hz: 902_000_000,
        end_freq_hz: 928_000_000,
        max_power_dbm: 30.0, // 1 Watt conducted / 36 dBm EIRP with FHSS
        max_bandwidth_hz: Some(500_000),
        max_duty_cycle_pct: None,
        encryption_permitted: true,
        citation: "47 CFR § 15.247",
    },
    // EU SRD Band 1 (868.0–868.6 MHz) - License-free, 25 mW, 1% duty cycle, encryption permitted
    RegulatoryBand {
        name: "EU 868 MHz SRD Band h1.1 (CEPT Rec 70-03)",
        jurisdiction: Jurisdiction::EU,
        band_type: BandType::Srd,
        start_freq_hz: 868_000_000,
        end_freq_hz: 868_600_000,
        max_power_dbm: 14.0, // 25 mW ERP
        max_bandwidth_hz: Some(125_000),
        max_duty_cycle_pct: Some(1.0),
        encryption_permitted: true,
        citation: "CEPT ERC Rec 70-03 Annex 1 (Band h1.1)",
    },
    // EU SRD Band 2 (869.4–869.65 MHz) - High power 500 mW, 10% duty cycle, encryption permitted
    RegulatoryBand {
        name: "EU 869.5 MHz SRD High-Power Band h1.3 (CEPT Rec 70-03)",
        jurisdiction: Jurisdiction::EU,
        band_type: BandType::Srd,
        start_freq_hz: 869_400_000,
        end_freq_hz: 869_650_000,
        max_power_dbm: 27.0, // 500 mW ERP
        max_bandwidth_hz: Some(250_000),
        max_duty_cycle_pct: Some(10.0),
        encryption_permitted: true,
        citation: "CEPT ERC Rec 70-03 Annex 1 (Band h1.3)",
    },
    // EU 433 MHz SRD (433.05–434.79 MHz) - License-free, 10 mW ERP, 10% duty cycle
    RegulatoryBand {
        name: "EU 433 MHz SRD (CEPT Rec 70-03)",
        jurisdiction: Jurisdiction::EU,
        band_type: BandType::Srd,
        start_freq_hz: 433_050_000,
        end_freq_hz: 434_790_000,
        max_power_dbm: 10.0, // 10 mW ERP
        max_bandwidth_hz: None,
        max_duty_cycle_pct: Some(10.0),
        encryption_permitted: true,
        citation: "CEPT ERC Rec 70-03 Annex 1 (Band f)",
    },
    // AU 915–928 MHz LIPD (ACMA) - License-free, 1W EIRP, encryption permitted
    RegulatoryBand {
        name: "AU 915 MHz LIPD (ACMA Class License)",
        jurisdiction: Jurisdiction::AU,
        band_type: BandType::Ism,
        start_freq_hz: 915_000_000,
        end_freq_hz: 928_000_000,
        max_power_dbm: 30.0, // 1 Watt EIRP
        max_bandwidth_hz: None,
        max_duty_cycle_pct: None,
        encryption_permitted: true,
        citation: "ACMA Radiocommunications (Low Interference Potential Devices) Class License",
    },
    // Global 2.4 GHz ISM (2400–2483.5 MHz) - Worldwide license-free, encryption permitted
    RegulatoryBand {
        name: "Global 2.4 GHz ISM (ITU-R 5.150)",
        jurisdiction: Jurisdiction::Global,
        band_type: BandType::Ism,
        start_freq_hz: 2_400_000_000,
        end_freq_hz: 2_483_500_000,
        max_power_dbm: 20.0, // 100 mW EIRP standard (30 dBm in US with FHSS)
        max_bandwidth_hz: None,
        max_duty_cycle_pct: None,
        encryption_permitted: true,
        citation: "ITU-R RR 5.150 / FCC § 15.247 / EN 300 328",
    },
    // Global 5.8 GHz ISM (5725–5875 MHz) - Worldwide license-free, encryption permitted
    RegulatoryBand {
        name: "Global 5.8 GHz ISM (ITU-R 5.150)",
        jurisdiction: Jurisdiction::Global,
        band_type: BandType::Ism,
        start_freq_hz: 5_725_000_000,
        end_freq_hz: 5_875_000_000,
        max_power_dbm: 14.0, // 25 mW standard (30 dBm in US)
        max_bandwidth_hz: None,
        max_duty_cycle_pct: None,
        encryption_permitted: true,
        citation: "ITU-R RR 5.150 / FCC § 15.247 / EN 300 440",
    },
    // US 2m Amateur (144–148 MHz) - Amateur Radio Service (ENCRYPTION STRICTLY FORBIDDEN)
    RegulatoryBand {
        name: "US 2-Meter Amateur Band (FCC Part 97)",
        jurisdiction: Jurisdiction::US,
        band_type: BandType::Amateur,
        start_freq_hz: 144_000_000,
        end_freq_hz: 148_000_000,
        max_power_dbm: 61.7, // 1500 Watts PEP for licensed operators
        max_bandwidth_hz: Some(100_000),
        max_duty_cycle_pct: None,
        encryption_permitted: false,
        citation: "47 CFR § 97.113(a)(4) (Encryption Strictly Prohibited)",
    },
    // US 70cm Amateur (420–450 MHz) - Amateur Radio Service (ENCRYPTION STRICTLY FORBIDDEN)
    RegulatoryBand {
        name: "US 70-Centimeter Amateur Band (FCC Part 97)",
        jurisdiction: Jurisdiction::US,
        band_type: BandType::Amateur,
        start_freq_hz: 420_000_000,
        end_freq_hz: 450_000_000,
        max_power_dbm: 61.7, // 1500 Watts PEP
        max_bandwidth_hz: Some(100_000),
        max_duty_cycle_pct: None,
        encryption_permitted: false,
        citation: "47 CFR § 97.113(a)(4) (Encryption Strictly Prohibited)",
    },
    // EU 2m Amateur (144–146 MHz) - CEPT Amateur Radio (ENCRYPTION STRICTLY FORBIDDEN)
    RegulatoryBand {
        name: "EU 2-Meter Amateur Band (CEPT T/R 61-01)",
        jurisdiction: Jurisdiction::EU,
        band_type: BandType::Amateur,
        start_freq_hz: 144_000_000,
        end_freq_hz: 146_000_000,
        max_power_dbm: 50.0,
        max_bandwidth_hz: None,
        max_duty_cycle_pct: None,
        encryption_permitted: false,
        citation: "ITU Radio Reg Article 25.2A / CEPT (Encryption Prohibited)",
    },
];
