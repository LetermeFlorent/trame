use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{OwnedValue, Value};

use super::uri::path_from_uri;
use super::{PickKind, PickRequest};

const DESTINATION: &str = "org.freedesktop.portal.Desktop";
const DESKTOP_PATH: &str = "/org/freedesktop/portal/desktop";
const FILE_CHOOSER: &str = "org.freedesktop.portal.FileChooser";
const REQUEST: &str = "org.freedesktop.portal.Request";
const RESPONSE_OK: u32 = 0;

pub enum Answer {
    Picked(PathBuf),
    Cancelled,
}

pub fn choose(request: &PickRequest, kind: PickKind) -> zbus::Result<Answer> {
    let connection = Connection::session()?;
    let token = handle_token();
    let request_path = request_path(&connection, &token)?;
    let response = Proxy::new(&connection, DESTINATION, request_path, REQUEST)?;
    let mut signals = response.receive_signal("Response")?;
    let chooser = Proxy::new(&connection, DESTINATION, DESKTOP_PATH, FILE_CHOOSER)?;
    let options = options(request, kind, &token);
    chooser.call_method("OpenFile", &("", request.title.as_str(), options))?;
    let message = signals
        .next()
        .ok_or_else(|| zbus::Error::Failure("the portal sent no answer".into()))?;
    let (code, results) = message
        .body()
        .deserialize::<(u32, HashMap<String, OwnedValue>)>()?;
    if code != RESPONSE_OK {
        return Ok(Answer::Cancelled);
    }
    Ok(first_path(&results).map_or(Answer::Cancelled, Answer::Picked))
}

fn handle_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.subsec_nanos());
    format!("trame{}_{nanos}", process::id())
}

fn request_path(connection: &Connection, token: &str) -> zbus::Result<String> {
    let name = connection
        .unique_name()
        .ok_or_else(|| zbus::Error::Failure("no name on the session bus".into()))?;
    let sender = name.trim_start_matches(':').replace('.', "_");
    Ok(format!("{DESKTOP_PATH}/request/{sender}/{token}"))
}

fn options<'a>(
    request: &'a PickRequest,
    kind: PickKind,
    token: &'a str,
) -> HashMap<&'a str, Value<'a>> {
    let mut options = HashMap::new();
    options.insert("handle_token", Value::from(token));
    options.insert("modal", Value::from(true));
    match kind {
        PickKind::Folder => {
            options.insert("directory", Value::from(true));
        }
        PickKind::Image => {
            let png = vec![("PNG", vec![(1_u32, "image/png"), (0_u32, "*.png")])];
            options.insert("filters", Value::new(png));
        }
    }
    if let Some(start) = &request.start {
        let mut bytes = start.as_os_str().as_bytes().to_vec();
        bytes.push(0);
        options.insert("current_folder", Value::new(bytes));
    }
    options
}

fn first_path(results: &HashMap<String, OwnedValue>) -> Option<PathBuf> {
    let Value::Array(uris) = &**results.get("uris")? else {
        return None;
    };
    uris.iter().find_map(|uri| match uri {
        Value::Str(text) => path_from_uri(text.as_str()),
        _ => None,
    })
}
