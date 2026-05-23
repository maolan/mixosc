#!/usr/bin/env python3
"""Generate src/parameters.rs from X32 OSC specification JSON files."""

import json
import re
from collections import defaultdict


def normalize_pattern(path: str) -> str:
    """Replace digits with N and normalize range notations in a path."""
    parts = path.split("/")
    result = []
    for p in parts:
        if p.isdigit():
            result.append("N")
        elif "1...n" in p or "1…n" in p:
            result.append("N")
        elif re.match(r"^\d+[-‐]\d+$", p):
            result.append("N")
        else:
            result.append(p)
    return "/".join(result)


def load_data():
    with open("/home/meka/Files/OSC/x32_osc_endpoints.json") as f:
        endpoints = json.load(f)
    with open("/home/meka/Files/OSC/x32_osc_full_extract.json") as f:
        full_extract = json.load(f)
    return endpoints, full_extract


def build_type_map(endpoints):
    type_map = {}
    for item in endpoints:
        path = item["path"]
        norm = normalize_pattern(path)
        type_map[norm] = item["type"]
    return type_map


def infer_type(pattern: str, type_map: dict) -> str:
    t = type_map.get(pattern)
    if t:
        return t

    suffix = pattern.split("/")[-1]
    # Bool-like suffixes
    if suffix in ("on", "invert", "hpon", "st", "mono", "auto"):
        return "bool"
    # Float-like suffixes
    if suffix in (
        "fader",
        "pan",
        "level",
        "time",
        "trim",
        "hpf",
        "thr",
        "range",
        "attack",
        "hold",
        "release",
        "knee",
        "mgain",
        "mix",
        "f",
        "g",
        "q",
        "weight",
    ):
        return "float"
    # Int-like suffixes
    if suffix in (
        "icon",
        "source",
        "keysrc",
        "color",
        "type",
        "mode",
        "det",
        "env",
        "pos",
        "hpslope",
        "filter",
        "sel",
        "group",
        "index",
        "src",
        "delay",
    ):
        return "int"
    if suffix == "name":
        return "string"
    return "unknown"


def extract_patterns(full_extract):
    all_patterns = defaultdict(int)
    for item in full_extract["concrete_paths"]:
        path = item["path"]
        norm = normalize_pattern(path)
        all_patterns[norm] += 1
    return all_patterns


def is_valid_parameter(pattern: str, all_patterns: dict) -> bool:
    """Check if a pattern looks like a real X32 parameter path."""
    # Must have at least 3 segments (e.g., /ch/N/config/name)
    parts = pattern.strip("/").split("/")
    if len(parts) < 3:
        return False
    # Exclude known artifacts
    artifacts = {"type", "par", "source", "delay", "dyn", "eq", "gate", "mix", "preamp", "filter"}
    if len(parts) == 1 and parts[0] in artifacts:
        return False
    # Exclude paths with weird characters
    if "…" in pattern or "‐" in pattern:
        return False
    if any("[" in part and "]" in part for part in parts):
        return False
    # Must be a leaf (no children)
    for other in all_patterns:
        if other != pattern and other.startswith(pattern + "/"):
            return False
    return True


def parse_pattern_params(pattern: str) -> list[tuple[str, str]]:
    """Extract parameter names and types from a pattern."""
    params = []
    parts = pattern.split("/")
    idx = 1
    for part in parts:
        if part == "N":
            params.append((f"n{idx}", "u8"))
            idx += 1
    return params


def function_name(pattern: str) -> str:
    """Generate a snake_case Rust function name from a pattern."""
    parts = pattern.strip("/").split("/")
    result = []
    for part in parts:
        if part == "N":
            continue
        part = part.replace("[", "").replace("]", "").replace("…", "_").replace("-", "_")
        # Convert camelCase/PascalCase to snake_case
        part = re.sub(r'([a-z0-9])([A-Z])', r'\1_\2', part)
        result.append(part.lower())
    return "_".join(result)


def build_path_expr(pattern: str) -> str:
    """Build a Rust format! expression for a pattern."""
    parts = pattern.strip("/").split("/")
    param_idx = 1
    format_parts = []
    for part in parts:
        if part == "N":
            format_parts.append(f"{{n{param_idx}:02}}")
            param_idx += 1
        else:
            format_parts.append(part)
    return "/".join(format_parts)


def generate_path_builders(leaves: list, leaf_types: dict) -> str:
    lines = []
    lines.append("/// Path builder functions for all X32 OSC parameters.")
    lines.append("pub mod path {")

    by_prefix = defaultdict(list)
    for pattern in leaves:
        parts = pattern.strip("/").split("/")
        prefix = parts[0] if parts else "root"
        by_prefix[prefix].append(pattern)

    seen_names = set()
    for prefix in sorted(by_prefix.keys()):
        patterns = sorted(by_prefix[prefix])
        lines.append(f"    // {prefix}")
        for pattern in patterns:
            params = parse_pattern_params(pattern)
            name = function_name(pattern)
            # Handle duplicate names
            orig_name = name
            counter = 2
            while name in seen_names:
                name = f"{orig_name}_{counter}"
                counter += 1
            seen_names.add(name)

            path_expr = build_path_expr(pattern)
            param_str = ", ".join(f"{n}: {t}" for n, t in params)
            type_comment = leaf_types.get(pattern, "unknown")
            lines.append(f"    /// {pattern} ({type_comment})")
            lines.append(f'    pub fn {name}({param_str}) -> String {{')
            if params:
                lines.append(f'        format!("/{path_expr}")')
            else:
                lines.append(f'        String::from("/{path_expr}")')
            lines.append("    }")
            lines.append("")

    lines.append("}")
    return "\n".join(lines)


def generate_osc_value() -> str:
    return """
/// A typed OSC value.
#[derive(Debug, Clone, PartialEq)]
pub enum OscValue {
    Float(f32),
    Int(i32),
    String(String),
    Bool(bool),
}

impl OscValue {
    /// Create a float value.
    pub fn float(v: f32) -> Self {
        Self::Float(v)
    }

    /// Create an int value.
    pub fn int(v: i32) -> Self {
        Self::Int(v)
    }

    /// Create a string value.
    pub fn string(v: impl Into<String>) -> Self {
        Self::String(v.into())
    }

    /// Create a bool value.
    pub fn bool(v: bool) -> Self {
        Self::Bool(v)
    }
}
"""


def generate_packet_builders() -> str:
    return """
/// Build an OSC query packet (get value).
pub fn build_get(path: &str) -> Vec<u8> {
    osc_string(path)
}

/// Build an OSC set packet with a typed value.
pub fn build_set(path: &str, value: OscValue) -> Vec<u8> {
    match value {
        OscValue::Float(v) => osc_float_message(path, v),
        OscValue::Int(v) => osc_int_message(path, v),
        OscValue::String(v) => osc_string_message(path, &v),
        OscValue::Bool(v) => osc_int_message(path, i32::from(v)),
    }
}

/// Parse an OSC response packet into (path, value).
pub fn parse_osc_value(packet: &[u8]) -> Option<(String, OscValue)> {
    let path = osc_address(packet)?;
    let mut offset = osc_padded_len(packet)?;
    let type_tag_end = packet.get(offset..)?.iter().position(|byte| *byte == 0)?;
    let type_tag = std::str::from_utf8(packet.get(offset..offset + type_tag_end)?).ok()?;
    let type_tag_len = osc_padded_len(packet.get(offset..)?)?;
    offset += type_tag_len;

    match type_tag {
        ",f" => {
            let value_bytes: [u8; 4] = packet.get(offset..offset + 4)?.try_into().ok()?;
            Some((path.to_owned(), OscValue::Float(f32::from_bits(u32::from_be_bytes(value_bytes)))))
        }
        ",i" => {
            let value_bytes: [u8; 4] = packet.get(offset..offset + 4)?.try_into().ok()?;
            Some((path.to_owned(), OscValue::Int(i32::from_be_bytes(value_bytes))))
        }
        ",s" => {
            let value_bytes = packet.get(offset..)?;
            let value_end = value_bytes.iter().position(|byte| *byte == 0)?;
            let value = std::str::from_utf8(&value_bytes[..value_end]).ok()?;
            Some((path.to_owned(), OscValue::String(value.to_owned())))
        }
        _ => None,
    }
}


"""


def make_variant_name(param_parts: list[str]) -> str:
    """Create a Rust enum variant name from parameter path parts."""
    return "".join(word.capitalize() for word in param_parts)


def generate_typed_parameters(leaves: list, leaf_types: dict) -> str:
    """Generate typed parameter enums for main strip types."""
    strip_params = defaultdict(list)
    for pattern in leaves:
        parts = pattern.strip("/").split("/")
        if len(parts) < 2:
            continue
        prefix = parts[0]
        strip_params[prefix].append(pattern)

    lines = []

    strip_types = {
        "ch": ("Channel", "channel", 1, 32),
        "bus": ("Bus", "bus", 1, 16),
        "auxin": ("AuxIn", "auxin", 1, 8),
        "fxrtn": ("FxRtn", "fxrtn", 1, 8),
        "mtx": ("Mtx", "mtx", 1, 6),
        "dca": ("Dca", "dca", 1, 8),
        "fx": ("Fx", "fx", 1, 8),
        "headamp": ("Headamp", "headamp", 0, 127),
    }

    for prefix, (type_name, _path_prefix, _min, _max) in strip_types.items():
        patterns = strip_params.get(prefix, [])
        if not patterns:
            continue

        lines.append(f"/// Parameters for {type_name} strips.")
        lines.append(f"#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
        lines.append(f"pub enum {type_name}Param {{")

        seen_variants = set()
        for pattern in sorted(patterns):
            parts = pattern.strip("/").split("/")
            if len(parts) < 3:
                continue
            param_parts = parts[2:]

            variant_parts = []
            has_index = False
            for p in param_parts:
                if p == "N":
                    has_index = True
                else:
                    variant_parts.append(p)

            variant_name = make_variant_name(variant_parts)
            if variant_name in seen_variants:
                continue
            seen_variants.add(variant_name)

            if has_index:
                lines.append(f"    {variant_name} {{ index: u8 }},")
            else:
                lines.append(f"    {variant_name},")

        lines.append("}")
        lines.append("")

        lines.append(f"impl {type_name}Param {{")
        lines.append(f"    /// Return the OSC path for this parameter on a given strip.")
        lines.append(f"    pub fn path(&self, strip: u8) -> String {{")
        lines.append(f"        match self {{")

        seen_variants = set()
        for pattern in sorted(patterns):
            parts = pattern.strip("/").split("/")
            if len(parts) < 3:
                continue
            param_parts = parts[2:]

            variant_parts = []
            has_index = False
            for p in param_parts:
                if p == "N":
                    has_index = True
                else:
                    variant_parts.append(p)

            variant_name = make_variant_name(variant_parts)
            if variant_name in seen_variants:
                continue
            seen_variants.add(variant_name)

            path_expr = build_path_expr(pattern)
            path_expr = path_expr.replace("{n1:02}", "{strip:02}")
            path_expr = path_expr.replace("{n2:02}", "{index:02}")
            path_expr = path_expr.replace("{n3:02}", "{index:02}")

            if has_index:
                lines.append(f'            Self::{variant_name} {{ index }} => format!("/{path_expr}"),')
            else:
                lines.append(f'            Self::{variant_name} => format!("/{path_expr}"),')

        lines.append("        }")
        lines.append("    }")
        lines.append("}")
        lines.append("")

    return "\n".join(lines)


def generate_main_parameters(leaves: list, leaf_types: dict) -> str:
    """Generate MainStereo and MainMono parameter enums."""
    lines = []

    for prefix, type_name in [("main/st", "MainStereo"), ("main/m", "MainMono")]:
        patterns = [p for p in leaves if p.startswith(f"/{prefix}/")]
        if not patterns:
            continue

        lines.append(f"/// Parameters for {type_name}.")
        lines.append(f"#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
        lines.append(f"pub enum {type_name}Param {{")

        seen_variants = set()
        for pattern in sorted(patterns):
            parts = pattern.strip("/").split("/")
            param_parts = parts[3:]  # Skip main/st or main/m

            variant_parts = []
            has_index = False
            for p in param_parts:
                if p == "N":
                    has_index = True
                else:
                    variant_parts.append(p)

            variant_name = make_variant_name(variant_parts)
            if variant_name in seen_variants:
                continue
            seen_variants.add(variant_name)

            if has_index:
                lines.append(f"    {variant_name} {{ index: u8 }},")
            else:
                lines.append(f"    {variant_name},")

        lines.append("}")
        lines.append("")

        lines.append(f"impl {type_name}Param {{")
        lines.append(f"    pub fn path(&self) -> String {{")
        lines.append(f"        match self {{")

        seen_variants = set()
        for pattern in sorted(patterns):
            parts = pattern.strip("/").split("/")
            param_parts = parts[3:]

            variant_parts = []
            has_index = False
            for p in param_parts:
                if p == "N":
                    has_index = True
                else:
                    variant_parts.append(p)

            variant_name = make_variant_name(variant_parts)
            if variant_name in seen_variants:
                continue
            seen_variants.add(variant_name)

            path_expr = build_path_expr(pattern)
            path_expr = path_expr.replace("{n1:02}", "{index:02}")
            path_expr = path_expr.replace("{n2:02}", "{index:02}")

            if has_index:
                lines.append(f'            Self::{variant_name} {{ index }} => format!("/{path_expr}"),')
            else:
                lines.append(f'            Self::{variant_name} => String::from("/{path_expr}"),')

        lines.append("        }")
        lines.append("    }")
        lines.append("}")
        lines.append("")

    return "\n".join(lines)


def generate_outputs_parameters(leaves: list, leaf_types: dict) -> str:
    """Generate output parameter enums."""
    lines = []

    output_types = defaultdict(list)
    for pattern in leaves:
        if pattern.startswith("/outputs/"):
            parts = pattern.strip("/").split("/")
            if len(parts) >= 3:
                output_types[parts[1]].append(pattern)

    for output_type, patterns in output_types.items():
        type_name = f"Output{output_type.capitalize()}Param"
        lines.append(f"/// Parameters for {output_type} outputs.")
        lines.append(f"#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
        lines.append(f"pub enum {type_name} {{")

        seen_variants = set()
        for pattern in sorted(patterns):
            parts = pattern.strip("/").split("/")
            param_parts = parts[3:]  # Skip outputs/type

            variant_parts = []
            has_index = False
            for p in param_parts:
                if p == "N":
                    has_index = True
                else:
                    variant_parts.append(p)

            variant_name = make_variant_name(variant_parts)
            if variant_name in seen_variants:
                continue
            seen_variants.add(variant_name)

            if has_index:
                lines.append(f"    {variant_name} {{ index: u8 }},")
            else:
                lines.append(f"    {variant_name},")

        lines.append("}")
        lines.append("")

        lines.append(f"impl {type_name} {{")
        lines.append(f"    pub fn path(&self, output: u8) -> String {{")
        lines.append(f"        match self {{")

        seen_variants = set()
        for pattern in sorted(patterns):
            parts = pattern.strip("/").split("/")
            param_parts = parts[3:]

            variant_parts = []
            has_index = False
            for p in param_parts:
                if p == "N":
                    has_index = True
                else:
                    variant_parts.append(p)

            variant_name = make_variant_name(variant_parts)
            if variant_name in seen_variants:
                continue
            seen_variants.add(variant_name)

            path_expr = build_path_expr(pattern)
            path_expr = path_expr.replace("{n1:02}", "{output:02}")
            path_expr = path_expr.replace("{n2:02}", "{index:02}")

            if has_index:
                lines.append(f'            Self::{variant_name} {{ index }} => format!("/{path_expr}"),')
            else:
                lines.append(f'            Self::{variant_name} => format!("/{path_expr}"),')

        lines.append("        }")
        lines.append("    }")
        lines.append("}")
        lines.append("")

    return "\n".join(lines)


def generate():
    endpoints, full_extract = load_data()
    type_map = build_type_map(endpoints)
    all_patterns = extract_patterns(full_extract)

    leaves = []
    for pattern in sorted(all_patterns.keys()):
        if is_valid_parameter(pattern, all_patterns):
            leaves.append(pattern)

    leaf_types = {p: infer_type(p, type_map) for p in leaves}

    output = []
    output.append("// Auto-generated from X32 OSC specification.")
    output.append("// Do not edit manually.")
    output.append("")
    output.append("use crate::x32::{osc_address, osc_float_message, osc_int_message, osc_padded_len, osc_string, osc_string_message};")
    output.append("")

    output.append(generate_osc_value())
    output.append(generate_packet_builders())
    output.append(generate_path_builders(leaves, leaf_types))
    output.append(generate_typed_parameters(leaves, leaf_types))
    output.append(generate_main_parameters(leaves, leaf_types))
    output.append(generate_outputs_parameters(leaves, leaf_types))

    return "\n".join(output)


if __name__ == "__main__":
    code = generate()
    with open("src/parameters.rs", "w") as f:
        f.write(code)
    print(f"Generated src/parameters.rs ({len(code)} bytes)")
