use super::{Error, Result};
use rust_decimal::Decimal;
use std::{collections::HashMap, path::Path};

pub struct DaemonHandle;

#[derive(Debug, serde::Serialize)]
pub struct PathResponse {
    pub path_loss: f64,
    pub received_power: f64,
    pub field_strength: f64,
    pub distance: Vec<f64>,
    pub reference: Vec<f64>,
    pub fresnel: Vec<f64>,
    pub fresnel60: Vec<f64>,
    pub curvature: Vec<f64>,
    pub profile: Vec<f64>,
}

#[derive(Debug, serde::Serialize)]
pub struct H3PlotResponse {
    pub hexes: HashMap<String, f64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PathParams {
    lat: Decimal,
    lon: Decimal,
    txh: Decimal,
    f: Decimal,
    erp: Decimal,
    rxh: Decimal,
    rt: Decimal,
    dbm: bool,
    m: bool,
    pm: Decimal,
    rla: Decimal,
    rlo: Decimal,
    pe: Option<u8>,
    gc: Option<u16>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PlotParams {
    lat: Decimal,
    lon: Decimal,
    txh: Decimal,
    f: Decimal,
    erp: Decimal,
    rxh: Decimal,
    rt: Decimal,
    dbm: bool,
    t: Option<bool>,
    m: bool,
    #[serde(rename = "R")]
    r: usize,
    pm: Decimal,
    pe: Option<u8>,
    gc: Option<u16>,
}

#[derive(Debug, serde::Deserialize)]
pub struct H3PlotParams {
    lat: f64,
    lon: f64,
    txh: Decimal,
    f: Decimal,
    erp: Decimal,
    rxh: Decimal,
    rt: f64,
    dbm: bool,
    pm: Decimal,
    // should be 0-15 or whatever h3 allows
    res: u8,
    pe: Option<u8>,
    gc: Option<u16>,
}

impl DaemonHandle {
    pub fn new() -> Result<DaemonHandle> {
        let (sdf_path, debug) = match std::env::var("SS_SDF") {
            Ok(sdf_path) => (sdf_path, false),
            _ => ("/tmp/doesnotexist".to_string(), true),
        };
        // Unwrap is fine, since this is only called once.
        signal_server::init(Path::new(&sdf_path), debug).unwrap();
        Ok(DaemonHandle)
    }

    pub fn path(&self, params: PathParams) -> Result<PathResponse> {
        let params = params.to_stdout_string();
        let report = signal_server::call_sigserve(&params).unwrap();
        Ok(PathResponse {
            path_loss: report.loss,
            received_power: report.dbm,
            field_strength: report.field_strength,
            distance: report.distancevec,
            reference: report.referencevec,
            fresnel: report.fresnelvec,
            fresnel60: report.fresnel60vec,
            curvature: report.curvaturevec,
            profile: report.profilevec,
        })
    }

    pub fn h3plot(&self, params: H3PlotParams) -> Result<H3PlotResponse> {
        let res = match params.res {
            1 => h3o::Resolution::One,
            2 => h3o::Resolution::Two,
            3 => h3o::Resolution::Three,
            4 => h3o::Resolution::Four,
            5 => h3o::Resolution::Five,
            6 => h3o::Resolution::Six,
            7 => h3o::Resolution::Seven,
            8 => h3o::Resolution::Eight,
            9 => h3o::Resolution::Nine,
            10 => h3o::Resolution::Ten,
            // below 10 is smaller than 3 arc seconds
            //11 => h3o::Resolution::Eleven,
            //12 => h3o::Resolution::Twelve,
            _ => return Err(Error::Axum("bad resolution".into())),
        };
        let ll = h3o::LatLng::new(params.lat, params.lon).unwrap();
        let cell = ll.to_cell(res);
        let mut i = 0;
        let mut hexes = HashMap::new();
        loop {
            let cells = cell
                .grid_ring_fast(i)
                .collect::<Option<Vec<_>>>()
                .unwrap_or_default();
            let mut found = false;
            for cell in cells {
                let latlng = h3o::LatLng::from(cell);
                let paramstr = params.to_stdout_string(latlng.lat(), latlng.lng());
                let report = signal_server::call_sigserve(&paramstr).unwrap();
                if report.dbm > params.rt {
                    hexes.insert(format!("{}", cell), report.dbm);
                    found = true;
                }
            }
            if !found {
                break Ok(H3PlotResponse { hexes });
            }
            i += 1;
        }
    }

    pub fn plot(&self, params: PlotParams) -> Result<Vec<u8>> {
        let params = params.to_stdout_string();
        let report = signal_server::call_sigserve(&params).unwrap();
        Ok(report.image_data)
    }
}

impl PathParams {
    fn to_stdout_string(&self) -> String {
        let mut output = format!(" -lat {}", self.lat);
        output.push_str(&format!(" -lon {}", self.lon));
        output.push_str(&format!(" -txh {}", self.txh));
        output.push_str(&format!(" -f {}", self.f));
        output.push_str(&format!(" -erp {}", self.erp));
        output.push_str(&format!(" -rxh {}", self.rxh));
        output.push_str(&format!(" -rt {}", self.rt));
        if self.dbm {
            output.push_str(" -dbm");
        }
        if self.m {
            output.push_str(" -m");
        }
        output.push_str(&format!(" -pm {}", self.pm));
        output.push_str(&format!(" -rla {}", self.rla));
        output.push_str(&format!(" -rlo {}", self.rlo));
        if self.pe.is_some() {
            output.push_str(&format!(" -pe {}", self.pe.unwrap()));
        }
        if self.gc.is_some() {
            output.push_str(&format!(" -gc {}", self.gc.unwrap()));
        }
        output.push_str("\r\n");
        output
    }
}

impl PlotParams {
    fn to_stdout_string(&self) -> String {
        let mut output = format!(" -lat {}", self.lat);
        output.push_str(&format!(" -lon {}", self.lon));
        output.push_str(&format!(" -txh {}", self.txh));
        output.push_str(&format!(" -f {}", self.f));
        output.push_str(&format!(" -erp {}", self.erp));
        output.push_str(&format!(" -rxh {}", self.rxh));
        output.push_str(&format!(" -rt {}", self.rt));
        if self.dbm {
            output.push_str(" -dbm");
        }
        if self.m {
            output.push_str(" -m");
        }
        output.push_str(&format!(" -pm {}", self.pm));
        output.push_str(&format!(" -R {}", self.r));
        if self.t.is_some() {
            output.push_str(" -t");
        }
        if self.pe.is_some() {
            output.push_str(&format!(" -pe {}", self.pe.unwrap()));
        }
        if self.gc.is_some() {
            output.push_str(&format!(" -gc {}", self.gc.unwrap()));
        }
        output.push_str("\r\n");
        output
    }
}

impl H3PlotParams {
    fn to_stdout_string(&self, lat: f64, lng: f64) -> String {
        let mut output = format!(" -lat {}", self.lat);
        output.push_str(&format!(" -lon {}", self.lon));
        output.push_str(&format!(" -rla {}", lat));
        output.push_str(&format!(" -rlo {}", lng));
        output.push_str(&format!(" -txh {}", self.txh));
        output.push_str(&format!(" -f {}", self.f));
        output.push_str(&format!(" -erp {}", self.erp));
        output.push_str(&format!(" -rxh {}", self.rxh));
        output.push_str(&format!(" -rt {}", self.rt));
        if self.dbm {
            output.push_str(" -dbm");
        }
        output.push_str(&format!(" -pm {}", self.pm));
        if self.pe.is_some() {
            output.push_str(&format!(" -pe {}", self.pe.unwrap()));
        }
        if self.gc.is_some() {
            output.push_str(&format!(" -gc {}", self.gc.unwrap()));
        }
        output.push_str("\r\n");
        output
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_params_to_string() {
        let params = PathParams {
            lat: Decimal::from_str("44.73566").unwrap(),
            lon: Decimal::from_str("-68.82446").unwrap(),
            txh: Decimal::from_str("4").unwrap(),
            f: Decimal::from_str("900").unwrap(),
            erp: Decimal::from_str("5").unwrap(),
            rxh: Decimal::from_str("2").unwrap(),
            rt: Decimal::from_str("-90").unwrap(),
            dbm: true,
            m: true,
            o: "test4".to_string(),
            r: 2,
            pm: Decimal::from_str("4").unwrap(),
            rla: Decimal::from_str("44.73436").unwrap(),
            rlo: Decimal::from_str("-68.81993").unwrap(),
        };
        let params_string = params.to_stdout_string();
        assert_eq!(params_string, "-sdf hgt -lat 44.73566 -lon -68.82446 -txh 4 -f 900 -erp 5 -rxh 2 -rt -90 -dbm -m -o test4 -res 1200 -pm 4 -rla 44.73436 -rlo -68.81993\n")
    }
}
