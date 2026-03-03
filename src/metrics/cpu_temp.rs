/// Parse the concatenated output of `cat /sys/class/thermal/thermal_zone*/temp`.
///
/// Each line is a temperature in millidegrees Celsius (e.g. `45000` = 45.0 °C).
/// Returns the average temperature across all valid zones in °C, or 0.0 if
/// no valid readings are found.
pub fn parse_thermal_zones(contents: &str) -> f32 {
    let mut sum: f64 = 0.0;
    let mut count: u32 = 0;

    for line in contents.lines() {
        if let Ok(millideg) = line.trim().parse::<u64>() {
            sum += millideg as f64 / 1000.0;
            count += 1;
        }
    }

    if count == 0 {
        0.0
    } else {
        (sum / count as f64) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_zone() {
        assert!((parse_thermal_zones("45000\n") - 45.0).abs() < 0.01);
    }

    #[test]
    fn multiple_zones_averaged() {
        let input = "40000\n50000\n60000\n";
        assert!((parse_thermal_zones(input) - 50.0).abs() < 0.01);
    }

    #[test]
    fn skips_invalid_lines() {
        let input = "45000\nnot-a-number\n55000\n";
        assert!((parse_thermal_zones(input) - 50.0).abs() < 0.01);
    }

    #[test]
    fn empty_returns_zero() {
        assert_eq!(parse_thermal_zones(""), 0.0);
    }

    #[test]
    fn all_garbage_returns_zero() {
        assert_eq!(parse_thermal_zones("foo\nbar\n"), 0.0);
    }
}
