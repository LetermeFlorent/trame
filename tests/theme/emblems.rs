use trame::theme::Emblem;

#[test]
fn the_default_emblem_is_the_grid() {
    assert_eq!(Emblem::default(), Emblem::Grid);
    assert_eq!(Emblem::ALL[0], Emblem::Grid);
}

#[test]
fn every_emblem_is_listed_once() {
    for (index, emblem) in Emblem::ALL.iter().enumerate() {
        assert!(!Emblem::ALL[index + 1..].contains(emblem));
    }
}

#[test]
fn a_pick_never_returns_a_taken_emblem_while_one_is_free() {
    let taken = &Emblem::ALL[..Emblem::ALL.len() - 1];
    for seed in 0..32 {
        assert_eq!(Emblem::pick(taken, seed), Emblem::Gear);
    }
    let few = [Emblem::Grid, Emblem::Code];
    for seed in 0..32 {
        assert!(!few.contains(&Emblem::pick(&few, seed)));
    }
}

#[test]
fn the_seed_chooses_among_the_free_emblems() {
    let picks: Vec<Emblem> = (0..Emblem::ALL.len() as u64)
        .map(|seed| Emblem::pick(&[], seed))
        .collect();
    assert_eq!(picks, Emblem::ALL.to_vec());
    assert_eq!(
        Emblem::pick(&[], u64::MAX),
        Emblem::ALL[(u64::MAX % 8) as usize]
    );
}

#[test]
fn when_every_emblem_is_taken_any_one_can_come_back() {
    let all = Emblem::ALL;
    for seed in 0..all.len() as u64 {
        assert_eq!(Emblem::pick(&all, seed), all[seed as usize]);
    }
    assert!(all.contains(&Emblem::random(&all)));
}

#[test]
fn a_random_emblem_is_free_when_one_is_left() {
    let taken = &Emblem::ALL[1..];
    for _ in 0..16 {
        assert_eq!(Emblem::random(taken), Emblem::Grid);
    }
}

#[test]
fn emblems_are_saved_by_their_snake_case_name() {
    let text = serde_json::to_string(&Emblem::Database).expect("serializes");
    assert_eq!(text, "\"database\"");
    let back: Emblem = serde_json::from_str("\"gear\"").expect("parses");
    assert_eq!(back, Emblem::Gear);
    assert!(serde_json::from_str::<Emblem>("\"rocket\"").is_err());
}
