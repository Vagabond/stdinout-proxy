use super::*;
use axum::{extract::Query, http::StatusCode, response, routing::post, Router};
use axum_auth::AuthBearer;
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;

pub type HandlerResult = std::result::Result<response::Json<Value>, (StatusCode, String)>;

#[derive(Debug, Clone, clap::Args)]
pub struct Server {}

impl Server {
    pub async fn run(self) -> Result {
        // build our application with a route
        let app = Router::new().route("/v1/stdin", post(stdin_handler));

        let server_endpoint = std::env::var("PORT").unwrap_or("3000".to_string());
        println!("Binding to port {}...", server_endpoint);
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], server_endpoint.parse().unwrap()));
        tokio::select!(
            result = axum::Server::bind(&addr)
                .serve(app.into_make_service()) =>
                    result.map_err(|e| Error::Axum(e.into())),
        )
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Params {
    sdf: String,
    lat: Decimal,
    lon: Decimal,
    txh: Decimal,
    f: Decimal,
    erp: Decimal,
    rxh: Decimal,
    rt: Decimal,
    dbm: bool,
    m: bool,
    o: String,
    #[serde(rename = "R")]
    r: usize,
    res: Decimal,
    pm: Decimal,
    rla: Decimal,
    rlo: Decimal,
}

#[derive(Debug, serde::Deserialize)]
pub struct Response {
    pub path_loss: Decimal,
    pub received_power: Decimal,
    pub field_strength: Decimal,
}

impl From<Error> for HandlerResult {
    fn from(e: Error) -> Self {
        Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }
}

pub async fn stdin_handler(
    query: Query<Params>,
    //AuthBearer(token): AuthBearer
) -> HandlerResult {
    // todo: check token
    
    use tokio::process::Command;
    let query = query.0.to_stdout_string();

    let url = std::env::var("PROXY_EXEC")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut child = Command::new(url)
        .arg("-daemon")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(query.as_bytes()).await.unwrap();
    stdin.flush().await.unwrap();
    // We drop the handle here which signals EOF to the child process.
    // This tells the child process that it there is no more data on the pipe.
    drop(stdin);

    let output = child.wait_with_output().await.unwrap();
    let mut response = String::from_utf8(output.stdout).unwrap();
    trim_newline(&mut response);
    let decimal = response
        .split(" ")
        .map(|s| s.parse::<Decimal>().unwrap())
        .collect::<Vec<Decimal>>();
    // todo: check token
    Ok(response::Json(json!({
        "status": "success",
        "data": {
            "path_loss": decimal[0],
            "received_power": decimal[1],
            "field_strength": decimal[2],
        }
    })))
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

impl Params {
    fn to_stdout_string(&self) -> String {
        let mut output = format!("-sdf {}", self.sdf);
        output.push_str(&format!(" -lat {}", self.lat));
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
        output.push_str(&format!(" -o {}", self.o));
        output.push_str(&format!(" -R {}", self.r));
        output.push_str(&format!(" -res {}", self.res));
        output.push_str(&format!(" -pm {}", self.pm));
        output.push_str(&format!(" -rla {}", self.rla));
        output.push_str(&format!(" -rlo {}", self.rlo));
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
        let params = Params {
            sdf: "hgt".to_string(),
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
            r: Decimal::from_str("2").unwrap(),
            res: Decimal::from_str("1200").unwrap(),
            pm: Decimal::from_str("4").unwrap(),
            rla: Decimal::from_str("44.73436").unwrap(),
            rlo: Decimal::from_str("-68.81993").unwrap(),
        };
        let params_string = params.to_stdout_string();
        assert_eq!(params_string, "-sdf hgt -lat 44.73566 -lon -68.82446 -txh 4 -f 900 -erp 5 -rxh 2 -rt -90 -dbm -m -o test4 -R 2 -res 1200 -pm 4 -rla 44.73436 -rlo -68.81993")
    }
}
