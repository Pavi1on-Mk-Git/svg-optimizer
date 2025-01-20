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
    let rounded = regex_replace!(r"(\.\d*?)0*$", rounded.as_str(), "$1");
    let rounded = regex_replace!(r"\.$", &rounded, "");
    let mut rounded = regex_replace!(r"(^|\D)0\.", &rounded, "$1.").to_owned();

    if rounded == "-0" {
        rounded = "0".into();
    }
    rounded.into_owned()
}

#[cfg(test)]
mod tests {
    use super::round_float;

    #[test]
    fn test_rounding() {
        assert_eq!(round_float(10000.1234567, 0), "10000");
        assert_eq!(round_float(10000.1234567, 4), "10000.1235");
        assert_eq!(round_float(1.0729712433664405, 3), "1.073");
        assert_eq!(round_float(-0.00001, 3), "0");
        assert_eq!(round_float(-0.0729712433664405, 2), "-.07");
    }
}
