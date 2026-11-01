use jiff::civil::{DateTime, Date};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Madhab {
    Shafi,
    Hanafi,
}

impl Madhab {
    pub fn shadow_factor(self) -> i32 {
        match self {
            Madhab::Shafi => 1,
            Madhab::Hanafi => 2,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HighLatitudeRule {
    None,
    MiddleOfTheNight,
    SeventhOfTheNight,
    TwilightAngle,
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct PrayerAdjustments {
    pub fajr: i32,
    pub sunrise: i32,
    pub dhuhr: i32,
    pub asr: i32,
    pub maghrib: i32,
    pub isha: i32,
}

impl std::ops::Add for PrayerAdjustments {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            fajr: self.fajr + other.fajr,
            sunrise: self.sunrise + other.sunrise,
            dhuhr: self.dhuhr + other.dhuhr,
            asr: self.asr + other.asr,
            maghrib: self.maghrib + other.maghrib,
            isha: self.isha + other.isha,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CalculationParameters {
    pub method: Option<&'static str>,
    pub fajr_angle: f64,
    pub maghrib_is_interval: bool,
    pub maghrib_value: f64,
    pub isha_is_interval: bool,
    pub isha_value: f64,
    pub method_adjustments: PrayerAdjustments,
    pub adjustments: PrayerAdjustments,
    pub madhab: Madhab,
    pub high_latitude_rule: HighLatitudeRule,
    pub is_ramadan: bool,
}

impl CalculationParameters {
    pub fn offsets(&self) -> PrayerAdjustments {
        self.method_adjustments + self.adjustments
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct DateComponents {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl DateComponents {
    pub fn from_date(d: Date) -> Self {
        Self { year: d.year() as i32, month: d.month() as u8, day: d.day() as u8 }
    }
    pub fn to_date(self) -> Date {
        Date::new(self.year as i16, self.month as i8, self.day as i8).unwrap()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Prayer {
    Fajr,
    Sherook,
    Dohr,
    Asr,
    Maghreb,
    Ishaa,
    FajrTomorrow,
}

#[derive(Debug, Copy, Clone)]
pub struct PrayerTimes {
    pub fajr: DateTime,
    pub sherook: DateTime,
    pub dohr: DateTime,
    pub asr: DateTime,
    pub maghreb: DateTime,
    pub ishaa: DateTime,
    pub first_third_of_night: DateTime,
    pub midnight: DateTime,
    pub last_third_of_night: DateTime,
    pub fajr_tomorrow: DateTime,
}

impl PrayerTimes {
    pub fn current(&self, now: DateTime) -> Prayer {
        let mut current: Option<Prayer> = None;
        let ranges = [
            (Prayer::Fajr, self.fajr, self.sherook),
            (Prayer::Sherook, self.sherook, self.dohr),
            (Prayer::Dohr, self.dohr, self.asr),
            (Prayer::Asr, self.asr, self.maghreb),
            (Prayer::Maghreb, self.maghreb, self.ishaa),
            (Prayer::Ishaa, self.ishaa, self.fajr_tomorrow),
        ];
        for (p, start, end) in ranges {
            if now >= start && now < end {
                current = Some(p);
            }
        }
        if current.is_none() && now < self.fajr {
            current = Some(Prayer::Ishaa);
        }
        current.unwrap_or(Prayer::Ishaa)
    }

    fn is_after_midnight(&self, now: DateTime) -> bool {
        now.time() <= self.fajr_tomorrow.time()
    }

    pub fn next(&self, now: DateTime) -> Prayer {
        match self.current(now) {
            Prayer::Fajr => Prayer::Sherook,
            Prayer::Sherook => Prayer::Dohr,
            Prayer::Dohr => Prayer::Asr,
            Prayer::Asr => Prayer::Maghreb,
            Prayer::Maghreb => Prayer::Ishaa,
            Prayer::Ishaa => {
                if self.is_after_midnight(now) {
                    Prayer::Fajr
                } else {
                    Prayer::FajrTomorrow
                }
            }
            Prayer::FajrTomorrow => Prayer::Fajr,
        }
    }

    pub fn time(&self, prayer: Prayer) -> DateTime {
        match prayer {
            Prayer::Fajr => self.fajr,
            Prayer::Sherook => self.sherook,
            Prayer::Dohr => self.dohr,
            Prayer::Asr => self.asr,
            Prayer::Maghreb => self.maghreb,
            Prayer::Ishaa => self.ishaa,
            Prayer::FajrTomorrow => self.fajr_tomorrow,
        }
    }

    pub fn time_remaining(&self, now: DateTime) -> (u32, u32) {
        let mut now2 = now;
        let next = self.next(now);
        if next == Prayer::FajrTomorrow && self.is_after_midnight(now) {
            now2 = now + jiff::Span::new().days(1);
        }
        let next_time = self.time(next);
        let dur = next_time - now2;
        let total_secs = dur.total(jiff::Unit::Second).unwrap_or(0.0);
        let whole = total_secs / 60.0 / 60.0;
        let fract = whole - whole.floor();
        let hours = whole.floor() as u32;
        let minutes = (fract * 60.0).round() as u32;
        (hours, minutes)
    }
}

impl Prayer {
    pub fn name(self) -> String {
        match self {
            Prayer::Fajr | Prayer::FajrTomorrow => "Fajr".to_string(),
            Prayer::Sherook => "Sherook".to_string(),
            Prayer::Dohr => "Dohr".to_string(),
            Prayer::Asr => "Asr".to_string(),
            Prayer::Maghreb => "Maghreb".to_string(),
            Prayer::Ishaa => "Ishaa".to_string(),
        }
    }
}
