use trame::term::Terminal;

pub fn send(term: &mut Terminal, keys: &str) {
    term.feed(format!("\x1b_G{keys}\x1b\\").as_bytes());
}

pub fn put(term: &mut Terminal, keys: &str) {
    term.feed(format!("\x1b_Ga=T,f=24,s=1,v=1,q=2,{keys};AAAA\x1b\\").as_bytes());
}

pub fn placed(term: &Terminal, row: usize) -> Vec<(u32, u32)> {
    term.screen()
        .grid()
        .row(row)
        .anchors()
        .iter()
        .map(|anchor| (anchor.image, anchor.placement))
        .collect()
}

pub fn all_placed(term: &Terminal) -> Vec<(u32, u32)> {
    term.screen()
        .grid()
        .anchors()
        .map(|anchor| (anchor.image, anchor.placement))
        .collect()
}
