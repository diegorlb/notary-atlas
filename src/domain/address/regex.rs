use std::sync::LazyLock;

use regex::{Regex, RegexSet};

pub static CLEANUP_CHARS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"[#\\/|@_"']"#).unwrap());

pub static CLEANUP_SET: LazyLock<RegexSet> = LazyLock::new(|| {
    RegexSet::new([
        // PARENTHESES
        r"\([^)]*(?:\)|$)",
        // DASH_ALIAS
        r"\s*-[^-]+-\s*",
        // KM_NUMBER_PATTERN
        r"\bkm\b\.?\s*\d+(?:[.,]\d+)?",
        // DANGLING_NUMBER
        r"\b(?:n[º°.]?|nro\.?|numero)[\s,;.]*$",
        // FLOOR_AFTER_NUMBER
        r"\b(\d+)\s+(?:\d+\s*(?:[º°ª]|er\b)|(?:p|pb|pa|of)\b).*$",
        // UNIT
        r"(?is)\s*\b(?:dpto|depto|dto|piso|oficina|ofic|local|uf)\b\.?.*$",
    ])
    .unwrap()
});

pub static MISSING_SPACE_DOT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-zA-Z])\.([a-zA-Z])").unwrap());

pub static FLOOR_OFFICE_NOTATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:pb|pa|p\.?b\.?|p\.?a\.?|p\.?\s*\d+|of\.?\s*[\d/]+)\b").unwrap()
});

pub static DASH_ALIAS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*-[^-]+-\s*").unwrap());

pub static NUMBER_PREFIX_FIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bn(?:ro|umero|[º°])?\.?\s*(\d+)").unwrap());

pub static NO_NUMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bs\s*/\s*n\b|\bsin\s+n(?:umero|ro|[º°])?(?:\b|\s|$)|\bsin\s+n[ .]").unwrap()
});

pub static NUMBER_ANCHORED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bn(?:ro|umero|[º°])?\.?\s*(\d+)").unwrap());

pub static NUMBER_TRAILING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(\d+)(?:\s*(?:bis|[a-z]))?\s*$").unwrap());

pub static PARENTHESES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\([^)]*(?:\)|$)").unwrap());

pub static KM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bkm\b\.?\s*\d+(?:[.,]\d+)?").unwrap());

pub static KM_NUMBER_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bkm\.?\s*(\d+(?:[.,]\d+)?)").unwrap());

pub static DANGLING_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:n[º°.]?|nro\.?|numero)[\s,;.]*$").unwrap());

pub static FLOOR_AFTER_NUMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d+)\s+(?:\d+\s*(?:[º°ª]|er\b)|(?:p|pb|pa|of)\b).*$").unwrap()
});

pub static BUILDING: LazyLock<Regex> = LazyLock::new(|| {
    const WORDS: &str = "edificio|edif|ed|timbre|cuerpo|loc|local|piso|torre|trre|lote";

    Regex::new(&format!(
        r",|\b\d+\s*(?:er\b|[º°ª])\.?\s*(?:{WORDS})\b|\b(?:{WORDS})\b"
    ))
    .unwrap()
});

pub static TAIL_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b\d{2,}\b").unwrap());

pub static UNIT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\s*\b(?:dpto|depto|dto|piso|oficina|ofic|local|uf)\b\.?.*$").unwrap()
});

pub static INTERSECTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\s*\besq(?:uina)?\b\.?.*?(\b(?:n[°º.]?\s*)?\d.*)?$").unwrap()
});

pub static BETWEEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\s*\b(?:entre\s|e/)\s*.*?(\bn(?:ro|umero|[º° ])?[ .]?\s*\d.*)?$").unwrap()
});

pub static IMPLICIT_BETWEEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(\b\d+)\s+\d+\s+y\s+\d+\b").unwrap());

pub static STREET_TYPE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    #[rustfmt::skip]
    const STREET_TYPES: &[&str] = &[
        "calle", "av", "avda", "avenida", "diag",
        "diagonal", "pje", "pasaje", "bv", "bvd",
        "boulevard",
    ];

    Regex::new(&format!(r"(?i)\b(?:{})\b\.?\s*", STREET_TYPES.join("|"))).unwrap()
});

pub static NUMERIC_CROSS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)\s+\d+$").unwrap());
