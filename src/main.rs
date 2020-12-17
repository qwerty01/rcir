use std::collections::HashSet;
use rcir::Poll;

fn main() {
    let mut candidates: HashSet<String> = HashSet::new();
    
    let num_candidates = 10;
    let num_ballots = 10_000_000;

    println!("Generating {} candidates...", num_candidates);
    for i in 0..num_candidates {
        candidates.insert(format!("Person {}", i));
    }
    let mut poll = Poll::new(&candidates);
    
    println!("Generating {} ballots...", num_ballots);
    for _ in 0..num_ballots {
        let ballot = poll.generate_ballot();
        poll.add_ballot(ballot).unwrap();
    }
    
    println!("Calculating results...");
    let mut winner = None;
    for result in poll.start_rounds() {
        println!("Round {}: {} ({} votes)", result.round, result.loser, result.votes);
        winner = Some(result.loser);
    }
    println!("Election winner: {}!", winner.unwrap());
}
