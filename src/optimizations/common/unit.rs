use regex::Regex;
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

pub fn find_and_convert_to_px(attributes: &[OwnedAttribute], name: &str) -> Option<f64> {
    let match_val_and_unit: Regex = Regex::new(r"(.*?)([^\d\.]*)$").unwrap();

    attributes
        .iter()
        .find(|attribute| attribute.name.local_name == name)
        .and_then(|attr| match_val_and_unit.captures(&attr.value))
        .map(|capture| capture.extract())
        .and_then(|(_, [val, unit])| {
            let base_val = val.parse::<f64>().ok()?;
            let mult = unit_to_multiplier(unit)?;
            Some(base_val * mult)
        })
}
