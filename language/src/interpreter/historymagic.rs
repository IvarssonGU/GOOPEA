use input::input;
use std::fmt::Debug;

pub trait HMT {
    fn next(&self) -> Self;
}

pub struct HistoryMagic<T> {
    size: usize,
    history: Vec<(usize, T)>,
}

impl<T: HMT + Clone> HistoryMagic<T> {
    fn from_init(size: usize, init: T) -> Self {
        HistoryMagic::<T> {
            size: size,
            history: vec![(0, init)],
        }
    }

    fn get(&self) -> T {
        self.history.last().unwrap().1.clone()
    }

    fn next(&mut self) {
        let size = self.size;
        let (n, curr) = self.history.last().unwrap();
        let n = *n + 1;
        self.history.push((n, curr.next()));

        if n % size == 0 && self.history.len() > 2 * size {
            let end = self.history.len() - size - 1;
            let start = end - size + 1;
            if self.history[start].0 == n - 2 * size + 1 {
                self.history.drain(start..end);
            }
        }
    }

    fn back(&mut self) {
        if self.history.len() == 1 {
            return;
        }

        let (n, _) = self.history.pop().unwrap();
        let (m, _) = self.history.last().unwrap();
        if *m == (n - 1) {
            return;
        }
        for _ in 1..self.size {
            self.next();
        }
    }
}

impl<T: Debug> Debug for HistoryMagic<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.history)
    }
}

#[derive(Clone)]
struct Hej {
    n: i32,
}

impl Debug for Hej {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.n)
    }
}

impl Hej {
    fn new() -> Self {
        Hej { n: 0 }
    }
}

impl HMT for Hej {
    fn next(&self) -> Self {
        Hej { n: self.n + 1 }
    }
}

pub fn test_hm() {
    let mut history = HistoryMagic::from_init(5, Hej::new());
    loop {
        println!("{:?}", history);
        let s: String = input("");
        match s.as_str() {
            "b" => history.back(),
            _ => history.next(),
        }
    }
}
