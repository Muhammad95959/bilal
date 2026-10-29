use crate::ptp::math::{self, *};
use crate::ptp::types::{CalculationParameters, Coordinates, DateComponents, HighLatitudeRule};

pub struct SolarTimes {
    pub fajr: f64,
    pub sunrise: f64,
    pub dhuhr: f64,
    pub asr: f64,
    pub sunset: f64,
    pub maghrib: f64,
    pub isha: f64,
}

const REFRACTION_DIP: f64 = 0.833;
const ELEVATION_COEFFICIENT: f64 = 0.0347;
const SOLAR_NOON: f64 = 12.0;
const HALF_CIRCLE: f64 = 180.0;
const RIGHT_ANGLE: f64 = 90.0;
const DEGREES_PER_HOUR: f64 = 15.0;
const MINUTES_PER_HOUR: f64 = 60.0;
const TWILIGHT_ANGLE_DIVISOR: f64 = 60.0;
const DEFAULT_ISHA_ANGLE: f64 = 18.0;
const RAMADAN_ISHA_BUMP_HOURS: f64 = 0.5;
const MAKKAH_KEY: &str = "makkah";
const SAUDI_ARABIA_CODE: &str = "SA";
const SEVENTH_PORTION: f64 = 0.14286;
const MIDDLE_PORTION: f64 = 0.5;

const FAJR_SEED: f64 = 5.0 / 24.0;
const SUNRISE_SEED: f64 = 6.0 / 24.0;
const DHUHR_SEED: f64 = 12.0 / 24.0;
const ASR_SEED: f64 = 13.0 / 24.0;
const SUNSET_SEED: f64 = 18.0 / 24.0;
const MAGHRIB_SEED: f64 = 18.0 / 24.0;
const ISHA_SEED: f64 = 18.0 / 24.0;

fn elevation_methods() -> &'static [&'static str] {
    &["iraq","morocco","tunisia","jordan","orleans","sudan","belgium","kazakhstan"]
}
fn elevation_countries() -> &'static [&'static str] {
    &["PS","IL","CZ","CH"]
}

pub struct PrayerTimeCalculator {
    latitude: f64,
    longitude: f64,
    altitude: f64,
    base_date: f64,
    parameters: CalculationParameters,
    country_code: String,
    time_zone_hours: f64,
}

impl PrayerTimeCalculator {
    pub fn new(coords: Coordinates, date: DateComponents, parameters: CalculationParameters, country_code: &str, time_zone_hours: f64) -> Self {
        let base_date = math::julian_day(date.year, date.month as i32, date.day as i32) - coords.longitude / 360.0;
        Self {
            latitude: coords.latitude,
            longitude: coords.longitude,
            altitude: coords.altitude,
            base_date,
            parameters,
            country_code: country_code.to_string(),
            time_zone_hours,
        }
    }

    pub fn compute(&self, rule: HighLatitudeRule) -> SolarTimes {
        let dip = if self.uses_elevation() {
            REFRACTION_DIP + ELEVATION_COEFFICIENT * self.altitude.sqrt()
        } else {
            REFRACTION_DIP
        };
        let offsets = self.parameters.offsets();

        let raw_fajr = self.sun_angle_time(HALF_CIRCLE - self.parameters.fajr_angle, FAJR_SEED);
        let raw_sunrise = self.sun_angle_time(HALF_CIRCLE - dip, SUNRISE_SEED);
        let raw_dhuhr = self.mid_day(DHUHR_SEED);
        let raw_asr = self.asr_time(self.parameters.madhab.shadow_factor() as f64, ASR_SEED);
        let raw_sunset = self.sun_angle_time(dip, SUNSET_SEED);
        let raw_maghrib = self.sun_angle_time(self.parameters.maghrib_value, MAGHRIB_SEED);
        let raw_isha = self.sun_angle_time(self.parameters.isha_value, ISHA_SEED);

        let localisation = self.time_zone_hours - self.longitude / DEGREES_PER_HOUR;
        let mut fajr = raw_fajr + localisation;
        let mut sunrise = raw_sunrise + localisation;
        let mut dhuhr = raw_dhuhr + localisation;
        let mut asr = raw_asr + localisation;
        let sunset = raw_sunset + localisation;
        let angle_maghrib = raw_maghrib + localisation + offsets.maghrib as f64 / MINUTES_PER_HOUR;
        let angle_isha = raw_isha + localisation + offsets.isha as f64 / MINUTES_PER_HOUR;

        fajr += offsets.fajr as f64 / MINUTES_PER_HOUR;
        sunrise += offsets.sunrise as f64 / MINUTES_PER_HOUR;
        dhuhr += offsets.dhuhr as f64 / MINUTES_PER_HOUR;
        asr += offsets.asr as f64 / MINUTES_PER_HOUR;

        let maghrib = self.resolve_maghrib(sunset, angle_maghrib, angle_isha, &offsets);

        let mut isha = if self.parameters.isha_is_interval {
            maghrib + self.parameters.isha_value / MINUTES_PER_HOUR + offsets.isha as f64 / MINUTES_PER_HOUR
        } else {
            angle_isha
        };

        if self.parameters.method == Some(MAKKAH_KEY)
            && self.country_code.eq_ignore_ascii_case(SAUDI_ARABIA_CODE)
            && self.parameters.is_ramadan
        {
            isha += RAMADAN_ISHA_BUMP_HOURS;
        }

        if rule != HighLatitudeRule::None {
            let night = fix_hour(sunrise - sunset);
            let fajr_portion = self.night_portion(rule, self.parameters.fajr_angle) * night;
            if fajr.is_nan() || fix_hour(sunrise - fajr) > fajr_portion {
                fajr = sunrise - fajr_portion + offsets.fajr as f64 / MINUTES_PER_HOUR;
            }
            let isha_twilight_angle = if self.parameters.isha_is_interval { DEFAULT_ISHA_ANGLE } else { self.parameters.isha_value };
            let isha_portion = self.night_portion(rule, isha_twilight_angle) * night;
            if isha.is_nan() || fix_hour(isha - maghrib) > isha_portion {
                isha = maghrib + isha_portion + offsets.isha as f64 / MINUTES_PER_HOUR;
            }
        }

        SolarTimes { fajr, sunrise, dhuhr, asr, sunset, maghrib, isha }
    }

    fn mid_day(&self, t: f64) -> f64 {
        let eq = math::sun_position(self.base_date + t).1;
        fix_hour(SOLAR_NOON - eq)
    }

    fn sun_angle_time(&self, angle: f64, t: f64) -> f64 {
        let declination = math::sun_position(self.base_date + t).0;
        let noon = self.mid_day(t);
        let numerator = -math::sin_deg(angle) - math::sin_deg(declination) * math::sin_deg(self.latitude);
        let denominator = math::cos_deg(declination) * math::cos_deg(self.latitude);
        let mut hour_angle = math::arccos_deg(numerator / denominator) / DEGREES_PER_HOUR;
        if angle > RIGHT_ANGLE {
            hour_angle = -hour_angle;
        }
        noon + hour_angle
    }

    fn asr_time(&self, factor: f64, t: f64) -> f64 {
        let declination = math::sun_position(self.base_date + t).0;
        let sun_altitude = math::arccot_deg(factor + math::tan_deg((self.latitude - declination).abs()));
        self.sun_angle_time(-sun_altitude, t)
    }

    fn uses_elevation(&self) -> bool {
        if elevation_methods().contains(&self.parameters.method.unwrap_or("")) {
            return true;
        }
        elevation_countries().contains(&self.country_code.to_uppercase().as_str())
    }

    fn resolve_maghrib(&self, sunset: f64, angle_maghrib: f64, angle_isha: f64, offsets: &crate::ptp::types::PrayerAdjustments) -> f64 {
        let sunset_based = sunset + offsets.maghrib as f64 / MINUTES_PER_HOUR;
        if self.parameters.maghrib_is_interval {
            return sunset_based + self.parameters.maghrib_value / MINUTES_PER_HOUR;
        }
        let is_chronological = angle_maghrib.is_finite()
            && angle_maghrib > sunset
            && (self.parameters.isha_is_interval || !angle_isha.is_finite() || angle_maghrib < angle_isha);
        if self.parameters.maghrib_value > 0.0 && is_chronological {
            angle_maghrib
        } else {
            sunset_based
        }
    }

    fn night_portion(&self, rule: HighLatitudeRule, angle: f64) -> f64 {
        match rule {
            HighLatitudeRule::Automatic => SEVENTH_PORTION,
            HighLatitudeRule::None => 0.0,
            HighLatitudeRule::MiddleOfTheNight => MIDDLE_PORTION,
            HighLatitudeRule::SeventhOfTheNight => SEVENTH_PORTION,
            HighLatitudeRule::TwilightAngle => angle / TWILIGHT_ANGLE_DIVISOR,
        }
    }
}

pub fn compute_prayer_times(
    coords: Coordinates,
    date: DateComponents,
    parameters: CalculationParameters,
    utc_offset_hours: f64,
    country_code: &str,
) -> (f64, f64, f64, f64, f64, f64, f64) {
    // returns (fajr, sunrise, dhuhr, asr, sunset, maghrib, isha) as minutes-of-day? No, fractional hours.
    let calc = PrayerTimeCalculator::new(coords, date, parameters, country_code, utc_offset_hours);
    let rule = parameters.high_latitude_rule;
    let first_pass = if rule == HighLatitudeRule::Automatic {
        calc.compute(HighLatitudeRule::None)
    } else {
        calc.compute(rule)
    };
    let resolved = if rule == HighLatitudeRule::Automatic {
        let fajr_min = rounded_minute_of_day(first_pass.fajr);
        let isha_min = rounded_minute_of_day(first_pass.isha);
        let fajr_broken = fajr_min.is_none() || fajr_min == Some(0);
        let isha_hour = isha_min.map(|m| m / 60);
        let isha_broken = isha_hour.is_none() || isha_hour == Some(0) || isha_hour == Some(12);
        if fajr_broken || isha_broken {
            calc.compute(HighLatitudeRule::SeventhOfTheNight)
        } else {
            first_pass
        }
    } else {
        first_pass
    };
    (resolved.fajr, resolved.sunrise, resolved.dhuhr, resolved.asr, resolved.sunset, resolved.maghrib, resolved.isha)
}

// helper re-export
pub use crate::ptp::math::rounded_minute_of_day;