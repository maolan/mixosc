#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FxParameterKind {
    Linear { min: f32, max: f32 },
    Logarithmic { min: f32, max: f32 },
    Enum(&'static [&'static str]),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FxParameter {
    pub(crate) name: &'static str,
    pub(crate) kind: FxParameterKind,
}

pub(crate) struct FxDefinition {
    pub(crate) parameters: &'static [FxParameter],
}

const fn lin(name: &'static str, min: f32, max: f32) -> FxParameter {
    FxParameter {
        name,
        kind: FxParameterKind::Linear { min, max },
    }
}

const fn log(name: &'static str, min: f32, max: f32) -> FxParameter {
    FxParameter {
        name,
        kind: FxParameterKind::Logarithmic { min, max },
    }
}

const fn en(name: &'static str, values: &'static [&'static str]) -> FxParameter {
    FxParameter {
        name,
        kind: FxParameterKind::Enum(values),
    }
}

impl FxParameterKind {
    pub(crate) fn value(self, normalized: f32) -> f32 {
        let normalized = normalized.clamp(0.0, 1.0);
        match self {
            Self::Linear { min, max } => min + normalized * (max - min),
            Self::Logarithmic { min, max } => min * (max / min).powf(normalized),
            Self::Enum(_) => normalized,
        }
    }

    pub(crate) fn format(self, name: &str, normalized: f32) -> String {
        if let Self::Enum(values) = self {
            return values
                .get(normalized.round() as usize)
                .copied()
                .unwrap_or("?")
                .to_owned();
        }
        let value = self.value(normalized);
        let lower = name.to_ascii_lowercase();
        let suffix = if lower.contains("freq")
            || lower.contains("cut")
            || lower.contains("damping")
            || lower.contains("tune")
            || lower.contains("xover") && matches!(self, Self::Logarithmic { .. })
        {
            if value >= 1000.0 {
                return format!("{:.2} kHz", value / 1000.0);
            }
            " Hz"
        } else if lower.contains("gain") || lower.contains("level") {
            " dB"
        } else if lower.contains("phase") {
            "°"
        } else if lower.contains("mix")
            || lower.contains("depth")
            || lower.contains("feedback")
            || lower.contains("contour")
        {
            "%"
        } else {
            ""
        };
        if value.abs() >= 100.0 {
            format!("{value:.0}{suffix}")
        } else if value.abs() >= 10.0 {
            format!("{value:.1}{suffix}")
        } else {
            format!("{value:.2}{suffix}")
        }
    }
}

static HALL: FxDefinition = FxDefinition {
    parameters: &[
        lin("Pre Delay", 0.0, 200.0),
        log("Decay", 0.2, 5.0),
        lin("Size", 2.0, 100.0),
        log("Damping", 1000.0, 20000.0),
        lin("Diffuse", 1.0, 30.0),
        lin("Level", -12.0, 12.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        log("Bass Multi", 0.5, 2.0),
        lin("Spread", 0.0, 50.0),
        lin("Shape", 0.0, 250.0),
        lin("Mod Speed", 0.0, 100.0),
    ],
};

static AMBI: FxDefinition = FxDefinition {
    parameters: &[
        lin("Pre Delay", 0.0, 200.0),
        log("Decay", 0.2, 7.3),
        lin("Size", 2.0, 100.0),
        log("Damping", 1000.0, 20000.0),
        lin("Diffuse", 1.0, 30.0),
        lin("Level", -12.0, 12.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        lin("Modulate", 0.0, 100.0),
        lin("Tail Gain", 0.0, 100.0),
    ],
};

static RPLT: FxDefinition = FxDefinition {
    parameters: &[
        lin("Pre Delay", 0.0, 200.0),
        log("Decay", 0.3, 29.0),
        lin("Size", 4.0, 39.0),
        log("Damping", 1000.0, 20000.0),
        lin("Diffuse", 1.0, 30.0),
        lin("Level", -12.0, 12.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        log("Bass Multi", 0.25, 4.0),
        lin("Spread", 0.0, 50.0),
        lin("Attack", 0.0, 100.0),
        lin("Spin", 0.0, 100.0),
        lin("Echo L", 0.0, 1200.0),
        lin("Echo R", 0.0, 1200.0),
        lin("Echo Feed L", -100.0, 100.0),
        lin("Echo Feed R", -100.0, 100.0),
    ],
};

static ROOM: FxDefinition = FxDefinition {
    parameters: &[
        lin("Pre Delay", 0.0, 200.0),
        log("Decay", 0.3, 29.0),
        lin("Size", 4.0, 72.0),
        log("Damping", 1000.0, 20000.0),
        lin("Diffuse", 1.0, 30.0),
        lin("Level", -12.0, 12.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        log("Bass Multi", 0.25, 4.0),
        lin("Spread", 0.0, 50.0),
        lin("Shape", 0.0, 250.0),
        lin("Spin", 0.0, 100.0),
        lin("Echo L", 0.0, 1200.0),
        lin("Echo R", 0.0, 1200.0),
        lin("Echo Feed L", -100.0, 100.0),
        lin("Echo Feed R", -100.0, 100.0),
    ],
};

static CHAM: FxDefinition = FxDefinition {
    parameters: &[
        lin("Pre Delay", 0.0, 200.0),
        log("Decay", 0.3, 29.0),
        lin("Size", 4.0, 72.0),
        log("Damping", 1000.0, 20000.0),
        lin("Diffuse", 1.0, 30.0),
        lin("Level", -12.0, 12.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        log("Bass Multi", 0.25, 4.0),
        lin("Spread", 0.0, 50.0),
        lin("Shape", 0.0, 250.0),
        lin("Spin", 0.0, 100.0),
        lin("Reflection L", 0.0, 500.0),
        lin("Reflection R", 0.0, 500.0),
        lin("Reflection Gain L", 0.0, 100.0),
        lin("Reflection Gain L", 0.0, 100.0),
    ],
};

static PLAT: FxDefinition = FxDefinition {
    parameters: &[
        lin("Pre Delay", 0.0, 200.0),
        log("Decay", 0.2, 10.0),
        lin("Size", 2.0, 100.0),
        log("Damping", 1000.0, 20000.0),
        lin("Diffuse", 1.0, 30.0),
        lin("Level", -12.0, 12.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        log("Bass Multi", 0.5, 2.0),
        log("Xover", 10.0, 500.0),
        lin("Mod", 0.0, 50.0),
        lin("Mod Speed", 0.0, 100.0),
    ],
};

static VREV: FxDefinition = FxDefinition {
    parameters: &[
        lin("Pre Delay", 0.0, 120.0),
        log("Decay", 0.3, 4.5),
        lin("Modulate", 0.0, 10.0),
        en("Vintage", &["OFF", "ON"]),
        en("Position", &["FRONT", "REAR"]),
        lin("Level", -12.0, 12.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        log("Lo Multiply", 0.5, 2.0),
        log("Hi Multiply", 0.25, 1.0),
    ],
};

static VRM: FxDefinition = FxDefinition {
    parameters: &[
        lin("Reverb Delay", 0.0, 20.0),
        log("Decay", 0.1, 20.0),
        lin("Size", 0.0, 10.0),
        lin("Density", 1.0, 30.0),
        lin("ER Level", 0.0, 190.0),
        lin("Level", -12.0, 12.0),
        log("Lo Multiply", 0.1, 10.0),
        log("Hi Multiply", 0.1, 10.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        lin("ER Left", 0.0, 10.0),
        lin("ER Right", 0.0, 10.0),
        en("Freeze", &["OFF", "ON"]),
    ],
};

static GATE: FxDefinition = FxDefinition {
    parameters: &[
        lin("Pre Delay", 0.0, 200.0),
        log("Decay", 140.0, 1000.0),
        lin("Attack", 0.0, 30.0),
        lin("Density", 1.0, 30.0),
        lin("Spread", 0.0, 100.0),
        lin("Level", -12.0, 12.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        lin("Hi Shv Gain", -30.0, 0.0),
        lin("Diffuse", 1.0, 30.0),
    ],
};

static RVRS: FxDefinition = FxDefinition {
    parameters: &[
        lin("Pre Delay", 0.0, 200.0),
        log("Decay", 140.0, 1000.0),
        lin("Rise", 0.0, 50.0),
        lin("Diffuse", 1.0, 30.0),
        lin("Spread", 1.0, 100.0),
        lin("Level", -12.0, 12.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        lin("Hi Shv Gain", -30.0, 0.0),
    ],
};

static DLY: FxDefinition = FxDefinition {
    parameters: &[
        lin("Mix", 0.0, 100.0),
        lin("Time", 0.0, 3000.0),
        en("Mode", &["ST", "X", "M"]),
        en(
            "Factor L",
            &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
        ),
        en(
            "Factor R",
            &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
        ),
        lin("Offset L/R", -100.0, 100.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        log("Feed Lo Cut", 10.0, 500.0),
        lin("Feed Left", 1.0, 100.0),
        lin("Feed Right", 1.0, 100.0),
        log("Feed Hi Cut", 200.0, 20000.0),
    ],
};

static FX_3TAP: FxDefinition = FxDefinition {
    parameters: &[
        lin("Time", 0.0, 3000.0),
        lin("Gain Base", 0.0, 100.0),
        lin("Pan Base", -100.0, 100.0),
        lin("Feedback", 0.0, 100.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        en(
            "Factor A",
            &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
        ),
        lin("Gain A", 0.0, 100.0),
        lin("Pan A", -100.0, 100.0),
        en(
            "Factor B",
            &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
        ),
        lin("Gain B", 0.0, 100.0),
        lin("Pan B", -100.0, 100.0),
        en("Cross Feed", &["OFF", "ON"]),
        en("Mono", &["OFF", "ON"]),
        en("Dry", &["OFF", "ON"]),
    ],
};

static FX_4TAP: FxDefinition = FxDefinition {
    parameters: &[
        lin("Time", 1.0, 3000.0),
        lin("Gain Base", 0.0, 100.0),
        lin("Feedback", 0.0, 100.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        lin("Spread", 0.0, 6.0),
        en(
            "Factor A",
            &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
        ),
        lin("Gain A", 0.0, 100.0),
        en(
            "Factor B",
            &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
        ),
        lin("Gain B", 0.0, 100.0),
        en(
            "Factor C",
            &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
        ),
        lin("Gain C", 0.0, 100.0),
        en("Cross Feed", &["OFF", "ON"]),
        en("Mono", &["OFF", "ON"]),
        en("Dry", &["OFF", "ON"]),
    ],
};

static CRS: FxDefinition = FxDefinition {
    parameters: &[
        log("Speed", 0.05, 5.0),
        lin("Depth L", 0.0, 100.0),
        lin("Depth R", 0.0, 100.0),
        log("Delay L", 0.5, 20.0),
        log("Delay R", 0.5, 20.0),
        lin("Mix", 0.0, 100.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        lin("Phase", 0.0, 180.0),
        lin("Wave", 0.0, 100.0),
        lin("Spread", 0.0, 100.0),
    ],
};

static FLNG: FxDefinition = FxDefinition {
    parameters: &[
        log("Speed", 0.05, 5.0),
        lin("Depth L", 0.0, 100.0),
        lin("Depth R", 0.0, 100.0),
        log("Delay L", 0.5, 20.0),
        log("Delay R", 0.5, 20.0),
        lin("Mix", 0.0, 100.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        lin("Phase", 0.0, 180.0),
        log("Feed Lo Cut", 10.0, 500.0),
        log("Feed Hi Cut", 200.0, 20000.0),
        lin("Feed", -90.0, 90.0),
    ],
};

static PHAS: FxDefinition = FxDefinition {
    parameters: &[
        log("Speed", 0.05, 5.0),
        lin("Depth", 0.0, 100.0),
        lin("Resonance", 0.0, 80.0),
        lin("Base", 0.0, 50.0),
        lin("Stages", 2.0, 12.0),
        lin("Mix", 0.0, 100.0),
        lin("Wave", -50.0, 50.0),
        lin("Phase", 0.0, 180.0),
        lin("Env. Modulation", -100.0, 100.0),
        log("Attack", 10.0, 1000.0),
        log("Hold", 1.0, 2000.0),
        log("Release", 10.0, 1000.0),
    ],
};

static DIMC: FxDefinition = FxDefinition {
    parameters: &[
        en("Active", &["OFF", "ON"]),
        en("Mode", &["M", "ST"]),
        en("Dry", &["OFF", "ON"]),
        en("Mode 1", &["OFF", "ON"]),
        en("Mode 2", &["OFF", "ON"]),
        en("Mode 3", &["OFF", "ON"]),
        en("Mode 4", &["OFF", "ON"]),
    ],
};

static FILT: FxDefinition = FxDefinition {
    parameters: &[
        log("Speed", 0.05, 20.0),
        lin("Depth", 0.0, 100.0),
        lin("Resonance", 0.0, 100.0),
        log("Base", 10.0, 15000.0),
        en("Mode", &["LP", "HP", "BP", "NO"]),
        lin("Mix", 0.0, 100.0),
        en("Wave", &["TRI", "SIN", "SAW", "SAW‐", "RMP", "SQU", "RND"]),
        lin("Phase", 0.0, 180.0),
        lin("Env. Modulation", -100.0, 100.0),
        log("Attack", 10.0, 250.0),
        log("Release", 10.0, 500.0),
        lin("Drive", 0.0, 100.0),
        en("4 Pole", &["OFF", "ON"]),
        en("Side Chain", &["OFF", "ON"]),
    ],
};

static ROTA: FxDefinition = FxDefinition {
    parameters: &[
        log("Lo Speed", 0.1, 4.0),
        log("Hi Speed", 2.0, 10.0),
        lin("Accelerate", 0.0, 100.0),
        lin("Distance", 0.0, 100.0),
        lin("Balance", -100.0, 100.0),
        lin("Mix", 0.0, 100.0),
        en("Stop", &["OFF", "ON"]),
        en("Slow", &["OFF", "ON"]),
    ],
};

static PAN: FxDefinition = FxDefinition {
    parameters: &[
        log("Speed", 0.05, 4.0),
        lin("Phase", 0.0, 180.0),
        lin("Wave", -50.0, 50.0),
        lin("Depth", 0.0, 100.0),
        lin("Env. Speed", 0.0, 100.0),
        lin("Env. Depth", 0.0, 100.0),
        log("Attack", 10.0, 1000.0),
        log("Hold", 1.0, 2000.0),
        log("Release", 10.0, 1000.0),
    ],
};

static SUB: FxDefinition = FxDefinition {
    parameters: &[
        en("Active", &["OFF", "ON"]),
        en("Range", &["LO", "MID", "HI"]),
        lin("Dry", 0.0, 100.0),
        lin("Octave ‐1", 0.0, 100.0),
        lin("Octave ‐2", 0.0, 100.0),
        en("Active", &["OFF", "ON"]),
        en("Range", &["LO", "MID", "HI"]),
        lin("Dry", 0.0, 100.0),
        lin("Octave ‐1", 0.0, 100.0),
        lin("Octave ‐2", 0.0, 100.0),
    ],
};

static D_RV: FxDefinition = FxDefinition {
    parameters: &[
        lin("Time", 1.0, 3000.0),
        en(
            "Pattern",
            &[
                "1/4", "1/3", "3/8", "1/2", "2/3", "3/4", "1", "1/4X", "1/3X", "3/8X", "1/2X",
                "2/3X", "3/4X", "1X",
            ],
        ),
        log("Feed Hi Cut", 1000.0, 20000.0),
        lin("Feedback", 0.0, 100.0),
        lin("Cross Feed", 0.0, 100.0),
        lin("Balance", -100.0, 100.0),
        lin("Pre Delay", 0.0, 200.0),
        log("Decay", 0.1, 5.0),
        lin("Size", 2.0, 100.0),
        log("Damping", 1000.0, 20000.0),
        log("Lo Cut", 10.0, 500.0),
        lin("Mix", 0.0, 100.0),
    ],
};

static CR_R: FxDefinition = FxDefinition {
    parameters: &[
        log("Speed", 0.05, 4.0),
        lin("Depth", 0.0, 100.0),
        log("Delay", 0.5, 50.0),
        lin("Phase", 0.0, 180.0),
        lin("Wave", 0.0, 100.0),
        lin("Balance", -100.0, 100.0),
        lin("Pre Delay", 0.0, 200.0),
        log("Decay", 0.1, 5.0),
        lin("Size", 2.0, 100.0),
        log("Damping", 1000.0, 20000.0),
        log("Lo Cut", 10.0, 500.0),
        lin("Mix", 0.0, 100.0),
    ],
};

static FL_R: FxDefinition = FxDefinition {
    parameters: &[
        log("Speed", 0.05, 4.0),
        lin("Depth", 0.0, 100.0),
        log("Delay", 0.5, 20.0),
        lin("Phase", 0.0, 180.0),
        lin("Feed", -90.0, 90.0),
        lin("Balance", -100.0, 100.0),
        lin("Pre Delay", 0.0, 200.0),
        log("Decay", 0.1, 5.0),
        lin("Size", 2.0, 100.0),
        log("Damping", 1000.0, 20000.0),
        log("Lo Cut", 10.0, 500.0),
        lin("Mix", 0.0, 100.0),
    ],
};

static D_CR: FxDefinition = FxDefinition {
    parameters: &[
        lin("Time", 1.0, 3000.0),
        en(
            "Pattern",
            &[
                "1/4", "1/3", "3/8", "1/2", "2/3", "3/4", "1", "1/4X", "1/3X", "3/8X", "1/2X",
                "2/3X", "3/4X", "1X",
            ],
        ),
        log("Feed Hi Cut", 1000.0, 20000.0),
        lin("Feedback", 0.0, 100.0),
        lin("Cross Feed", 0.0, 100.0),
        lin("Balance", -100.0, 100.0),
        log("Speed", 0.05, 4.0),
        lin("Depth", 0.0, 100.0),
        log("Delay", 0.5, 50.0),
        lin("Phase", 0.0, 180.0),
        lin("Wave", 0.0, 100.0),
        lin("Mix", 0.0, 100.0),
    ],
};

static D_FL: FxDefinition = FxDefinition {
    parameters: &[
        lin("Time", 1.0, 3000.0),
        en(
            "Pattern",
            &[
                "1/4", "1/3", "3/8", "1/2", "2/3", "3/4", "1", "1/4X", "1/3X", "3/8X", "1/2X",
                "2/3X", "3/4X", "1X",
            ],
        ),
        log("Feed Hi Cut", 1000.0, 20000.0),
        lin("Feedback", 0.0, 100.0),
        lin("Cross Feed", 0.0, 100.0),
        lin("Balance", -100.0, 100.0),
        log("Speed", 0.05, 4.0),
        lin("Depth", 0.0, 100.0),
        log("Delay", 0.5, 20.0),
        lin("Phase", 0.0, 180.0),
        lin("Feed", -90.0, 90.0),
        lin("Mix", 0.0, 100.0),
    ],
};

static MODD: FxDefinition = FxDefinition {
    parameters: &[
        lin("Time", 1.0, 3000.0),
        en("Delay", &["1", "1/2", "2/3", "3/2"]),
        lin("Feed", 0.0, 100.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 200.0, 20000.0),
        lin("Depth Rate", 0.0, 100.0),
        log("Rate", 0.05, 10.0),
        en("Setup", &["PAR", "SER"]),
        en("Type", &["AMB", "CLUB", "HALL"]),
        lin("Decay", 1.0, 10.0),
        log("Damping", 1000.0, 20000.0),
        lin("Balance", -100.0, 100.0),
        lin("Mix", 0.0, 100.0),
    ],
};

static GEQ2: FxDefinition = FxDefinition {
    parameters: &[
        lin("20 Hz A", -15.0, 15.0),
        lin("25 Hz A", -15.0, 15.0),
        lin("31.5 Hz A", -15.0, 15.0),
        lin("40 Hz A", -15.0, 15.0),
        lin("50 Hz A", -15.0, 15.0),
        lin("63 Hz A", -15.0, 15.0),
        lin("80 Hz A", -15.0, 15.0),
        lin("100 Hz A", -15.0, 15.0),
        lin("125 Hz A", -15.0, 15.0),
        lin("160 Hz A", -15.0, 15.0),
        lin("200 Hz A", -15.0, 15.0),
        lin("250 Hz A", -15.0, 15.0),
        lin("315 Hz A", -15.0, 15.0),
        lin("400 Hz A", -15.0, 15.0),
        lin("500 Hz A", -15.0, 15.0),
        lin("630 Hz A", -15.0, 15.0),
        lin("800 Hz A", -15.0, 15.0),
        lin("1 kHz A", -15.0, 15.0),
        lin("1.25 kHz A", -15.0, 15.0),
        lin("1.6 kHz A", -15.0, 15.0),
        lin("2 kHz A", -15.0, 15.0),
        lin("2.5 kHz A", -15.0, 15.0),
        lin("3.15 kHz A", -15.0, 15.0),
        lin("4 kHz A", -15.0, 15.0),
        lin("5 kHz A", -15.0, 15.0),
        lin("6.3 kHz A", -15.0, 15.0),
        lin("8 kHz A", -15.0, 15.0),
        lin("10 kHz A", -15.0, 15.0),
        lin("12.5 kHz A", -15.0, 15.0),
        lin("16 kHz A", -15.0, 15.0),
        lin("20 kHz A", -15.0, 15.0),
        lin("Master Level A", -15.0, 15.0),
        lin("20 Hz B", -15.0, 15.0),
        lin("25 Hz B", -15.0, 15.0),
        lin("31.5 Hz B", -15.0, 15.0),
        lin("40 Hz B", -15.0, 15.0),
        lin("50 Hz B", -15.0, 15.0),
        lin("63 Hz B", -15.0, 15.0),
        lin("80 Hz B", -15.0, 15.0),
        lin("100 Hz B", -15.0, 15.0),
        lin("125 Hz B", -15.0, 15.0),
        lin("160 Hz B", -15.0, 15.0),
        lin("200 Hz B", -15.0, 15.0),
        lin("250 Hz B", -15.0, 15.0),
        lin("315 Hz B", -15.0, 15.0),
        lin("400 Hz B", -15.0, 15.0),
        lin("500 Hz B", -15.0, 15.0),
        lin("630 Hz B", -15.0, 15.0),
        lin("800 Hz B", -15.0, 15.0),
        lin("1 kHz B", -15.0, 15.0),
        lin("1.25 kHz B", -15.0, 15.0),
        lin("1.6 kHz B", -15.0, 15.0),
        lin("2 kHz B", -15.0, 15.0),
        lin("2.5 kHz B", -15.0, 15.0),
        lin("3.15 kHz B", -15.0, 15.0),
        lin("4 kHz B", -15.0, 15.0),
        lin("5 kHz B", -15.0, 15.0),
        lin("6.3 kHz B", -15.0, 15.0),
        lin("8 kHz B", -15.0, 15.0),
        lin("10 kHz B", -15.0, 15.0),
        lin("12.5 kHz B", -15.0, 15.0),
        lin("16 kHz B", -15.0, 15.0),
        lin("20 kHz B", -15.0, 15.0),
        lin("Master Level B", -15.0, 15.0),
    ],
};

static GEQ: FxDefinition = FxDefinition {
    parameters: &[
        lin("20 Hz L/R", -15.0, 15.0),
        lin("25 Hz L/R", -15.0, 15.0),
        lin("31.5 Hz L/R", -15.0, 15.0),
        lin("40 Hz L/R", -15.0, 15.0),
        lin("50 Hz L/R", -15.0, 15.0),
        lin("63 Hz L/R", -15.0, 15.0),
        lin("80 Hz L/R", -15.0, 15.0),
        lin("100 Hz L/R", -15.0, 15.0),
        lin("125 Hz L/R", -15.0, 15.0),
        lin("160 Hz L/R", -15.0, 15.0),
        lin("200 Hz L/R", -15.0, 15.0),
        lin("250 Hz L/R", -15.0, 15.0),
        lin("315 Hz L/R", -15.0, 15.0),
        lin("400 Hz L/R", -15.0, 15.0),
        lin("500 Hz L/R", -15.0, 15.0),
        lin("630 Hz L/R", -15.0, 15.0),
        lin("800 Hz L/R", -15.0, 15.0),
        lin("1 kHz L/R", -15.0, 15.0),
        lin("1.25 kHz L/R", -15.0, 15.0),
        lin("1.6 kHz L/R", -15.0, 15.0),
        lin("2 kHz L/R", -15.0, 15.0),
        lin("2.5 kHz L/R", -15.0, 15.0),
        lin("3.15 kHz L/R", -15.0, 15.0),
        lin("4 kHz L/R", -15.0, 15.0),
        lin("5 kHz L/R", -15.0, 15.0),
        lin("6.3 kHz L/R", -15.0, 15.0),
        lin("8 kHz L/R", -15.0, 15.0),
        lin("10 kHz L/R", -15.0, 15.0),
        lin("12.5 kHz L/R", -15.0, 15.0),
        lin("16 kHz L/R", -15.0, 15.0),
        lin("20 kHz L/R", -15.0, 15.0),
        lin("Master Level L/R", -15.0, 15.0),
    ],
};

static TEQ2: FxDefinition = FxDefinition {
    parameters: &[
        lin("20 Hz A", -15.0, 15.0),
        lin("25 Hz A", -15.0, 15.0),
        lin("31.5 Hz A", -15.0, 15.0),
        lin("40 Hz A", -15.0, 15.0),
        lin("50 Hz A", -15.0, 15.0),
        lin("63 Hz A", -15.0, 15.0),
        lin("80 Hz A", -15.0, 15.0),
        lin("100 Hz A", -15.0, 15.0),
        lin("125 Hz A", -15.0, 15.0),
        lin("160 Hz A", -15.0, 15.0),
        lin("200 Hz A", -15.0, 15.0),
        lin("250 Hz A", -15.0, 15.0),
        lin("315 Hz A", -15.0, 15.0),
        lin("400 Hz A", -15.0, 15.0),
        lin("500 Hz A", -15.0, 15.0),
        lin("630 Hz A", -15.0, 15.0),
        lin("800 Hz A", -15.0, 15.0),
        lin("1 kHz A", -15.0, 15.0),
        lin("1.25 kHz A", -15.0, 15.0),
        lin("1.6 kHz A", -15.0, 15.0),
        lin("2 kHz A", -15.0, 15.0),
        lin("2.5 kHz A", -15.0, 15.0),
        lin("3.15 kHz A", -15.0, 15.0),
        lin("4 kHz A", -15.0, 15.0),
        lin("5 kHz A", -15.0, 15.0),
        lin("6.3 kHz A", -15.0, 15.0),
        lin("8 kHz A", -15.0, 15.0),
        lin("10 kHz A", -15.0, 15.0),
        lin("12.5 kHz A", -15.0, 15.0),
        lin("16 kHz A", -15.0, 15.0),
        lin("20 kHz A", -15.0, 15.0),
        lin("Master Level A", -15.0, 15.0),
        lin("20 Hz B", -15.0, 15.0),
        lin("25 Hz B", -15.0, 15.0),
        lin("31.5 Hz B", -15.0, 15.0),
        lin("40 Hz B", -15.0, 15.0),
        lin("50 Hz B", -15.0, 15.0),
        lin("63 Hz B", -15.0, 15.0),
        lin("80 Hz B", -15.0, 15.0),
        lin("100 Hz B", -15.0, 15.0),
        lin("125 Hz B", -15.0, 15.0),
        lin("160 Hz B", -15.0, 15.0),
        lin("200 Hz B", -15.0, 15.0),
        lin("250 Hz B", -15.0, 15.0),
        lin("315 Hz B", -15.0, 15.0),
        lin("400 Hz B", -15.0, 15.0),
        lin("500 Hz B", -15.0, 15.0),
        lin("630 Hz B", -15.0, 15.0),
        lin("800 Hz B", -15.0, 15.0),
        lin("1 kHz B", -15.0, 15.0),
        lin("1.25 kHz B", -15.0, 15.0),
        lin("1.6 kHz B", -15.0, 15.0),
        lin("2 kHz B", -15.0, 15.0),
        lin("2.5 kHz B", -15.0, 15.0),
        lin("3.15 kHz B", -15.0, 15.0),
        lin("4 kHz B", -15.0, 15.0),
        lin("5 kHz B", -15.0, 15.0),
        lin("6.3 kHz B", -15.0, 15.0),
        lin("8 kHz B", -15.0, 15.0),
        lin("10 kHz B", -15.0, 15.0),
        lin("12.5 kHz B", -15.0, 15.0),
        lin("16 kHz B", -15.0, 15.0),
        lin("20 kHz B", -15.0, 15.0),
        lin("Master Level B", -15.0, 15.0),
    ],
};

static TEQ: FxDefinition = FxDefinition {
    parameters: &[
        lin("20 Hz L/R", -15.0, 15.0),
        lin("25 Hz L/R", -15.0, 15.0),
        lin("31.5 Hz L/R", -15.0, 15.0),
        lin("40 Hz L/R", -15.0, 15.0),
        lin("50 Hz L/R", -15.0, 15.0),
        lin("63 Hz L/R", -15.0, 15.0),
        lin("80 Hz L/R", -15.0, 15.0),
        lin("100 Hz L/R", -15.0, 15.0),
        lin("125 Hz L/R", -15.0, 15.0),
        lin("160 Hz L/R", -15.0, 15.0),
        lin("200 Hz L/R", -15.0, 15.0),
        lin("250 Hz L/R", -15.0, 15.0),
        lin("315 Hz L/R", -15.0, 15.0),
        lin("400 Hz L/R", -15.0, 15.0),
        lin("500 Hz L/R", -15.0, 15.0),
        lin("630 Hz L/R", -15.0, 15.0),
        lin("800 Hz L/R", -15.0, 15.0),
        lin("1 kHz L/R", -15.0, 15.0),
        lin("1.25 kHz L/R", -15.0, 15.0),
        lin("1.6 kHz L/R", -15.0, 15.0),
        lin("2 kHz L/R", -15.0, 15.0),
        lin("2.5 kHz L/R", -15.0, 15.0),
        lin("3.15 kHz L/R", -15.0, 15.0),
        lin("4 kHz L/R", -15.0, 15.0),
        lin("5 kHz L/R", -15.0, 15.0),
        lin("6.3 kHz L/R", -15.0, 15.0),
        lin("8 kHz L/R", -15.0, 15.0),
        lin("10 kHz L/R", -15.0, 15.0),
        lin("12.5 kHz L/R", -15.0, 15.0),
        lin("16 kHz L/R", -15.0, 15.0),
        lin("20 kHz L/R", -15.0, 15.0),
        lin("Master Level L/R", -15.0, 15.0),
    ],
};

static DES2: FxDefinition = FxDefinition {
    parameters: &[
        lin("Lo Band A", 0.0, 50.0),
        lin("Hi Band A", 0.0, 50.0),
        lin("Lo Band B", 0.0, 50.0),
        lin("Hi Band B", 0.0, 50.0),
        en("Voice A", &["FEM", "MALE"]),
        en("Voice B", &["FEM", "MALE"]),
    ],
};

static DES: FxDefinition = FxDefinition {
    parameters: &[
        lin("Lo Band L", 0.0, 50.0),
        lin("Hi Band L", 0.0, 50.0),
        lin("Lo Band R", 0.0, 50.0),
        lin("Hi Band R", 0.0, 50.0),
        en("Voice", &["FEM", "MALE"]),
        en("Mode", &["ST", "M/S"]),
    ],
};

static P1A: FxDefinition = FxDefinition {
    parameters: &[
        en("Active", &["OFF", "ON"]),
        lin("Gain", -12.0, 12.0),
        lin("Lo Boost", 0.0, 10.0),
        en("Lo Freq", &["0", "30", "60", "100"]),
        lin("Mid Width", 0.0, 10.0),
        lin("Mid Boost", 0.0, 10.0),
        en("Mid Freq", &["3k", "4k", "5k", "8k", "10k", "12k", "16k"]),
        lin("Hi Attenuation", 0.0, 10.0),
        en("Hi Freq", &["5k", "10k", "20k"]),
        en("Transformer", &["OFF", "ON"]),
    ],
};

static P1A2: FxDefinition = FxDefinition {
    parameters: &[
        en("Active A", &["OFF", "ON"]),
        lin("Gain A", -12.0, 12.0),
        lin("Lo Boost A", 0.0, 10.0),
        en("Lo Freq A", &["0", "30", "60", "100"]),
        lin("Mid Width A", 0.0, 10.0),
        lin("Mid Boost A", 0.0, 10.0),
        en("Mid Freq A", &["3k", "4k", "5k", "8k", "10k", "12k", "16k"]),
        lin("Hi Attenuation A", 0.0, 10.0),
        en("Hi Freq A", &["5k", "10k", "20k"]),
        en("Transformer A", &["OFF", "ON"]),
        en("Active B", &["OFF", "ON"]),
        lin("Gain B", -12.0, 12.0),
        lin("Lo Boost B", 0.0, 10.0),
        en("Lo Freq B", &["0", "30", "60", "100"]),
        lin("Mid Width B", 0.0, 10.0),
        lin("Mid Boost B", 0.0, 10.0),
        en("Mid Freq B", &["3k", "4k", "5k", "8k", "10k", "12k", "16k"]),
        lin("Hi Attenuation B", 0.0, 10.0),
        en("Hi Freq B", &["5k", "10k", "20k"]),
        en("Transformer B", &["OFF", "ON"]),
    ],
};

static PQ5: FxDefinition = FxDefinition {
    parameters: &[
        en("Active", &["OFF", "ON"]),
        lin("Gain", -12.0, 12.0),
        en("Lo Freq", &["200", "300", "500", "700", "1000"]),
        lin("Lo Boost", 0.0, 10.0),
        en(
            "Mid Freq",
            &[
                "200", "300", "500", "700", "1k", "1k5", "2k", "3k", "4k", "5k", "7k",
            ],
        ),
        lin("Mid Boost", 0.0, 10.0),
        en("Hi Freq", &["1k5", "2k", "3k", "4k", "5k"]),
        lin("Hi Boost", 0.0, 10.0),
        en("Transformer", &["OFF", "ON"]),
    ],
};

static PQ5S: FxDefinition = FxDefinition {
    parameters: &[
        en("Active A", &["OFF", "ON"]),
        lin("Gain A", -12.0, 12.0),
        en("Lo Freq A", &["200", "300", "500", "700", "1000"]),
        lin("Lo Boost A", 0.0, 10.0),
        en(
            "Mid Freq A",
            &[
                "200", "300", "500", "700", "1k", "1k5", "2k", "3k", "4k", "5k", "7k",
            ],
        ),
        lin("Mid Boost A", 0.0, 10.0),
        en("Hi Freq A", &["1k5", "2k", "3k", "4k", "5k"]),
        lin("Hi Boost A", 0.0, 10.0),
        en("Transformer A", &["OFF", "ON"]),
        en("Active B", &["OFF", "ON"]),
        lin("Gain B", -12.0, 12.0),
        en("Lo Freq B", &["200", "300", "500", "700", "1000"]),
        lin("Lo Boost B", 0.0, 10.0),
        en(
            "Mid Freq B",
            &[
                "200", "300", "500", "700", "1k", "1k5", "2k", "3k", "4k", "5k", "7k",
            ],
        ),
        lin("Mid Boost B", 0.0, 10.0),
        en("Hi Freq B", &["1k5", "2k", "3k", "4k", "5k"]),
        lin("Hi Boost B", 0.0, 10.0),
        en("Transformer B", &["OFF", "ON"]),
    ],
};

static WAVD: FxDefinition = FxDefinition {
    parameters: &[
        lin("Attack A", -100.0, 100.0),
        lin("Sustain A", -100.0, 100.0),
        lin("Gain A", -24.0, 24.0),
        lin("Attack B", -100.0, 100.0),
        lin("Sustain B", -100.0, 100.0),
        lin("Gain B", -24.0, 24.0),
    ],
};

static LIM: FxDefinition = FxDefinition {
    parameters: &[
        lin("Input Gain", 0.0, 18.0),
        lin("Out Gain", -18.0, 18.0),
        lin("Squeeze", 0.0, 100.0),
        lin("Knee", 0.0, 10.0),
        log("Attack", 0.05, 1.0),
        log("Release", 20.0, 2000.0),
        en("Stereo Link", &["OFF", "ON"]),
        en("Auto Gain", &["OFF", "ON"]),
    ],
};

static CMB: FxDefinition = FxDefinition {
    parameters: &[
        en("Active", &["OFF", "ON"]),
        en("Band Solo", &["OFF", "Bd1", "Bd2", "Bd3", "Bd4", "Bd5"]),
        lin("Mix", 0.0, 100.0),
        lin("Attack", 0.0, 19.0),
        log("Release", 20.0, 3000.0),
        en("Autorelease", &["OFF", "ON"]),
        lin("SBC speed", 0.0, 10.0),
        en("SBC ON", &["OFF", "ON"]),
        lin("Xover", -50.0, 50.0),
        en("Xover Slope", &["12", "48"]),
        en(
            "Ratio",
            &[
                "1.1", "1.2", "1.3", "1.5", "1.7", "2", "2.5", "3", "3.5", "4", "5", "7", "10",
                "LIM",
            ],
        ),
        lin("Threshold", -40.0, 0.0),
        lin("Gain", -10.0, 10.0),
        lin("Band 1 Threshold", -10.0, 10.0),
        lin("Band 1 Gain", -10.0, 10.0),
        en("Band 1 Lock", &["0", "1"]),
        lin("Band 2 Threshold", -10.0, 10.0),
        lin("Band 2 Gain", -10.0, 10.0),
        en("Band 2 Lock", &["0", "1"]),
        lin("Band 3 Threshold", -10.0, 10.0),
        lin("Band 3 Gain", -10.0, 10.0),
        en("Band 3 Lock", &["0", "1"]),
        lin("Band 4 Threshold", -10.0, 10.0),
        lin("Band 4 Gain", -10.0, 10.0),
        en("Band 4 Lock", &["0", "1"]),
        lin("Band 5 Threshold", -10.0, 10.0),
        lin("Band 5 Gain", -10.0, 10.0),
        en("Band 5 Lock", &["0", "1"]),
        en("Meter Mode", &["GR", "SBC", "PEAK"]),
    ],
};

static CMB2: FxDefinition = FxDefinition {
    parameters: &[
        en("Active A", &["OFF", "ON"]),
        en("Band Solo A", &["OFF", "Bd1", "Bd2", "Bd3", "Bd4", "Bd5"]),
        lin("Mix A", 0.0, 100.0),
        lin("Attack A", 0.0, 19.0),
        log("Release A", 20.0, 3000.0),
        en("Autorelease A", &["OFF", "ON"]),
        lin("SBC speed A", 0.0, 10.0),
        en("SBC ON A", &["OFF", "ON"]),
        lin("Xover A", -50.0, 50.0),
        en("Xover Slope A", &["12", "48"]),
        en(
            "Ratio A",
            &[
                "1.1", "1.2", "1.3", "1.5", "1.7", "2", "2.5", "3", "3.5", "4", "5", "7", "10",
                "LIM",
            ],
        ),
        lin("Threshold A", -40.0, 0.0),
        lin("Gain A", -10.0, 10.0),
        lin("Band 1 Threshold A", -10.0, 10.0),
        lin("Band 1 Gain A", -10.0, 10.0),
        en("Band 1 Lock A", &["0", "1"]),
        lin("Band 2 Threshold A", -10.0, 10.0),
        lin("Band 2 Gain A", -10.0, 10.0),
        en("Band 2 Lock A", &["0", "1"]),
        lin("Band 3 Threshold A", -10.0, 10.0),
        lin("Band 3 Gain A", -10.0, 10.0),
        en("Band 3 Lock A", &["0", "1"]),
        lin("Band 4 Threshold A", -10.0, 10.0),
        lin("Band 4 Gain A", -10.0, 10.0),
        en("Band 4 Lock A", &["0", "1"]),
        lin("Band 5 Threshold A", -10.0, 10.0),
        lin("Band 5 Gain A", -10.0, 10.0),
        en("Band 5 Lock A", &["0", "1"]),
        en("Meter Mode A", &["GR", "SBC", "PEAK"]),
        en("Active B", &["OFF", "ON"]),
        en("Band Solo B", &["OFF", "Bd1", "Bd2", "Bd3", "Bd4", "Bd5"]),
        lin("Mix B", 0.0, 100.0),
        lin("Attack B", 0.0, 19.0),
        log("Release B", 20.0, 3000.0),
        en("Autorelease B", &["OFF", "ON"]),
        lin("SBC speed B", 0.0, 10.0),
        en("SBC ON B", &["OFF", "ON"]),
        lin("Xover B", -50.0, 50.0),
        en("Xover Slope B", &["12", "48"]),
        en(
            "Ratio B",
            &[
                "1.1", "1.2", "1.3", "1.5", "1.7", "2", "2.5", "3", "3.5", "4", "5", "7", "10",
                "LIM",
            ],
        ),
        lin("Threshold B", -40.0, 0.0),
        lin("Gain B", -10.0, 10.0),
        lin("Band 1 Threshold B", -10.0, 10.0),
        lin("Band 1 Gain B", -10.0, 10.0),
        en("Band 1 Lock B", &["0", "1"]),
        lin("Band 2 Threshold B", -10.0, 10.0),
        lin("Band 2 Gain B", -10.0, 10.0),
        en("Band 2 Lock B", &["0", "1"]),
        lin("Band 3 Threshold B", -10.0, 10.0),
        lin("Band 3 Gain B", -10.0, 10.0),
        en("Band 3 Lock B", &["0", "1"]),
        lin("Band 4 Threshold B", -10.0, 10.0),
        lin("Band 4 Gain B", -10.0, 10.0),
        en("Band 4 Lock B", &["0", "1"]),
        lin("Band 5 Threshold B", -10.0, 10.0),
        lin("Band 5 Gain B", -10.0, 10.0),
        en("Band 5 Lock B", &["0", "1"]),
        en("Meter Mode B", &["GR", "SBC", "PEAK"]),
    ],
};

static FAC: FxDefinition = FxDefinition {
    parameters: &[
        en("Active", &["OFF", "ON"]),
        lin("Input Gain", -20.0, 20.0),
        lin("Threshold", 0.0, 10.0),
        lin("Time", 0.0, 6.0),
        lin("Bias", 0.0, 100.0),
        lin("Gain", -18.0, 6.0),
        lin("Balance", -100.0, 100.0),
    ],
};

static FAC1M: FxDefinition = FxDefinition {
    parameters: &[
        en("Active", &["OFF", "ON"]),
        lin("Input Gain", -20.0, 20.0),
        lin("Threshold", 0.0, 10.0),
        lin("Time", 0.0, 6.0),
        lin("FAC1M Bias", 0.0, 100.0),
        lin("Gain", -18.0, 6.0),
        lin("Balance", -100.0, 100.0),
        en("Active", &["OFF", "ON"]),
        lin("Input Gain", -20.0, 20.0),
        lin("Threshold", 0.0, 10.0),
        lin("Time", 0.0, 6.0),
        lin("Bias", 0.0, 100.0),
        lin("Gain", -18.0, 6.0),
        lin("Balance", -100.0, 100.0),
    ],
};

static FAC2: FxDefinition = FxDefinition {
    parameters: &[
        en("Active", &["OFF", "ON"]),
        lin("Input Gain", -20.0, 20.0),
        lin("Threshold", 0.0, 10.0),
        lin("Time", 0.0, 6.0),
        lin("FAC1M Bias", 0.0, 100.0),
        lin("Gain", -18.0, 6.0),
        lin("Balance", -100.0, 100.0),
        en("Active", &["OFF", "ON"]),
        lin("Input Gain", -20.0, 20.0),
        lin("Threshold", 0.0, 10.0),
        lin("Time", 0.0, 6.0),
        lin("Bias", 0.0, 100.0),
        lin("Gain", -18.0, 6.0),
        lin("Balance", -100.0, 100.0),
    ],
};

static LEC: FxDefinition = FxDefinition {
    parameters: &[
        en("Active", &["OFF", "ON"]),
        lin("Gain", 0.0, 100.0),
        lin("Peak", 0.0, 100.0),
        en("Mode", &["COMP", "LIM"]),
        lin("Gain", -18.0, 6.0),
    ],
};

static LEC2: FxDefinition = FxDefinition {
    parameters: &[
        en("Active A", &["OFF", "ON"]),
        lin("Gain A", 0.0, 100.0),
        lin("Peak A", 0.0, 100.0),
        en("Mode A", &["COMP", "LIM"]),
        lin("Gain A", -18.0, 6.0),
        en("Active B", &["OFF", "ON"]),
        lin("Gain B", 0.0, 100.0),
        lin("Peak B", 0.0, 100.0),
        en("Mode B", &["COMP", "LIM"]),
        lin("Gain B", -18.0, 6.0),
    ],
};

static ULC: FxDefinition = FxDefinition {
    parameters: &[
        en("Active", &["OFF", "ON"]),
        lin("Input Gain", -48.0, 0.0),
        lin("Out Gain", -48.0, 0.0),
        lin("Attack", 1.0, 7.0),
        lin("Release", 1.0, 7.0),
        en("Ratio", &["4", "8", "12", "20", "ALL"]),
    ],
};

static ULC2: FxDefinition = FxDefinition {
    parameters: &[
        en("Active A", &["OFF", "ON"]),
        lin("Input Gain A", -48.0, 0.0),
        lin("Out Gain A", -48.0, 0.0),
        lin("Attack A", 1.0, 7.0),
        lin("Release A", 1.0, 7.0),
        en("Ratio A", &["4", "8", "12", "20", "ALL"]),
        en("Active B", &["OFF", "ON"]),
        lin("Input Gain B", -48.0, 0.0),
        lin("Out Gain B", -48.0, 0.0),
        lin("Attack B", 1.0, 7.0),
        lin("Release B", 1.0, 7.0),
        en("Ratio B", &["4", "8", "12", "20", "ALL"]),
    ],
};

static ENH2: FxDefinition = FxDefinition {
    parameters: &[
        lin("Out Gain A", -12.0, 12.0),
        lin("Speed A", 0.0, 100.0),
        lin("Bass Gain A", 0.0, 100.0),
        lin("Bass Freq A", 1.0, 50.0),
        lin("Mid Gain A", 0.0, 100.0),
        lin("Mid Freq A", 1.0, 50.0),
        lin("Hi Gain A", 0.0, 100.0),
        lin("Hi Freq A", 1.0, 50.0),
        en("Solo A", &["OFF", "ON"]),
        lin("Out Gain B", -12.0, 12.0),
        lin("Speed B", 0.0, 100.0),
        lin("Bass Gain B", 0.0, 100.0),
        lin("Bass Freq B", 1.0, 50.0),
        lin("Mid Gain B", 0.0, 100.0),
        lin("Mid Freq B", 1.0, 50.0),
        lin("Hi Gain B", 0.0, 100.0),
        lin("Hi Freq B", 1.0, 50.0),
        en("Solo B", &["OFF", "ON"]),
    ],
};

static ENH: FxDefinition = FxDefinition {
    parameters: &[
        lin("Out Gain", -12.0, 12.0),
        lin("Speed", 0.0, 100.0),
        lin("Bass Gain", 0.0, 100.0),
        lin("Bass Freq", 1.0, 50.0),
        lin("Mid Gain", 0.0, 100.0),
        lin("Mid Freq", 1.0, 50.0),
        lin("Hi Gain", 0.0, 100.0),
        lin("Hi Freq", 1.0, 50.0),
        en("Solo", &["OFF", "ON"]),
    ],
};

static EXC2: FxDefinition = FxDefinition {
    parameters: &[
        log("Tune A", 1000.0, 10000.0),
        lin("Peak A", 0.0, 100.0),
        lin("Zero Fill A", 0.0, 100.0),
        lin("Timbre A", -50.0, 50.0),
        lin("Harmonics A", 0.0, 100.0),
        lin("Mix A", 0.0, 100.0),
        en("Solo A", &["OFF", "ON"]),
        log("Tune B", 1000.0, 10000.0),
        lin("Peak B", 0.0, 100.0),
        lin("Zero Fill B", 0.0, 100.0),
        lin("Timbre B", -50.0, 50.0),
        lin("Harmonics B", 0.0, 100.0),
        lin("Mix B", 0.0, 100.0),
        en("Solo B", &["OFF", "ON"]),
    ],
};

static EXC: FxDefinition = FxDefinition {
    parameters: &[
        log("Tune", 1000.0, 10000.0),
        lin("Peak", 0.0, 100.0),
        lin("Zero Fill", 0.0, 100.0),
        lin("Timbre", -50.0, 50.0),
        lin("Harmonics", 0.0, 100.0),
        lin("Mix", 0.0, 100.0),
        en("Solo", &["OFF", "ON"]),
    ],
};

static IMG: FxDefinition = FxDefinition {
    parameters: &[
        lin("Balance", -100.0, 100.0),
        lin("Mono Pan", -100.0, 100.0),
        lin("Stereo Pan", -100.0, 100.0),
        lin("Shv Gain", 0.0, 12.0),
        log("Shv Freq", 100.0, 1000.0),
        log("Shv Q", 1.0, 10.0),
        lin("Out Gain", -12.0, 12.0),
    ],
};

static EDI: FxDefinition = FxDefinition {
    parameters: &[
        en("Active", &["OFF", "ON"]),
        en("Stereo Input", &["ST", "M/S"]),
        en("Stereo Output", &["ST", "M/S"]),
        lin("ST Spread", -50.0, 50.0),
        lin("LMF Spread", -50.0, 50.0),
        lin("Balance", -50.0, 50.0),
        lin("Center Distance", -50.0, 50.0),
        lin("Out Gain", -12.0, 12.0),
    ],
};

static SON: FxDefinition = FxDefinition {
    parameters: &[
        en("Active A", &["OFF", "ON"]),
        lin("Lo Contour A", 0.0, 10.0),
        lin("Process A", 0.0, 10.0),
        lin("Out Gain A", -12.0, 12.0),
        en("Active B", &["OFF", "ON"]),
        lin("Lo Contour B", 0.0, 10.0),
        lin("Process B", 0.0, 10.0),
        lin("Out Gain B", -12.0, 12.0),
    ],
};

static AMP2: FxDefinition = FxDefinition {
    parameters: &[
        lin("Preamp A", 0.0, 10.0),
        lin("Buzz A", 0.0, 10.0),
        lin("Punch A", 0.0, 10.0),
        lin("Crunch A", 0.0, 10.0),
        lin("Drive A", 0.0, 10.0),
        lin("Low A", 0.0, 10.0),
        lin("High A", 0.0, 10.0),
        lin("Level A", 0.0, 10.0),
        en("Cabinet A", &["OFF", "ON"]),
        lin("Preamp B", 0.0, 10.0),
        lin("Buzz B", 0.0, 10.0),
        lin("Punch B", 0.0, 10.0),
        lin("Crunch B", 0.0, 10.0),
        lin("Drive B", 0.0, 10.0),
        lin("Low B", 0.0, 10.0),
        lin("High B", 0.0, 10.0),
        lin("Level B", 0.0, 10.0),
        en("Cabinet B", &["OFF", "ON"]),
    ],
};

static AMP: FxDefinition = FxDefinition {
    parameters: &[
        lin("Preamp", 0.0, 10.0),
        lin("Buzz", 0.0, 10.0),
        lin("Punch", 0.0, 10.0),
        lin("Crunch", 0.0, 10.0),
        lin("Drive", 0.0, 10.0),
        lin("Low", 0.0, 10.0),
        lin("High", 0.0, 10.0),
        lin("Level", 0.0, 10.0),
        en("Cabinet", &["OFF", "ON"]),
    ],
};

static DRV2: FxDefinition = FxDefinition {
    parameters: &[
        lin("Drive A", 0.0, 100.0),
        lin("Even Ear A", 0.0, 50.0),
        lin("Odd Ear A", 0.0, 50.0),
        lin("Gain A", -12.0, 12.0),
        log("Lo Cut A", 20.0, 200.0),
        log("Hi Cut A", 4000.0, 20000.0),
        lin("Lo Gain A", -12.0, 12.0),
        log("Lo Freq A", 50.0, 400.0),
        lin("Hi Gain A", -12.0, 12.0),
        log("Hi Freq A", 1000.0, 10000.0),
        lin("Drive B", 0.0, 100.0),
        lin("Even Ear B", 0.0, 50.0),
        lin("Odd Ear B", 0.0, 50.0),
        lin("Gain B", -12.0, 12.0),
        log("Lo Cut B", 20.0, 200.0),
        log("Hi Cut B", 4000.0, 20000.0),
        lin("Lo Gain B", -12.0, 12.0),
        log("Lo Freq B", 50.0, 400.0),
        lin("Hi Gain B", -12.0, 12.0),
        log("Hi Freq B", 1000.0, 10000.0),
    ],
};

static DRV: FxDefinition = FxDefinition {
    parameters: &[
        lin("Drive", 0.0, 100.0),
        lin("Even Ear", 0.0, 50.0),
        lin("Odd Ear", 0.0, 50.0),
        lin("Gain", -12.0, 12.0),
        log("Lo Cut", 20.0, 200.0),
        log("Hi Cut", 4000.0, 20000.0),
        lin("Lo Gain", -12.0, 12.0),
        log("Lo Freq", 50.0, 400.0),
        lin("Hi Gain", -12.0, 12.0),
        log("Hi Freq", 1000.0, 10000.0),
    ],
};

static PIT2: FxDefinition = FxDefinition {
    parameters: &[
        lin("Semitone A", -12.0, 12.0),
        lin("Cent A", -50.0, 50.0),
        log("Delay A", 1.0, 100.0),
        log("Lo Cut A", 10.0, 500.0),
        log("Hi Cut A", 2000.0, 20000.0),
        lin("Mix A", 0.0, 100.0),
        lin("Semitone B", -12.0, 12.0),
        lin("Cent B", -50.0, 50.0),
        log("Delay B", 1.0, 100.0),
        log("Lo Cut B", 10.0, 500.0),
        log("Hi Cut B", 2000.0, 20000.0),
        lin("Mix B", 0.0, 100.0),
    ],
};

static PIT: FxDefinition = FxDefinition {
    parameters: &[
        lin("Semitone", -12.0, 12.0),
        lin("Cent", -50.0, 50.0),
        log("Delay", 1.0, 100.0),
        log("Lo Cut", 10.0, 500.0),
        log("Hi Cut", 2000.0, 20000.0),
        lin("Mix", 0.0, 100.0),
    ],
};

pub(crate) const FX1_4_NAMES: &[&str] = &[
    "HALL", "AMBI", "RPLT", "ROOM", "CHAM", "PLAT", "VREV", "VRM", "GATE", "RVRS", "DLY", "3TAP",
    "4TAP", "CRS", "FLNG", "PHAS", "DIMC", "FILT", "ROTA", "PAN", "SUB", "D/RV", "CR/R", "FL/R",
    "D/CR", "D/FL", "MODD", "GEQ2", "GEQ", "TEQ2", "TEQ", "DES2", "DES", "P1A", "P1A2", "PQ5",
    "PQ5S", "WAVD", "LIM", "CMB", "CMB2", "FAC", "FAC1M", "FAC2", "LEC", "LEC2", "ULC", "ULC2",
    "ENH2", "ENH", "EXC2", "EXC", "IMG", "EDI", "SON", "AMP2", "AMP", "DRV2", "DRV", "PIT2", "PIT",
];

pub(crate) const FX5_8_NAMES: &[&str] = &[
    "GEQ2", "GEQ", "TEQ2", "TEQ", "DES2", "DES", "P1A", "P1A2", "PQ5", "PQ5S", "WAVD", "LIM",
    "FAC", "FAC1M", "FAC2", "LEC", "LEC2", "ULC", "ULC2", "ENH2", "ENH", "EXC2", "EXC", "IMG",
    "EDI", "SON", "AMP2", "AMP", "DRV2", "DRV", "PHAS", "FILT", "PAN", "SUB",
];

pub(crate) fn fx_type_name(slot: u8, fx_type: i32) -> &'static str {
    let names = if slot <= 4 { FX1_4_NAMES } else { FX5_8_NAMES };
    names.get(fx_type as usize).copied().unwrap_or("UNKNOWN")
}

/// Human-readable effect names from the X32 OSC protocol reference.
pub(crate) fn fx_type_label(slot: u8, fx_type: i32) -> &'static str {
    match fx_type_name(slot, fx_type) {
        "HALL" => "Hall Reverb",
        "AMBI" => "Ambiance Reverb",
        "RPLT" => "Rich Plate Reverb",
        "ROOM" => "Room Reverb",
        "CHAM" => "Chamber Reverb",
        "PLAT" => "Plate Reverb",
        "VREV" => "Vintage Reverb",
        "GATE" => "Gated Reverb",
        "DLY" => "Stereo Delay",
        "3TAP" => "3-Tap Delay",
        "4TAP" => "4-Tap Delay",
        "CRS" => "Stereo Chorus",
        "FLNG" => "Stereo Flanger",
        "PHAS" => "Stereo Phaser",
        "DIMC" => "Dimensional Chorus",
        "FILT" => "Mood Filter",
        "ROTA" => "Rotary Speaker",
        "PAN" => "Tremolo / Panner",
        "SUB" => "Suboctaver",
        "D/RV" => "Delay / Chamber",
        "CR/R" => "Chorus / Chamber",
        "FL/R" => "Flanger / Chamber",
        "D/CR" => "Delay / Chorus",
        "D/FL" => "Delay / Flanger",
        "MODD" => "Modulation Delay",
        "GEQ2" => "Dual Graphic Equalizer / True Dual Graphic Equalizer",
        "GEQ" => "Graphic Equalizer / True Graphic Equalizer",
        "DES2" => "Stereo / Dual De-Esser",
        "LIM" => "Precision Limiter",
        "P1A" | "P1A2" => "Stereo / Dual Program EQ",
        "PQ5" | "PQ5S" => "Stereo / Dual Midrange EQ",
        "CMB" | "CMB2" => "Stereo / Dual Combinator",
        "FAC" | "FAC1M" | "FAC2" => "Stereo / Dual Fair Compressor",
        "LEC" | "LEC2" => "Stereo / Dual Leisure Compressor",
        "EDI" => "Edison EX1",
        "ULC" | "ULC2" => "Stereo / Dual Ultimo Compressor",
        other => other,
    }
}

pub(crate) fn fx_definition(name: &str) -> Option<&'static FxDefinition> {
    match name {
        "HALL" => Some(&HALL),
        "AMBI" => Some(&AMBI),
        "RPLT" => Some(&RPLT),
        "ROOM" => Some(&ROOM),
        "CHAM" => Some(&CHAM),
        "PLAT" => Some(&PLAT),
        "VREV" => Some(&VREV),
        "VRM" => Some(&VRM),
        "GATE" => Some(&GATE),
        "RVRS" => Some(&RVRS),
        "DLY" => Some(&DLY),
        "3TAP" => Some(&FX_3TAP),
        "4TAP" => Some(&FX_4TAP),
        "CRS" => Some(&CRS),
        "FLNG" => Some(&FLNG),
        "PHAS" => Some(&PHAS),
        "DIMC" => Some(&DIMC),
        "FILT" => Some(&FILT),
        "ROTA" => Some(&ROTA),
        "PAN" => Some(&PAN),
        "SUB" => Some(&SUB),
        "D/RV" => Some(&D_RV),
        "CR/R" => Some(&CR_R),
        "FL/R" => Some(&FL_R),
        "D/CR" => Some(&D_CR),
        "D/FL" => Some(&D_FL),
        "MODD" => Some(&MODD),
        "GEQ2" => Some(&GEQ2),
        "GEQ" => Some(&GEQ),
        "TEQ2" => Some(&TEQ2),
        "TEQ" => Some(&TEQ),
        "DES2" => Some(&DES2),
        "DES" => Some(&DES),
        "P1A" => Some(&P1A),
        "P1A2" => Some(&P1A2),
        "PQ5" => Some(&PQ5),
        "PQ5S" => Some(&PQ5S),
        "WAVD" => Some(&WAVD),
        "LIM" => Some(&LIM),
        "CMB" => Some(&CMB),
        "CMB2" => Some(&CMB2),
        "FAC" => Some(&FAC),
        "FAC1M" => Some(&FAC1M),
        "FAC2" => Some(&FAC2),
        "LEC" => Some(&LEC),
        "LEC2" => Some(&LEC2),
        "ULC" => Some(&ULC),
        "ULC2" => Some(&ULC2),
        "ENH2" => Some(&ENH2),
        "ENH" => Some(&ENH),
        "EXC2" => Some(&EXC2),
        "EXC" => Some(&EXC),
        "IMG" => Some(&IMG),
        "EDI" => Some(&EDI),
        "SON" => Some(&SON),
        "AMP2" => Some(&AMP2),
        "AMP" => Some(&AMP),
        "DRV2" => Some(&DRV2),
        "DRV" => Some(&DRV),
        "PIT2" => Some(&PIT2),
        "PIT" => Some(&PIT),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_effect_type_has_a_definition() {
        for (slot, names) in [(1, FX1_4_NAMES), (5, FX5_8_NAMES)] {
            for (index, name) in names.iter().enumerate() {
                assert_eq!(fx_type_name(slot, index as i32), *name);
                let definition = fx_definition(name).expect("effect definition");
                assert!(!definition.parameters.is_empty(), "{name}");
            }
        }
    }

    #[test]
    fn large_effects_include_all_documented_parameters() {
        assert_eq!(fx_definition("GEQ2").unwrap().parameters.len(), 64);
        assert_eq!(fx_definition("TEQ").unwrap().parameters.len(), 32);
        assert_eq!(fx_definition("CMB").unwrap().parameters.len(), 29);
        assert_eq!(fx_definition("CMB2").unwrap().parameters.len(), 58);
        assert_eq!(fx_definition("DRV2").unwrap().parameters.len(), 20);
    }

    #[test]
    fn numeric_parameters_decode_normalized_values() {
        let linear = FxParameterKind::Linear {
            min: -15.0,
            max: 15.0,
        };
        assert_eq!(linear.value(0.0), -15.0);
        assert_eq!(linear.value(0.5), 0.0);
        assert_eq!(linear.value(1.0), 15.0);
        let logarithmic = FxParameterKind::Logarithmic {
            min: 20.0,
            max: 20_000.0,
        };
        assert!((logarithmic.value(0.5) - 632.455_5).abs() < 0.001);
    }
}
