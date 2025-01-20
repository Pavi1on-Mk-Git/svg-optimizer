use super::id_usage::find_attribute;
use lazy_regex::{regex, regex_replace};
use xml::attribute::OwnedAttribute;

fn unit_to_multiplier(unit: &str) -> Option<f64> {
    // Convert only units which would realistically shorten the value
    match unit {
        "px" | "" => Some(1.),
        "pt" => Some(1.25),
        "pc" => Some(15.),
        _ => None,
    }
}

pub fn convert_to_px(value: &str) -> Option<f64> {
    let match_val_and_unit = regex!(r"(.*?)([^\d\.]*)$");

    match_val_and_unit
        .captures(value)
        .map(|capture| capture.extract())
        .and_then(|(_, [val, unit])| {
            let base_val = val.parse::<f64>().ok()?;
            let mult = unit_to_multiplier(unit)?;
            Some(base_val * mult)
        })
}

pub fn find_and_convert_to_px(attributes: &[OwnedAttribute], name: &str) -> Option<f64> {
    find_attribute(attributes, name).and_then(|value| convert_to_px(value))
}

pub fn round_float(number: f64, precision: usize) -> String {
    let rounded = format!("{:.1$}", number, precision);
    let no_trailing_zeros = regex_replace!(r"\.?0*$", rounded.as_str(), "");
    let no_leading_zeros = regex_replace!(r"(^|\D)0+([\d\.])", &no_trailing_zeros, "$1$2");
    no_leading_zeros.into_owned()
}
