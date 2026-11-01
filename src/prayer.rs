use std::sync::Arc;

use jiff::{Span, Zoned};
use jiff::civil::{Date, DateTime};
use crate::ptp::math::rounded_minute_of_day;

use crate::{config::Config, error::Error, ptp};

/// Resolve a user-facing method string to a prayer-time-plus key.
pub fn method_key(method: &str) -> Result<&'static str, Error> {
    let n = normalize(method);
    let key = match n.as_str() {
        // legacy misykat names
        "karachi" => "karachi",
        "muslimworldleague" | "mwl" => "mwl",
        "egyptian" | "egypt" => "egypt",
        "ummalqura" | "makkah" => "makkah",
        "northamerica" | "isna" => "isna",
        "french" => "uoif",
        "singapore" => "malaysia",
        "russia" => "moscow",
        "fixedinterval" | "fixed" => "fixedinterval",
        // prayer-time-plus keys / normalized enum names
        _ => {
            match ptp::method_keys().into_iter().find(|k| k.to_lowercase() == n) {
                Some(k) => k,
                None => return Err(Error::InvalidMethod(method.to_string())),
            }
        }
    };
    Ok(key)
}

fn normalize(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase()
}

/// Get a madhab
pub fn madhab(m: &str) -> Result<ptp::Madhab, Error> {
    match normalize(m).as_str() {
        "shafi" | "shafii" => Ok(ptp::Madhab::Shafi),
        "hanafi" => Ok(ptp::Madhab::Hanafi),
        _ => Err(Error::InvalidMadhab(m.to_string())),
    }
}

fn params_for(key: &'static str, m: ptp::Madhab) -> ptp::CalculationParameters {
    if key == "fixedinterval" {
        ptp::CalculationParameters {
            method: Some("custom"),
            fajr_angle: 19.5,
            maghrib_is_interval: false,
            maghrib_value: 0.0,
            isha_is_interval: true,
            isha_value: 90.0,
            method_adjustments: ptp::PrayerAdjustments::default(),
            adjustments: ptp::PrayerAdjustments::default(),
            madhab: m,
            high_latitude_rule: ptp::HighLatitudeRule::MiddleOfTheNight,
            is_ramadan: false,
        }
    } else {
        ptp::params_for(key, m)
    }
}

/// Returns all prayers
pub fn all(config: Arc<Config>) -> Result<ptp::PrayerTimes, Error> {
    let latitude = config.latitude as f64;
    let longitude = config.longitude as f64;
    let key = method_key(&config.method)?;
    let m = madhab(&config.madhab)?;
    let params = params_for(key, m);

    let country_code = config.country_code.clone().unwrap_or_default();

    let now_zoned = Zoned::now();
    let utc_offset_hours = now_zoned.offset().seconds() as f64 / 3600.0;
    let today_date = now_zoned.datetime().date();
    let dc = ptp::DateComponents::from_date(today_date);

    let coords = ptp::Coordinates { latitude, longitude, altitude: 0.0 };

    let (fajr, sunrise, dhuhr, asr, _sunset, maghrib, isha) =
        ptp::compute_prayer_times(coords, dc, params, utc_offset_hours, &country_code);

    let to_dt = |frac: f64, date: Date| -> DateTime {
        let minute = rounded_minute_of_day(frac).unwrap_or(0);
        let h = (minute / 60) as i8;
        let mi = (minute % 60) as i8;
        date.at(h, mi, 0, 0)
    };

    let mut fajr_dt = to_dt(fajr, today_date);
    let mut sherook_dt = to_dt(sunrise, today_date);
    let mut dohr_dt = to_dt(dhuhr, today_date);
    let mut asr_dt = to_dt(asr, today_date);
    let mut maghreb_dt = to_dt(maghrib, today_date);
    let mut ishaa_dt = to_dt(isha, today_date);

    // city-specific minute tweaks
    let deltas = config
        .city
        .as_ref()
        .and_then(|c| ptp::city_tweaks(key, c));
    if let Some(d) = deltas {
        let shift = |dt: DateTime, mins: i32| -> DateTime {
            if mins == 0 { dt } else { dt + Span::new().minutes(mins as i64) }
        };
        fajr_dt = shift(fajr_dt, d[0]);
        sherook_dt = shift(sherook_dt, d[1]);
        dohr_dt = shift(dohr_dt, d[2]);
        asr_dt = shift(asr_dt, d[3]);
        maghreb_dt = shift(maghreb_dt, d[5]);
        ishaa_dt = shift(ishaa_dt, d[6]);
    }

    // tomorrow's fajr
    let tomorrow_date = today_date.tomorrow().unwrap();
    let dc_tom = ptp::DateComponents::from_date(tomorrow_date);
    let (fajr_tom, ..) = ptp::compute_prayer_times(coords, dc_tom, params, utc_offset_hours, &country_code);
    let mut fajr_tomorrow_dt = to_dt(fajr_tom, tomorrow_date);
    if let Some(d) = deltas && d[0] != 0 {
        fajr_tomorrow_dt += Span::new().minutes(d[0] as i64);
    }

    // sunnah times
    let mag_min = maghreb_dt.hour() as i64 * 60 + maghreb_dt.minute() as i64;
    let fajr_tom_min = (fajr_tomorrow_dt.hour() as i64 * 60 + fajr_tomorrow_dt.minute() as i64) + 1440;
    let night = fajr_tom_min - mag_min;
    let first_min = mag_min + (night as f64 / 3.0).round() as i64;
    let mid_min = mag_min + (night as f64 / 2.0).round() as i64;
    let last_min = mag_min + (2.0 * night as f64 / 3.0).round() as i64;

    let add_minutes = |base: DateTime, mins: i64| -> DateTime {
        base + Span::new().minutes(mins)
    };
    let first_third = add_minutes(maghreb_dt, first_min - mag_min);
    let midnight = add_minutes(maghreb_dt, mid_min - mag_min);
    let last_third = add_minutes(maghreb_dt, last_min - mag_min);

    Ok(ptp::PrayerTimes {
        fajr: fajr_dt,
        sherook: sherook_dt,
        dohr: dohr_dt,
        asr: asr_dt,
        maghreb: maghreb_dt,
        ishaa: ishaa_dt,
        first_third_of_night: first_third,
        midnight,
        last_third_of_night: last_third,
        fajr_tomorrow: fajr_tomorrow_dt,
    })
}

