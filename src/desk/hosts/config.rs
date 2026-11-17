use crate::host::persist;
use crate::log_error;
use crate::preferences::{SshHost, vault};

pub fn written_by_hand(host: &SshHost) -> bool {
    let Some(alias) = host.alias() else {
        return false;
    };
    let Some(config) = persist::ssh_config() else {
        return false;
    };
    config.written_by_hand(alias).unwrap_or_else(|error| {
        log_error!("the SSH config could not be read: {error}");
        false
    })
}

pub(super) fn remove_from_config(host: &SshHost) {
    if let (Some(alias), Some(config)) = (host.alias(), persist::ssh_config())
        && let Err(error) = config.remove(alias)
    {
        log_error!("{alias} could not be removed from the SSH config: {error}");
    }
}

pub(super) fn save_secret(host: &SshHost, secret: Option<&str>) {
    let Some(secret) = secret else {
        return;
    };
    if let Err(error) = vault::SSH.store(host, secret) {
        log_error!(
            "the password for {} could not be saved: {error}",
            host.label()
        );
    }
}

pub(super) fn kept_in_config(host: SshHost) -> SshHost {
    let Some(config) = persist::ssh_config() else {
        return host;
    };
    config.keep(&host).unwrap_or_else(|error| {
        log_error!(
            "{} could not be added to the SSH config: {error}",
            host.label()
        );
        host
    })
}

pub(super) fn changed_in_config(original: &SshHost, host: SshHost) -> SshHost {
    let (Some(alias), Some(config)) = (original.alias(), persist::ssh_config()) else {
        return kept_in_config(host);
    };
    config.replace(alias, &host).unwrap_or_else(|error| {
        log_error!(
            "{} could not be changed in the SSH config: {error}",
            host.label()
        );
        host
    })
}
