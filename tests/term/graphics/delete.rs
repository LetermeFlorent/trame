use trame::term::Terminal;

use super::fixture::{all_placed, put, send};

fn three_images() -> Terminal {
    let mut term = Terminal::new(6, 10, 100);
    put(&mut term, "i=1,p=1,c=2,r=2,C=1");
    term.feed(b"\x1b[3;5H");
    put(&mut term, "i=2,c=3,r=1,z=5,C=1");
    term.feed(b"\x1b[5;1H");
    put(&mut term, "i=3,c=1,r=1,C=1");
    term
}

fn left(term: &Terminal) -> Vec<u32> {
    let mut images: Vec<u32> = all_placed(term).into_iter().map(|(id, _)| id).collect();
    images.sort_unstable();
    images
}

#[test]
fn deleting_by_id_keeps_the_data_unless_uppercase() {
    let mut term = three_images();
    send(&mut term, "a=d,d=i,i=1");
    assert_eq!(left(&term), vec![2, 3]);
    assert!(term.screen().graphics().get(1).is_some());
    send(&mut term, "a=d,d=I,i=2");
    assert_eq!(left(&term), vec![3]);
    assert!(term.screen().graphics().get(2).is_none());
}

#[test]
fn deleting_by_id_and_placement_spares_other_placements() {
    let mut term = three_images();
    send(&mut term, "a=p,i=1,p=2,c=1,r=1,C=1");
    send(&mut term, "a=d,d=i,i=1,p=1");
    assert_eq!(
        all_placed(&term).iter().filter(|(id, _)| *id == 1).count(),
        1
    );
}

#[test]
fn deleting_by_id_reaches_the_history_and_virtual_placements() {
    let mut term = three_images();
    put(&mut term, "i=1,U=1,c=1,r=1");
    term.feed(b"\x1b[6;1H\n\n\n\n\n\n");
    send(&mut term, "a=d,d=I,i=1");
    assert_eq!(left(&term), vec![2, 3]);
    assert!(term.screen().graphics().virtuals.is_empty());
    assert!(term.screen().graphics().get(1).is_none());
}

#[test]
fn deleting_one_virtual_placement_keeps_the_others() {
    let mut term = Terminal::new(6, 10, 0);
    put(&mut term, "i=1,p=1,U=1,c=1,r=1");
    send(&mut term, "a=p,i=1,p=2,U=1,c=2,r=1,q=2");
    send(&mut term, "a=d,d=i,i=1,p=1");
    let virtuals = &term.screen().graphics().virtuals;
    assert_eq!(virtuals.keys().copied().collect::<Vec<_>>(), vec![(1, 2)]);
}

#[test]
fn deleting_at_the_cursor_cell() {
    let mut term = three_images();
    term.feed(b"\x1b[2;2H");
    send(&mut term, "a=d,d=c");
    assert_eq!(left(&term), vec![2, 3]);
}

#[test]
fn deleting_at_a_cell_column_or_row() {
    let mut term = three_images();
    send(&mut term, "a=d,d=p,x=7,y=3");
    assert_eq!(left(&term), vec![1, 3]);
    send(&mut term, "a=d,d=x,x=1");
    assert!(left(&term).is_empty());
    let mut term = three_images();
    send(&mut term, "a=d,d=y,y=2");
    assert_eq!(left(&term), vec![2, 3]);
}

#[test]
fn deleting_by_layer_range_and_number() {
    let mut term = three_images();
    send(&mut term, "a=d,d=z,z=5");
    assert_eq!(left(&term), vec![1, 3]);
    send(&mut term, "a=d,d=R,x=1,y=2");
    assert_eq!(left(&term), vec![3]);
    assert!(term.screen().graphics().get(2).is_none());
    send(&mut term, "a=T,f=24,s=1,v=1,I=9,q=2,C=1;AAAA");
    send(&mut term, "a=d,d=N,I=9");
    assert_eq!(left(&term), vec![3]);
}

#[test]
fn deleting_everything_visible_frees_only_what_it_removed() {
    let mut term = three_images();
    send(&mut term, "a=t,f=24,s=1,v=1,i=8,q=2;AAAA");
    send(&mut term, "a=d,d=A");
    assert!(left(&term).is_empty());
    assert!(term.screen().graphics().get(1).is_none());
    assert!(term.screen().graphics().get(8).is_some());
}

#[test]
fn deleting_everything_spares_the_history() {
    let mut term = three_images();
    term.feed(b"\x1b[6;1H\n\n");
    send(&mut term, "a=d");
    assert_eq!(left(&term), vec![1]);
}
