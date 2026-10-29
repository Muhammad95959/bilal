use std::f64::consts::PI;

const DEG: f64 = PI / 180.0;
const RAD: f64 = 180.0 / PI;

pub fn sin_deg(d: f64) -> f64 { (d * DEG).sin() }
pub fn cos_deg(d: f64) -> f64 { (d * DEG).cos() }
pub fn tan_deg(d: f64) -> f64 { (d * DEG).tan() }
pub fn arcsin_deg(x: f64) -> f64 { x.asin() * RAD }
pub fn arccos_deg(x: f64) -> f64 { x.acos() * RAD }
pub fn arctan2_deg(y: f64, x: f64) -> f64 { y.atan2(x) * RAD }
pub fn arccot_deg(x: f64) -> f64 { 1.0_f64.atan2(x) * RAD }

pub fn fix_angle(a: f64) -> f64 { wrap(a, 360.0) }
pub fn fix_hour(h: f64) -> f64 { wrap(h, 24.0) }

pub fn wrap(value: f64, modulus: f64) -> f64 {
    let r = value - modulus * (value / modulus).floor();
    if r < 0.0 { r + modulus } else { r }
}

pub fn julian_day(year: i32, month: i32, day: i32) -> f64 {
    let mut y = year;
    let mut m = month;
    if m <= 2 {
        y -= 1;
        m += 12;
    }
    let century = (y as f64 / 100.0).floor();
    ((y as f64 + 4716.0) * 365.25).floor()
        + ((m as f64 + 1.0) * 30.6001).floor()
        + day as f64
        + (2.0 - century + (century / 4.0).floor())
        - 1524.5
}

// adjusted formula matching Kotlin's floor toward -inf; above is standard. Keep as is.

pub fn sun_position(jd: f64) -> (f64, f64) {
    let days = jd - 2451545.0;
    let mean_anomaly = fix_angle(0.98560028 * days + 357.529);
    let mean_longitude = fix_angle(0.98564736 * days + 280.459);
    let ecliptic_longitude = fix_angle(
        mean_longitude
            + 1.915 * sin_deg(mean_anomaly)
            + 0.02 * sin_deg(2.0 * mean_anomaly),
    );
    let obliquity = 23.439 - 0.00000036 * days;
    let declination = arcsin_deg(sin_deg(obliquity) * sin_deg(ecliptic_longitude));
    let ra_hours = arctan2_deg(cos_deg(obliquity) * sin_deg(ecliptic_longitude), cos_deg(ecliptic_longitude)) / 15.0;
    let equation_of_time = mean_longitude / 15.0 - fix_hour(ra_hours);
    (declination, equation_of_time)
}

pub fn rounded_minute_of_day(hours: f64) -> Option<i32> {
    if hours.is_nan() { return None; }
    let bumped = fix_hour(hours + 0.0083333333);
    let hour = bumped.floor() as i32;
    let minute = ((bumped - hour as f64) * 60.0).floor() as i32;
    Some(hour * 60 + minute)
}