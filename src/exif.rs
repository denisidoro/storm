use crate::geo::db::LatLng;
use crate::shell;
use crate::smalldate::SmallDate;
use crate::{config, fs::IPathBuf};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, FixedOffset};
use std::path::{Path, PathBuf};

#[derive(PartialEq, Eq, Debug)]
enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug)]
pub struct CompressionInvariantProps {
    millis: Option<u32>,
    lat: Option<f32>,
    make: Option<String>,
    model: Option<String>,
    orientation: Orientation,
}

fn abs_diff(slf: u32, other: u32) -> u32 {
    if slf < other {
        other - slf
    } else {
        slf - other
    }
}

impl PartialEq for CompressionInvariantProps {
    fn eq(&self, other: &Self) -> bool {
        if self.make != other.make || self.model != other.model {
            return false;
        }

        let ignore_rotation = config::get().clap.ignore_rotation;
        if !ignore_rotation && self.orientation != other.orientation {
            return false;
        }

        let same_lat = match (self.lat, other.lat) {
            (None, None) => true,
            (Some(l1), Some(l2)) => (l1 - l2).abs() < 0.01,
            _ => false,
        };
        if !same_lat {
            return false;
        }

        match (self.millis, other.millis) {
            (None, None) => true,
            (Some(m1), Some(m2)) => abs_diff(m1, m2) < 2000,
            _ => false,
        }
    }
}

impl Eq for CompressionInvariantProps {}

pub struct Props {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub rotation: Option<i32>,
    pub compression_invariant: CompressionInvariantProps,
}

pub fn add_time(filepath: &Path, minutes: u32) -> Result<()> {
    let d = datetime(filepath)?;
    let d = d + Duration::minutes(minutes.into());

    let date_time_original = format!("-datetimeoriginal=\"{}\"", d);
    let all_dates = format!("-AllDates=\"{}\"", d);
    let filename = filepath.to_string();
    let args = [
        date_time_original.as_str(),
        all_dates.as_str(),
        "-overwrite_original",
        filename.as_str(),
    ];

    shell::out("exiftool", &args)?;

    Ok(())
}

pub fn datestr(filepath: &Path, format: &str) -> Result<String> {
    let exiftool_args = &[
        "-T",
        "-DateTimeOriginal",
        "-MediaCreateDate",
        "-d",
        format,
        &filepath.to_string(),
    ];

    let out = shell::out("exiftool", exiftool_args)?
        .res
        .context("no exiftool output")?
        .stdout;

    let mut parts = out.split('\t');

    let mut datetime = parts.next().unwrap_or_default();
    if datetime.len() < 3 {
        datetime = parts.next().unwrap();
    }

    Ok(datetime.into())
}

pub fn datetime(filepath: &Path) -> Result<DateTime<FixedOffset>> {
    let mut datestr = datestr(filepath, "%Y-%m-%dT%H:%M:%S%z")?;

    // Convert 2022-10-13T05:23:05-0300 to 2022-10-13T05:23:05-03:00
    datestr = format!(
        "{}:{}",
        &datestr[0..datestr.len() - 2],
        &datestr[datestr.len() - 2..datestr.len()]
    );

    let datetime = DateTime::parse_from_rfc3339(&datestr)?;
    Ok(datetime)
}

pub fn date(filepath: &Path) -> Result<SmallDate> {
    let datestr = datestr(filepath, "%y%m%d")?;
    let date = SmallDate::from_str(&datestr)?;
    Ok(date)
}

fn is_empty(value: &str) -> bool {
    matches!(value, "" | "-" | "Unknown" | "Undefined")
}

fn normalize_opt(value: &str) -> Option<String> {
    if is_empty(value) {
        None
    } else {
        Some(value.into())
    }
}

impl Props {
    fn args(&self) -> Vec<String> {
        let mut v = vec![];

        if let Some(x) = &self.compression_invariant.make {
            v.push(format!("-make={}", x));
        }

        if let Some(x) = &self.compression_invariant.model {
            v.push(format!("-model={}", x));
        }

        if self.rotation.is_some() {
            v.push("--orientation".into());
        }

        v
    }

    fn from_str(txt: &str, path: &Path) -> Result<Self> {
        let mut parts = txt.split('\t');

        let orientation_u32: Option<u32> = {
            let raw = parts.next().context("no orientation")?;
            if is_empty(raw) {
                None
            } else {
                Some(raw.parse().context("orientation is not a number")?)
            }
        };

        let width: u32 = parts
            .next()
            .context("no width")?
            .parse()
            .context("width is not a number")?;

        let height: u32 = parts
            .next()
            .context("no height")?
            .parse()
            .context("height is not a number")?;

        let millis = {
            let duration = parts.next().context("no duration")?;
            if is_empty(duration) {
                None
            } else {
                let secs_f64 = duration.parse::<f64>()?;
                let millis = (secs_f64 * 1000.0) as u32;
                Some(millis)
            }
        };

        let lat = {
            let raw = normalize_opt(parts.next().context("no lat")?);
            match raw {
                Some(lat_str) => {
                    let lat_f32: f32 = lat_str
                        .parse()
                        .with_context(|| format!("lat is not a number: {}", lat_str))?;
                    Some(lat_f32)
                }
                None => None,
            }
        };

        let make = normalize_opt({
            let make = parts.next().context("no make")?;
            let android_make = parts.next().context("no android make")?;
            if !is_empty(make) {
                make
            } else {
                android_make
            }
        });

        let model = normalize_opt({
            let model = parts.next().context("no model")?;
            let android_model = parts.next().context("no android model")?;
            if !is_empty(model) {
                model
            } else {
                android_model
            }
        });

        let rotation: Option<i32> = {
            let raw = normalize_opt(parts.next().context("no rotation")?);
            match raw {
                Some(r) => Some(r.parse().context("rotation is not a number")?),
                None => None,
            }
        };

        let orientation = {
            let is_wider_originally = width > height;

            let is_rotated = {
                match orientation_u32 {
                    Some(o) => (5..=8).contains(&o),
                    None => {
                        let degrees = rotation.unwrap_or(0);
                        (degrees / 90) % 2 == 1
                    }
                }
            };

            let is_wider = (is_wider_originally && !is_rotated) || (!is_wider_originally && is_rotated);
            if is_wider {
                Orientation::Landscape
            } else {
                Orientation::Portrait
            }
        };

        let compression_invariant = CompressionInvariantProps {
            millis,
            lat,
            make,
            model,
            orientation,
        };

        Ok(Self {
            path: path.into(),
            width,
            height,
            rotation,
            compression_invariant,
        })
    }
}

pub fn props(path: &Path) -> Result<Props> {
    let args = &[
        "-T",
        "-n",
        "-orientation",
        "-imageWidth",
        "-imageHeight",
        "-duration",
        "-gpsLatitude",
        "-make",
        "-androidManufacturer",
        "-model",
        "-androidModel",
        "-rotation",
        &path.to_string(),
    ];

    shell::out("exiftool", args)?
        .res
        .context("no exiftool output")
        .and_then(|r| Props::from_str(r.stdout.trim(), path))
}

pub fn copy_metadata(from_props: &Props, to: &Path) -> Result<()> {
    let from_str = &from_props.path.to_string();
    let to_str = &to.to_string();

    shell::out("touch", &["-r", from_str, to_str])?;

    let from_args = from_props.args();
    let args = {
        let mut a = vec![
            "-overwrite_original_in_place",
            "-TagsFromFile",
            from_str,
            "-all:all>all:all",
            "--imageWidth",
            "--imageHeight",
        ];
        a.append(&mut from_args.iter().map(String::as_str).collect());
        a.push(to_str);
        a
    };

    shell::out("exiftool", &args)?;

    Ok(())
}

pub fn has_latitude(filepath: &Path) -> Result<bool> {
    let exiftool_args = &["-T", "-n", "-gpsLatitude", &filepath.to_string()];

    let out = shell::out("exiftool", exiftool_args)?
        .res
        .context("no exiftool output")?
        .stdout;

    Ok(out.len() > 2)
}

pub fn add_geo(filepath: &Path, latlng: LatLng) -> Result<()> {
    let latitude = format!("-xmp:gpslatitude={}", latlng.0);
    let longitude = format!("-xmp:gpslongitude={}", latlng.1);
    let exiftool_args = &[
        latitude.as_str(),
        longitude.as_str(),
        "-overwrite_original",
        &filepath.to_string(),
    ];

    shell::out("exiftool", exiftool_args)?
        .res
        .context("no exiftool output")?;

    Ok(())
}
