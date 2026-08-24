//! Jurisdictional RF Regulatory Band Advisor and Transmission Compliance Engine.

use std::fmt;
use std::str::FromStr;

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
}

/// Error returned when a jurisdiction name is not recognized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseJurisdictionError {
    input: String,
}

impl fmt::Display for ParseJurisdictionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown jurisdiction {:?}; expected US, EU, UK, AU, or Global",
            self.input
        )
    }
}

impl std::error::Error for ParseJurisdictionError {}

impl FromStr for Jurisdiction {
    type Err = ParseJurisdictionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_uppercase().as_str() {
            "US" | "USA" | "FCC" => Ok(Jurisdiction::US),
            "EU" | "EUROPE" | "CEPT" | "ETSI" => Ok(Jurisdiction::EU),
            "UK" | "GB" | "OFCOM" => Ok(Jurisdiction::UK),
            "AU" | "AUSTRALIA" | "ACMA" => Ok(Jurisdiction::AU),
            "GLOBAL" | "WORLD" | "ITU" => Ok(Jurisdiction::Global),
            _ => Err(ParseJurisdictionError {
                input: s.to_string(),
            }),
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

/// Complete set of inputs needed to decide whether one transmission is
/// permitted by the offline regulatory catalog.
///
/// Frequencies and occupied bandwidth are expressed in Hz, EIRP in dBm, and
/// duty cycle in percentage points (`1.0` means one percent).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransmissionPlan {
    pub jurisdiction: Jurisdiction,
    pub center_frequency_hz: f64,
    pub occupied_bandwidth_hz: f64,
    pub eirp_dbm: f64,
    pub duty_cycle_pct: f64,
    pub encrypted: bool,
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
    pub fn check_compliance(plan: &TransmissionPlan) -> ComplianceResult {
        let mut invalid_inputs = Vec::new();
        if !plan.center_frequency_hz.is_finite() || plan.center_frequency_hz <= 0.0 {
            invalid_inputs.push(format!(
                "Center frequency must be finite and greater than 0 Hz (got {:?})",
                plan.center_frequency_hz
            ));
        }
        if !plan.occupied_bandwidth_hz.is_finite() || plan.occupied_bandwidth_hz <= 0.0 {
            invalid_inputs.push(format!(
                "Occupied bandwidth must be finite and greater than 0 Hz (got {:?})",
                plan.occupied_bandwidth_hz
            ));
        }
        if !plan.eirp_dbm.is_finite() {
            invalid_inputs.push(format!(
                "EIRP must be finite in dBm (got {:?})",
                plan.eirp_dbm
            ));
        }
        if !plan.duty_cycle_pct.is_finite()
            || plan.duty_cycle_pct <= 0.0
            || plan.duty_cycle_pct > 100.0
        {
            invalid_inputs.push(format!(
                "Duty cycle must be finite and in the range (0, 100] percent (got {:?})",
                plan.duty_cycle_pct
            ));
        }
        if !invalid_inputs.is_empty() {
            return ComplianceResult::NonCompliant {
                reasons: invalid_inputs,
            };
        }

        let matching_bands: Vec<&RegulatoryBand> = REGULATORY_BANDS
            .iter()
            .filter(|b| {
                (b.jurisdiction == plan.jurisdiction || b.jurisdiction == Jurisdiction::Global)
                    && plan.center_frequency_hz >= b.start_freq_hz as f64
                    && plan.center_frequency_hz <= b.end_freq_hz as f64
            })
            .collect();

        if matching_bands.is_empty() {
            return ComplianceResult::NonCompliant {
                reasons: vec![format!(
                    "Frequency {:.3} MHz is not in any cataloged license-exempt or amateur band for {:?}",
                    plan.center_frequency_hz / 1e6,
                    plan.jurisdiction
                )],
            };
        }

        // A frequency resolves to a single cataloged band; evaluate the first match.
        if let Some(band) = matching_bands.into_iter().next() {
            let mut reasons = Vec::new();
            let mut warnings = Vec::new();

            let half_bandwidth_hz = plan.occupied_bandwidth_hz / 2.0;
            let occupied_start_hz = plan.center_frequency_hz - half_bandwidth_hz;
            let occupied_end_hz = plan.center_frequency_hz + half_bandwidth_hz;
            // Compare the full width with the exact representable margins.
            // Computing only `center +/- width / 2` can round a sub-ULP width
            // back to `center` at RF-scale frequencies and fail open at an edge.
            let lower_margin_hz = plan.center_frequency_hz - band.start_freq_hz as f64;
            let upper_margin_hz = band.end_freq_hz as f64 - plan.center_frequency_hz;
            let max_contained_bandwidth_hz = 2.0 * lower_margin_hz.min(upper_margin_hz);
            if plan.occupied_bandwidth_hz > max_contained_bandwidth_hz {
                reasons.push(format!(
                    "Occupied span {:.3}-{:.3} MHz crosses the {} band edge ({:.3}-{:.3} MHz; {})",
                    occupied_start_hz / 1e6,
                    occupied_end_hz / 1e6,
                    band.name,
                    band.start_freq_hz as f64 / 1e6,
                    band.end_freq_hz as f64 / 1e6,
                    band.citation
                ));
            }

            if let Some(max_bandwidth_hz) = band.max_bandwidth_hz {
                if plan.occupied_bandwidth_hz > max_bandwidth_hz as f64 {
                    reasons.push(format!(
                        "Occupied bandwidth {:.0} Hz exceeds legal limit of {} Hz ({})",
                        plan.occupied_bandwidth_hz, max_bandwidth_hz, band.citation
                    ));
                }
            }

            if let Some(max_duty_cycle_pct) = band.max_duty_cycle_pct {
                if plan.duty_cycle_pct > max_duty_cycle_pct as f64 {
                    reasons.push(format!(
                        "Duty cycle {:.3}% exceeds legal limit of {:.3}% ({})",
                        plan.duty_cycle_pct, max_duty_cycle_pct, band.citation
                    ));
                } else {
                    warnings.push(format!(
                        "Duty cycle restriction: max {:.1}% transmission time ({})",
                        max_duty_cycle_pct, band.citation
                    ));
                }
            }

            // Check encryption legality (strictly forbidden on Amateur bands by telecommunications law)
            if plan.encrypted && !band.encryption_permitted {
                reasons.push(format!(
                    "ENCRYPTION PROHIBITED: Frequency {:.3} MHz is in the {} band ({}) where transmitting encrypted payloads (like SSH/TLS) violates telecommunications law ({})",
                    plan.center_frequency_hz / 1e6,
                    band.name,
                    if band.band_type == BandType::Amateur { "Amateur Radio Service" } else { "Unpermitted Service" },
                    band.citation
                ));
            }

            // Check power limit
            if plan.eirp_dbm > band.max_power_dbm as f64 {
                reasons.push(format!(
                    "Power {:.1} dBm exceeds legal limit of {:.1} dBm ({})",
                    plan.eirp_dbm, band.max_power_dbm, band.citation
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
            reasons: vec![format!(
                "Frequency {:.3} MHz violates regulatory rules for jurisdiction {:?}",
                plan.center_frequency_hz / 1e6,
                plan.jurisdiction
            )],
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

#[cfg(test)]
mod tests {
    use super::*;

    fn us_915_plan() -> TransmissionPlan {
        TransmissionPlan {
            jurisdiction: Jurisdiction::US,
            center_frequency_hz: 915_000_000.0,
            occupied_bandwidth_hz: 100_000.0,
            eirp_dbm: 20.0,
            duty_cycle_pct: 100.0,
            encrypted: true,
        }
    }

    fn assert_noncompliant_contains(result: ComplianceResult, expected: &str) {
        match result {
            ComplianceResult::NonCompliant { reasons } => assert!(
                reasons
                    .iter()
                    .any(|reason| reason.to_ascii_lowercase().contains(expected)),
                "expected refusal containing {expected:?}, got {reasons:?}"
            ),
            other => panic!("expected non-compliant result, got {other:?}"),
        }
    }

    #[test]
    fn test_jurisdiction_from_str_rejects_unknown() {
        assert_eq!("fcc".parse::<Jurisdiction>().unwrap(), Jurisdiction::US);
        let err = "somewhere-else".parse::<Jurisdiction>().unwrap_err();
        assert!(err.to_string().contains("unknown jurisdiction"));
    }

    #[test]
    fn test_compliance_rejects_non_finite_inputs() {
        let base = us_915_plan();
        let plans = [
            TransmissionPlan {
                center_frequency_hz: f64::NAN,
                ..base
            },
            TransmissionPlan {
                center_frequency_hz: f64::INFINITY,
                ..base
            },
            TransmissionPlan {
                occupied_bandwidth_hz: f64::NEG_INFINITY,
                ..base
            },
            TransmissionPlan {
                eirp_dbm: f64::NAN,
                ..base
            },
            TransmissionPlan {
                eirp_dbm: f64::INFINITY,
                ..base
            },
            TransmissionPlan {
                duty_cycle_pct: f64::NAN,
                ..base
            },
        ];

        for plan in plans {
            assert!(
                matches!(
                    RegulatoryDatabase::check_compliance(&plan),
                    ComplianceResult::NonCompliant { .. }
                ),
                "non-finite plan must fail closed: {plan:?}"
            );
        }
    }

    #[test]
    fn test_compliance_rejects_invalid_frequency_and_bandwidth() {
        let base = us_915_plan();
        for plan in [
            TransmissionPlan {
                center_frequency_hz: 0.0,
                ..base
            },
            TransmissionPlan {
                center_frequency_hz: -1.0,
                ..base
            },
            TransmissionPlan {
                occupied_bandwidth_hz: 0.0,
                ..base
            },
            TransmissionPlan {
                occupied_bandwidth_hz: -1.0,
                ..base
            },
        ] {
            assert!(
                matches!(
                    RegulatoryDatabase::check_compliance(&plan),
                    ComplianceResult::NonCompliant { .. }
                ),
                "invalid frequency/bandwidth must fail closed: {plan:?}"
            );
        }
    }

    #[test]
    fn test_compliance_rejects_invalid_duty_cycle_range() {
        let base = us_915_plan();
        for duty_cycle_pct in [-1.0, 0.0, 100.000_001] {
            let plan = TransmissionPlan {
                duty_cycle_pct,
                ..base
            };
            assert_noncompliant_contains(RegulatoryDatabase::check_compliance(&plan), "duty cycle");
        }
    }

    #[test]
    fn test_compliance_accepts_zero_and_negative_eirp() {
        for eirp_dbm in [0.0, -40.0] {
            let plan = TransmissionPlan {
                eirp_dbm,
                ..us_915_plan()
            };
            assert!(
                matches!(
                    RegulatoryDatabase::check_compliance(&plan),
                    ComplianceResult::Compliant { .. }
                ),
                "finite non-positive EIRP is a valid input: {plan:?}"
            );
        }
    }

    #[test]
    fn test_compliance_uses_percentage_duty_cycle_units() {
        let base = TransmissionPlan {
            jurisdiction: Jurisdiction::EU,
            center_frequency_hz: 868_300_000.0,
            occupied_bandwidth_hz: 100_000.0,
            eirp_dbm: 10.0,
            duty_cycle_pct: 1.0,
            encrypted: true,
        };
        for duty_cycle_pct in [0.01, 1.0] {
            let plan = TransmissionPlan {
                duty_cycle_pct,
                ..base
            };
            assert!(
                matches!(
                    RegulatoryDatabase::check_compliance(&plan),
                    ComplianceResult::Compliant { .. }
                ),
                "{duty_cycle_pct} must be interpreted as percentage points"
            );
        }
        assert_noncompliant_contains(
            RegulatoryDatabase::check_compliance(&TransmissionPlan {
                duty_cycle_pct: 1.01,
                ..base
            }),
            "duty cycle",
        );
    }

    #[test]
    fn test_compliance_rejects_bandwidth_over_catalog_limit() {
        let plan = TransmissionPlan {
            occupied_bandwidth_hz: 500_001.0,
            ..us_915_plan()
        };
        assert_noncompliant_contains(
            RegulatoryDatabase::check_compliance(&plan),
            "occupied bandwidth",
        );
    }

    #[test]
    fn test_compliance_rejects_occupied_span_crossing_band_edge() {
        let plan = TransmissionPlan {
            center_frequency_hz: 902_000_000.0,
            occupied_bandwidth_hz: 1.0,
            ..us_915_plan()
        };
        assert_noncompliant_contains(RegulatoryDatabase::check_compliance(&plan), "occupied span");
    }

    #[test]
    fn test_compliance_rejects_sub_ulp_bandwidth_at_band_edge() {
        let plan = TransmissionPlan {
            center_frequency_hz: 902_000_000.0,
            occupied_bandwidth_hz: 0.000_000_01,
            ..us_915_plan()
        };
        assert_eq!(
            plan.center_frequency_hz - plan.occupied_bandwidth_hz / 2.0,
            plan.center_frequency_hz,
            "fixture must exercise endpoint rounding"
        );
        assert_noncompliant_contains(RegulatoryDatabase::check_compliance(&plan), "occupied span");
    }

    #[test]
    fn test_compliance_rejects_duty_cycle_over_catalog_limit() {
        let plan = TransmissionPlan {
            jurisdiction: Jurisdiction::EU,
            center_frequency_hz: 868_300_000.0,
            occupied_bandwidth_hz: 100_000.0,
            eirp_dbm: 10.0,
            duty_cycle_pct: 1.1,
            encrypted: true,
        };
        assert_noncompliant_contains(RegulatoryDatabase::check_compliance(&plan), "duty cycle");
    }

    #[test]
    fn test_compliance_accepts_values_at_catalog_limits() {
        let plan = TransmissionPlan {
            jurisdiction: Jurisdiction::EU,
            center_frequency_hz: 869_525_000.0,
            occupied_bandwidth_hz: 250_000.0,
            eirp_dbm: 27.0,
            duty_cycle_pct: 10.0,
            encrypted: true,
        };
        assert!(
            matches!(
                RegulatoryDatabase::check_compliance(&plan),
                ComplianceResult::Compliant { .. }
            ),
            "values exactly at encoded limits must be accepted"
        );
    }

    #[test]
    fn test_compliance_preserves_power_and_encryption_refusals() {
        let plan = TransmissionPlan {
            jurisdiction: Jurisdiction::US,
            center_frequency_hz: 145_000_000.0,
            occupied_bandwidth_hz: 100_000.0,
            eirp_dbm: 61.8,
            duty_cycle_pct: 100.0,
            encrypted: true,
        };
        let ComplianceResult::NonCompliant { reasons } =
            RegulatoryDatabase::check_compliance(&plan)
        else {
            panic!("over-power encrypted amateur transmission must be refused");
        };
        assert!(reasons.iter().any(|reason| reason.contains("Power")));
        assert!(reasons
            .iter()
            .any(|reason| reason.contains("ENCRYPTION PROHIBITED")));
    }
}
