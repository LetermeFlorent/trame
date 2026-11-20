use super::SshHost;

impl SshHost {
    pub fn kept_command(&self, session: &str) -> String {
        let base = self.command();
        if !is_valid_session(session) {
            return base;
        }
        let command = base.trim_end();
        format!("{command} -t 'tmux new-session -A -s {session} || exec $SHELL -l'\r")
    }
}

fn is_valid_session(session: &str) -> bool {
    !session.is_empty()
        && !session.starts_with('-')
        && session
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}
