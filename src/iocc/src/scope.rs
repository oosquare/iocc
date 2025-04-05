use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;

/// A type that specifies how long a managed object can live.
///
/// A [`Scope`] is typically implemented as an `enum`, whose each variant
/// represents a possible scope. In other words, a [`Scope`] is not a single
/// scope but a set of supported scopes. Variants in a [`Scope`] are analogous
/// to sets, where each variant has at most one predecesssor and at most one
/// successor in a *subset* partial order relationship. Note that the [`Ord`]
/// trait is required by [`Scope`], and the implementation should satisfy:
///
/// - `a >= b` if `a` is a superset of `b` (either true superset or equal)
/// - `a <= b` if `a` is a subset of `b` (either true subset or equal)
///
/// Each [`Registry`] is associated with a [`Scope`], since it stores all
/// definitions of how to construct managed objects, which should be aware of
/// the lifetime of those objects.
///
/// [`Registry`]: crate::container::registry::Registry
pub trait Scope: Copy + Debug + Display + Ord + Hash + Sized + Send + Sync + 'static {
    /// A scope corresponded to objects that are created only once and managed
    /// through out the container's lifetime.
    const SINGLETON: Self;

    /// The shortest possible scope of scoped objects.
    const MIN: Self;

    /// Returns true if `self` is a true superset of `other` or is equal to it.
    fn outlive(self, other: Self) -> bool {
        self >= other
    }

    /// Returns true if `self` is a true subset of `other` or is equal to it.
    fn within(self, other: Self) -> bool {
        self <= other
    }

    /// Returns the shortest scope which strictly outlives `self`.
    fn super_scope(self) -> Option<Self>;

    /// Returns the longest scope which is strictly within `self`.
    fn sub_scope(self) -> Option<Self>;

    /// Returns the name of the current scope in a string literal.
    fn to_str(&self) -> &'static str;
}

/// A [`Scope`] which only has one possible scope, i.e. the singleton scope.
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

/// A [`Scope`] whose variants are corresponded to possible lifetimes in web
/// applications.
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

/// A type that represents arbitrary lifetimes for objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Transient;

/// A type that represents the lifetime of an object created by a container.
///
/// Objects can be either scoped or transient. A scoped object is something
/// whose both creation and ownership are managed by a container, with its
/// uniqueness enforced by the container within a certain scope. A transient
/// object, also called unbounded object, is an object which only has its
/// creation managed by any container in contrast, and can live for arbitrary
/// duration as long as other objects preserve its ownership.
///
/// However, the lifetime of a scoped object can still be expanded and outlives
/// the container, since the object are shared and it's possible to hold the
/// object after the container's destruction. In other word, lifetime only makes
/// sense in object construction and just acts as a hint in terms of object
/// destruction.
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
        match self {
            Self::Scoped(s) => write!(f, "{s}"),
            Self::Transient(_) => write!(f, "Transient"),
        }
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
