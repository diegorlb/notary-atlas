mod regex;

use itertools::Itertools as _;

use super::address::regex::{
    BETWEEN, BUILDING, DANGLING_NUMBER, DASH_ALIAS, FLOOR_AFTER_NUMBER, FLOOR_OFFICE_NOTATION,
    IMPLICIT_BETWEEN, INTERSECTION, KM, KM_NUMBER_PATTERN, MISSING_SPACE_DOT_PATTERN, NO_NUMBER,
    NUMBER_ANCHORED, NUMBER_PREFIX_FIX, NUMBER_TRAILING, NUMERIC_CROSS, PARENTHESES, QUOTES,
    STREET_TYPE_PATTERN, SYMBOLS, TAIL_NUMBER, UNIT,
};

pub fn word_to_digit(word: &str) -> &str {
    match word {
        "nueve" => "9",
        "veinticinco" => "25",

        other => other,
    }
}

pub fn unaccent_and_lower(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|character| match character {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            'ñ' => 'n',

            character => character,
        })
        .collect()
}

pub fn expand_abbreviations(text: &str) -> String {
    let text = MISSING_SPACE_DOT_PATTERN.replace_all(text, "$1. $2");
    let text = MISSING_SPACE_DOT_PATTERN.replace_all(&text, "$1. $2");
    let base = unaccent_and_lower(&text);
    let base = NUMBER_PREFIX_FIX.replace_all(&base, " nro $1");
    let base = FLOOR_OFFICE_NOTATION.replace_all(&base, " ");

    base.split_whitespace()
        .map(|token| match token {
            token if token.len() == 1 && !token.chars().next().unwrap().is_ascii_digit() => "",

            token
                if token.len() == 2
                    && token.ends_with('.')
                    && token.chars().next().unwrap().is_alphabetic() =>
            {
                ""
            }

            "gdor." | "gdor" => "gobernador",
            "gral." | "gral" => "general",
            "cnel." | "cnel" => "coronel",
            "tte." | "tte" => "teniente",
            "cap." | "cap" => "capitan",

            "av." | "av" | "avda." | "avda" => "avenida",
            "pje." | "pje" => "pasaje",
            "diag." | "diag" => "diagonal",
            "pte." | "pte" => "presidente",
            "alte." | "alte" => "almirante",
            "bme." | "bme" => "bartolome",
            "lib." | "lib" => "libertador",
            "blvd." | "blvd" => "boulevard",

            "dr." | "dr" | "dra." | "dra" | "col." | "col" | "h." | "h" | "hipolito" => "",

            "malv." | "malv" => "malvinas",
            "nro" | "nro." | "nº" | "n°" => "nro",

            other => word_to_digit(other),
        })
        .filter(|token| !token.is_empty())
        .join(" ")
}

pub fn simplify_for_match(text: &str) -> String {
    let text = MISSING_SPACE_DOT_PATTERN.replace_all(text, "$1. $2");
    let text = MISSING_SPACE_DOT_PATTERN.replace_all(&text, "$1. $2");
    let base = unaccent_and_lower(&text);

    base.split(|c: char| !c.is_alphanumeric())
        .filter_map(|word| match word {
            "" | "av" | "avda" | "avenida" | "calle" | "pasaje" | "pje" | "diag" | "diagonal"
            | "blvd" | "bv" | "bvd" | "boulevard" | "partido" | "pdo" | "dpto" | "departamento"
            | "gral" | "grl" | "general" | "cnel" | "cnl" | "coronel" | "tte" | "tnte"
            | "teniente" | "cap" | "cpt" | "capt" | "capitan" | "alte" | "almte" | "alm"
            | "almirante" | "brig" | "brigadier" | "pte" | "pdte" | "pres" | "presidente"
            | "gdor" | "gob" | "gobernador" | "dr" | "dra" | "doctor" | "ing" | "ingeniero"
            | "lic" | "arq" | "prof" | "sgto" | "sarg" | "cdte" | "cmte" | "mons" | "pbro"
            | "n" | "nro" | "numero" => None,

            word if word.len() == 1 => None,

            word => Some(word_to_digit(word)),
        })
        .join(" ")
}

pub fn parse_address(address: &str) -> (String, String, Option<String>) {
    let address = SYMBOLS.replace_all(address, " ");
    let address = expand_abbreviations(&address);

    let km_number = KM_NUMBER_PATTERN
        .captures(&address)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().replace(',', ".")));

    let address = PARENTHESES.replace_all(&address, " ");
    let address = DASH_ALIAS.replace_all(&address, " ");
    let address = QUOTES.replace_all(&address, " ");
    let address = KM.replace_all(&address, " ");
    let address = DANGLING_NUMBER.replace(&address, "");
    let address = FLOOR_AFTER_NUMBER.replace(&address, "");
    let address = drop_building(address.trim());

    let address = UNIT.replace_all(&address, "");
    let address = INTERSECTION.replace_all(&address, " ${1}");

    let address = if unaccent_and_lower(&address).starts_with("entre rios") {
        address.into_owned()
    } else {
        BETWEEN.replace_all(&address, " ${1}").into_owned()
    };

    let address = IMPLICIT_BETWEEN.replace_all(&address, "${1}");
    let address = STREET_TYPE_PATTERN.replace_all(&address, "");
    let address = address.split_whitespace().join(" ");

    if !address.is_empty() && address.bytes().all(|b| b.is_ascii_digit()) {
        return (address.clone(), String::new(), Some(address));
    }

    let clean_address = NUMBER_ANCHORED
        .replace_all(&address, "$1")
        .trim()
        .to_string();

    let (street, mut number) = split_street_number(&address);

    if km_number.is_some() && number.is_none() {
        number = km_number;
    }

    let street = NUMERIC_CROSS
        .captures(&street)
        .map_or_else(|| street.clone(), |capture| capture[1].to_owned());

    (clean_address, street, number)
}

fn ends_with_number(text: &str) -> bool {
    text.split_whitespace()
        .last()
        .is_some_and(|token| token.bytes().all(|b| b.is_ascii_digit()))
}

fn drop_building(text: &str) -> String {
    let Some(found) = BUILDING.find(text) else {
        return text.to_owned();
    };

    let head = text[..found.start()]
        .trim_end_matches(|c: char| c.is_whitespace() || matches!(c, ',' | '-' | ';' | '.'));

    if !head.chars().any(char::is_alphabetic) {
        return text.to_owned();
    }

    if head.split_whitespace().count() > 1 && ends_with_number(head) {
        return head.to_owned();
    }

    NUMBER_ANCHORED
        .captures(&text[found.start()..])
        .and_then(|capture| capture.get(1))
        .map(|matched| format!("{head} {}", matched.as_str()))
        .or_else(|| {
            TAIL_NUMBER
                .find_iter(&text[found.start()..])
                .last()
                .map(|number| format!("{head} {}", number.as_str()))
        })
        .unwrap_or_else(|| head.to_owned())
}

fn split_street_number(address: &str) -> (String, Option<String>) {
    let address = address.trim();

    if let Some(matched) = NO_NUMBER.find(address) {
        let start = matched.start();

        let street = if start == 0 {
            address[matched.end()..].trim_start()
        } else {
            address[..start].trim_end()
        };

        return (street.to_owned(), None);
    }

    let found = NUMBER_ANCHORED.captures_iter(address).last().or_else(|| {
        NUMBER_TRAILING
            .captures(address)
            .filter(|capture| !address[..capture.get(0).unwrap().start()].trim().is_empty())
    });

    let Some(capture) = found else {
        return (address.to_owned(), None);
    };

    let full_match = capture.get(0).unwrap();
    let start = full_match.start();

    let street = if start == 0 {
        address[full_match.end()..].trim_start()
    } else {
        address[..start].trim_end()
    };

    let digits = capture.get(1).unwrap().as_str();
    let number = (!digits.bytes().all(|b| b == b'0')).then_some(digits.to_owned());

    (street.to_owned(), number)
}
