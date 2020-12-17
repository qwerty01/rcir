use rand::thread_rng;
use rand::seq::SliceRandom;
use std::fmt;
use std::error::Error;
use std::hash::Hash;
use std::collections::{HashMap, HashSet};

pub struct RoundIterator<'a, T: Eq + Hash + fmt::Debug> {
    curr_round: PollRound<'a, T>,
}

impl<'a, T: Eq + Hash + fmt::Debug> Iterator for RoundIterator<'a, T> {
    type Item = PollResult<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let round = self.curr_round.next_round();
        match round {
            Some((result, r)) => {
                self.curr_round = r;
                Some(result)
            },
            None => None
        }
    }
}

pub struct PollResult<'a, T: Eq + Hash + fmt::Debug> {
    pub loser: &'a T,
    pub votes: usize,
    pub results: HashMap<&'a T, usize>,
    pub round: usize,
}
pub struct PollRound<'a, T: Eq + Hash + fmt::Debug> {
    candidates: Vec<&'a T>,
    ballots: Vec<Vec<&'a T>>,
    last_round: usize,
}
impl<'a, 'b, T: Eq + Hash + fmt::Debug> PollRound<'a, T> {
    pub fn first_round(poll: &'b Poll<'a, T>) -> RoundIterator<'a, T> {
        let ballots = poll.ballots.clone();
        RoundIterator {
            curr_round: PollRound {
                candidates: poll.candidates.iter().collect(),
                ballots,
                last_round: 0
            }
        }
    }
    fn next_round(&self) -> Option<(PollResult<'a, T>, Self)> {
        let mut results = HashMap::new();
        let mut next_ballots: Vec<Vec<&'a T>> = Vec::new();
        
        for &i in &self.candidates {
            results.insert(i, 0);
        }
        
        for ballot in &self.ballots {
            // Ballot should have been validated in Poll::add_ballot.
            // All ballots have the same amount of candidates, so if this one does not have a first candidate,
            //   then all of them are empty and we're done the vote
            let &vote = match ballot.first() {
                Some(v) => v,
                None => return None,
            };
            // Since Poll::add_ballot already verified that this will never fail, panic if it does (indicates a bug in Poll::add_ballot).
            let vote_box = results.get_mut(vote).unwrap();
            *vote_box += 1;
        }
        
        let mut lowest = None;
        let mut loser = None;
        
        for (&k, &v) in &results {
            if lowest.is_none() {
                lowest = Some(v);
                loser = Some(k);
                continue;
            }
            let lw = lowest.unwrap();
            if v < lw {
                lowest = Some(v);
                loser = Some(k);
            }
        }
        
        let loser = loser.unwrap();
        let lowest = lowest.unwrap();
        
        for ballot in &self.ballots {
            let new_ballot = ballot.iter().filter(|&&i| i != loser).map(|&i| i).collect();
            next_ballots.push(new_ballot);
        }
        
        Some((
            PollResult {
                loser,
                votes: lowest,
            results,
            round: self.last_round + 1,
            },
            Self {
                candidates: self.candidates.iter().filter(|&&i| i != loser).map(|&i| i).collect(),
                ballots: next_ballots,
                last_round: self.last_round + 1
            }
        ))
    }
}

#[derive(Debug, PartialEq)]
pub enum BallotError<'a, T> {
    MissingCandidate(&'a T),
    DuplicateCandidate(&'a T),
    ExtraCandidate(&'a T),
}
impl<'a, T: fmt::Display> fmt::Display for BallotError<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingCandidate(c) => write!(f, "Candidate missing: {}", c),
            Self::DuplicateCandidate(c) => write!(f, "Duplicate candidate: {}", c),
            Self::ExtraCandidate(c) => write!(f, "Candidate not in poll: {}", c),
        }
    }
}
impl<'a, T: fmt::Debug + fmt::Display> Error for BallotError<'a, T> {}

pub struct Poll<'a, T: Eq + Hash + fmt::Debug> {
    candidates: &'a HashSet<T>,
    ballots: Vec<Vec<&'a T>>,
    candidate_map: HashMap<&'a T, bool>,
}
impl<'a, T: Eq + Hash + fmt::Debug> Poll<'a, T> {
    pub fn new(candidates: &'a HashSet<T>) -> Self {
        let mut candidate_map: HashMap<&'a T, bool> = HashMap::new();

        for i in candidates {
            candidate_map.insert(i, false);
        }

        Self {
            candidates: candidates,
            ballots: Vec::new(),
            candidate_map,
        }
    }
    pub fn generate_ballot(&self) -> Vec<&'a T> {
        let mut vec: Vec<&T> = self.candidates.iter().collect();
        // Randomize ballot to negate ballot order effect
        vec.shuffle(&mut thread_rng());
        vec
    }
    pub fn add_ballot(&mut self, ballot: Vec<&'a T>) -> Result<(), BallotError<'a, T>> {
        // A hashmap is used so that we can verify all the candidates provided are in the poll,
        // that we don't have any duplicate candidates, and that there are no missing candidates
        let mut hm = self.candidate_map.clone();
        for &v in &ballot {
            let available = match hm.get_mut(v) {
                // Candidate is part of the poll
                Some(b) => b,
                // Ballot contains a candidate that wasn't in the poll
                None => return Err(BallotError::ExtraCandidate(v)),
            };
            if *available {
                // Candidate was already chosen at a higher rank
                return Err(BallotError::DuplicateCandidate(v));
            }
            *available = true;
        }

        for (&k, v) in &hm {
            if !*v {
                // Candidate wasn't included in the ballot
                return Err(BallotError::MissingCandidate(k));
            }
        }
        
        // Ballot is verified
        self.ballots.push(ballot);

        Ok(())
    }
    pub fn start_rounds<'b>(&self) -> RoundIterator<'a, T> {
        PollRound::first_round(self)
    }
}
impl<'a, T: Eq + Hash + fmt::Debug> From<&'a HashSet<T>> for Poll<'a, T> {
    fn from(candidates: &'a HashSet<T>) -> Self {
        Self::new(candidates)
    }
}
