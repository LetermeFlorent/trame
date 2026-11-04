use std::time::{Duration, Instant};

use trame::preferences::{
    ANSWER_WINDOW, HostEntry, Prompt, SecretAnswer, SshHost, SshLogin, prompt_in,
};

const REPLY: Option<&str> = Some("hunter2\r");
const PASSWORD: &str = "root@10.0.0.2's password: ";
const PASSPHRASE: &str = "Enter passphrase for key 'C:\\Users\\me/.ssh/id_ed25519': ";
const TYPED: &str = "PS C:\\Users\\me> ssh root@10.0.0.2";

fn answer_for(target: &str, entry: Option<&HostEntry>, now: Instant) -> SecretAnswer {
    let host = SshHost::parse(target).expect("a host");
    SecretAnswer::new("hunter2".to_owned(), SshLogin::of(&host, entry), now)
}

#[test]
fn ssh_prompts_are_recognised() {
    let cases = [
        (PASSWORD, Some(Prompt::Password)),
        ("(me@box) Password: ", Some(Prompt::Password)),
        (PASSPHRASE, Some(Prompt::Passphrase)),
        ("ENTER PASSPHRASE FOR KEY 'k':", Some(Prompt::Passphrase)),
    ];
    for (line, expected) in cases {
        assert_eq!(prompt_in(line), expected, "{line}");
    }
}

#[test]
fn other_prompts_are_left_to_the_user() {
    let cases = [
        "[sudo] password for me: ",
        "Password: ",
        "Enter password: ",
        "me@box:~$ echo password:",
        "Are you sure you want to continue connecting (yes/no/[fingerprint])? ",
        "Enter passphrase for key 'k': typed",
        "",
    ];
    for line in cases {
        assert_eq!(prompt_in(line), None, "{line}");
    }
}

#[test]
fn the_password_is_given_once() {
    let now = Instant::now();
    let mut answer = answer_for("root@10.0.0.2", None, now);
    assert_eq!(answer.reply(PASSWORD, now).as_deref(), REPLY);
    assert!(answer.is_over(now));
    assert_eq!(answer.reply(PASSWORD, now), None);
}

#[test]
fn the_secret_is_only_given_shortly_after_connecting() {
    let now = Instant::now();
    let mut answer = answer_for("root@10.0.0.2", None, now);
    let late = now + ANSWER_WINDOW;
    assert_eq!(answer.deadline(), late);
    assert!(!answer.is_over(late - Duration::from_millis(1)));
    assert!(answer.is_over(late));
    assert_eq!(answer.reply(PASSWORD, late), None);
}

#[test]
fn no_answer_for_another_host() {
    let now = Instant::now();
    let mut answer = answer_for("root@10.0.0.2", None, now);
    assert_eq!(answer.reply(TYPED, now), None);
    assert_eq!(answer.reply("me@bastion's password: ", now), None);
    assert_eq!(answer.reply("(root@10.0.0.3) Password: ", now), None);
    assert_eq!(answer.reply(PASSWORD, now).as_deref(), REPLY);
}

#[test]
fn no_answer_after_the_first_line_that_is_not_a_prompt() {
    let now = Instant::now();
    let mut answer = answer_for("root@10.0.0.2", None, now);
    assert_eq!(answer.reply(TYPED, now), None);
    assert_eq!(answer.reply("Welcome to Ubuntu 24.04 LTS", now), None);
    assert_eq!(answer.reply(PASSWORD, now), None);
    assert!(answer.is_over(now));
}

#[test]
fn no_password_for_a_passphrase_prompt() {
    let now = Instant::now();
    let mut answer = answer_for("root@10.0.0.2", None, now);
    assert_eq!(answer.reply(TYPED, now), None);
    assert_eq!(answer.reply(PASSPHRASE, now), None);
    assert_eq!(answer.reply(PASSWORD, now).as_deref(), REPLY);
}

#[test]
fn the_lines_of_a_first_connection_keep_the_answer() {
    let now = Instant::now();
    let mut answer = answer_for("root@10.0.0.2", None, now);
    let lines = [
        "Windows PowerShell",
        TYPED,
        "",
        "Are you sure you want to continue connecting (yes/no/[fingerprint])? yes",
    ];
    for line in lines {
        assert_eq!(answer.reply(line, now), None, "{line}");
    }
    assert_eq!(answer.reply(PASSWORD, now).as_deref(), REPLY);
}

#[test]
fn the_prompt_must_name_the_saved_user_and_host() {
    let entry = HostEntry {
        address: "w.example.com".to_owned(),
        user: Some("me".to_owned()),
        port: None,
    };
    let cases = [
        ("work", "me@w.example.com's password: ", true),
        ("work", "(me@work) Password: ", true),
        ("work", "root@w.example.com's password: ", false),
        ("work", "me@x.example.com's password: ", false),
        ("root@work", "root@w.example.com's password: ", true),
        ("root@work", "me@w.example.com's password: ", false),
    ];
    for (target, line, answered) in cases {
        let now = Instant::now();
        let mut answer = answer_for(target, Some(&entry), now);
        assert_eq!(
            answer.reply(line, now).is_some(),
            answered,
            "{target} {line}"
        );
    }
}
