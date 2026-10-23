use std::time::{Duration, Instant};

use trame::preferences::{ANSWER_WINDOW, Prompt, SecretAnswer, prompt_in};

const PASSWORD: &str = "root@10.0.0.2's password: ";
const PASSPHRASE: &str = "Enter passphrase for key 'C:\\Users\\me/.ssh/id_ed25519': ";

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
fn each_kind_of_prompt_gets_the_secret_once() {
    let now = Instant::now();
    let mut answer = SecretAnswer::new("hunter2".to_owned(), now);
    assert_eq!(answer.reply("Welcome", now), None);
    assert_eq!(answer.reply(PASSPHRASE, now).as_deref(), Some("hunter2\r"));
    assert_eq!(answer.reply(PASSPHRASE, now), None);
    assert_eq!(answer.reply(PASSWORD, now).as_deref(), Some("hunter2\r"));
    assert_eq!(answer.reply(PASSWORD, now), None);
}

#[test]
fn the_secret_is_only_given_shortly_after_connecting() {
    let now = Instant::now();
    let mut answer = SecretAnswer::new("hunter2".to_owned(), now);
    let late = now + ANSWER_WINDOW;
    assert!(!answer.is_over(late - Duration::from_millis(1)));
    assert!(answer.is_over(late));
    assert_eq!(answer.reply(PASSWORD, late), None);
}
