use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;

pub trait Scope: Copy + Debug + Display + Ord + Hash + Sized + Send + Sync {
    const SINGLETON: Self;

    const MIN: Self;

    fn outlive(self, other: Self) -> bool {
        self >= other
    }

    fn within(self, other: Self) -> bool {
        self <= other
    }

    fn super_scope(self) -> Option<Self>;

    fn sub_scope(self) -> Option<Self>;

    fn to_str(&self) -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SingletonScope;

impl Display for SingletonScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_str())
    }
}

impl Scope for SingletonScope {
    const SINGLETON: Self = Self;

    const MIN: Self = Self;

    fn super_scope(self) -> Option<Self> {
        None
    }

    fn sub_scope(self) -> Option<Self> {
        None
    }

    fn to_str(&self) -> &'static str {
        "Singleton"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WebScope {
    Singleton = 3,
    Session = 2,
    Request = 1,
}

impl Display for WebScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_str())
    }
}

impl Scope for WebScope {
    const SINGLETON: Self = Self::Singleton;

    const MIN: Self = Self::Request;

    fn super_scope(self) -> Option<Self> {
        match self {
            Self::Singleton => None,
            Self::Session => Some(Self::Singleton),
            Self::Request => Some(Self::Session),
        }
    }

    fn sub_scope(self) -> Option<Self> {
        match self {
            Self::Singleton => Some(Self::Session),
            Self::Session => Some(Self::Request),
            Self::Request => None,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Self::Singleton => "Singleton",
            Self::Session => "Session",
            Self::Request => "Request",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lifetime<S: Scope> {
    Scoped(S),
    Transient(Transient),
}

impl<S: Scope> Lifetime<S> {
    pub fn scoped(scope: S) -> Self {
        Self::Scoped(scope)
    }

    pub fn transient() -> Self {
        Self::Transient(Transient)
    }
}

impl<S: Scope> Display for Lifetime<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_str())
    }
}

impl<S: Scope> PartialOrd for Lifetime<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: Scope> Ord for Lifetime<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Scoped(s1), Self::Scoped(s2)) => s1.cmp(s2),
            (Self::Scoped(_), Self::Transient(_)) => Ordering::Greater,
            (Self::Transient(_), Self::Scoped(_)) => Ordering::Less,
            (Self::Transient(_), Self::Transient(_)) => Ordering::Equal,
        }
    }
}

impl<S: Scope> Scope for Lifetime<S> {
    const SINGLETON: Self = Self::Scoped(S::SINGLETON);

    const MIN: Self = Self::Transient(Transient);

    fn super_scope(self) -> Option<Self> {
        match self {
            Self::Scoped(s) => s.super_scope().map(|s| Self::Scoped(s)),
            Self::Transient(_) => Some(Self::Scoped(S::MIN)),
        }
    }

    fn sub_scope(self) -> Option<Self> {
        match self {
            Lifetime::Scoped(s) => Some(s.sub_scope().map_or(Self::MIN, |s| Self::Scoped(s))),
            Lifetime::Transient(_) => None,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Self::Scoped(s) => s.to_str(),
            Self::Transient(_) => "Transient",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Transient;
