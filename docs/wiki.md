# Wiki

## Configuration

You can get latitude and longitude value from online services, such as
[OpenStreetMap](https://www.openstreetmap.org), or Google Maps.

In OSM, click the area, then select "show address".

### Method

Bilal uses the prayer-time-plus engine, so all of its 50+ calculation methods are
available. Pass either the method key (lowercase, e.g. `egypt`, `mwl`) or a
case-insensitive name/alias (e.g. `Egyptian`, `MuslimWorldLeague`, `FixedInterval`).

Some common ones:

- `mwl` / `MuslimWorldLeague` — Muslim World League
- `egypt` / `Egyptian` — Egyptian General Authority of Survey
- `makkah` / `UmmAlQura` — Umm al-Qura, Makkah
- `isna` / `NorthAmerica` — Islamic Society of North America
- `karachi` / `Karachi` — University of Islamic Sciences, Karachi
- `turkey` / `Turkey` — Diyanet, Turkey
- `indonesia` / `Indonesia` — Kemenag, Indonesia
- `malaysia` / `Singapore` — JAKIM, Malaysia/Singapore
- `moscow` / `Russia` — Moscow
- `uoif` / `French` — UOIF, France
- `fixedinterval` / `FixedInterval` — Fixed Isha interval (90 min)

For the full list of keys see `src/ptp/data/methods.rs` (`method_keys()`).

### Madhab

This settings corresponds to Asr prayer time.
For Hanafi madhab, the Asr is bit later than that of the Shafi madhab.

- `Shafi`: Earlier Asr time
- `Hanafi`: Later Asr time
