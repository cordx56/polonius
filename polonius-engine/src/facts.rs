use std::fmt::Debug;
use std::hash::Hash;

/// The "facts" which are the basis of the NLL borrow analysis.
#[derive(Clone, Debug, Hash, PartialEq, Eq, serde::Serialize)]
pub struct AllFacts<T: FactTypes> {
    /// `loan_issued_at(origin, loan, point)` indicates that the `loan` was "issued"
    /// at the given `point`, creating a reference with the `origin`.
    /// Effectively, `origin` may refer to data from `loan` starting at `point` (this is usually
    /// the point *after* a borrow rvalue).
    #[serde(serialize_with = "ser3")]
    pub loan_issued_at: Vec<(T::Origin, T::Loan, T::Point)>,

    /// `universal_region(origin)` -- this is a "free region" within fn body
    #[serde(serialize_with = "ser1")]
    pub universal_region: Vec<T::Origin>,

    /// `cfg_edge(point1, point2)` for each edge `point1 -> point2` in the control flow
    #[serde(serialize_with = "ser2")]
    pub cfg_edge: Vec<(T::Point, T::Point)>,

    /// `loan_killed_at(loan, point)` when some prefix of the path borrowed at `loan`
    /// is assigned at `point`.
    /// Indicates that the path borrowed by the `loan` has changed in some way that the loan no
    /// longer needs to be tracked. (In particular, mutations to the path that was borrowed
    /// no longer invalidate the loan)
    #[serde(serialize_with = "ser2")]
    pub loan_killed_at: Vec<(T::Loan, T::Point)>,

    /// `subset_base(origin1, origin2, point)` when we require `origin1@point: origin2@point`.
    /// Indicates that `origin1 <= origin2` -- i.e., the set of loans in `origin1` are a subset
    /// of those in `origin2`.
    #[serde(serialize_with = "ser3")]
    pub subset_base: Vec<(T::Origin, T::Origin, T::Point)>,

    /// `loan_invalidated_at(point, loan)` indicates that the `loan` is invalidated by some action
    /// taking place at `point`; if any origin that references this loan is live, this is an error.
    #[serde(serialize_with = "ser2")]
    pub loan_invalidated_at: Vec<(T::Point, T::Loan)>,

    /// `var_used_at(var, point)` when the variable `var` is used for anything
    /// but a drop at `point`
    #[serde(serialize_with = "ser2")]
    pub var_used_at: Vec<(T::Variable, T::Point)>,

    /// `var_defined_at(var, point)` when the variable `var` is overwritten at `point`
    #[serde(serialize_with = "ser2")]
    pub var_defined_at: Vec<(T::Variable, T::Point)>,

    /// `var_dropped_at(var, point)` when the variable `var` is used in a drop at `point`
    #[serde(serialize_with = "ser2")]
    pub var_dropped_at: Vec<(T::Variable, T::Point)>,

    /// `use_of_var_derefs_origin(variable, origin)`: References with the given
    /// `origin` may be dereferenced when the `variable` is used.
    ///
    /// In rustc, we generate this whenever the type of the variable includes the
    /// given origin.
    #[serde(serialize_with = "ser2")]
    pub use_of_var_derefs_origin: Vec<(T::Variable, T::Origin)>,

    /// `drop_of_var_derefs_origin(var, origin)` when the type of `var` includes
    /// the `origin` and uses it when dropping
    #[serde(serialize_with = "ser2")]
    pub drop_of_var_derefs_origin: Vec<(T::Variable, T::Origin)>,

    /// `child_path(child, parent)` when the path `child` is the direct child of
    /// `parent`, e.g. `child_path(x.y, x)`, but not `child_path(x.y.z, x)`.
    #[serde(serialize_with = "ser2")]
    pub child_path: Vec<(T::Path, T::Path)>,

    /// `path_is_var(path, var)` the root path `path` starting in variable `var`.
    #[serde(serialize_with = "ser2")]
    pub path_is_var: Vec<(T::Path, T::Variable)>,

    /// `path_assigned_at_base(path, point)` when the `path` was initialized at point
    /// `point`. This fact is only emitted for a prefix `path`, and not for the
    /// implicit initialization of all of `path`'s children. E.g. a statement like
    /// `x.y = 3` at `point` would give the fact `path_assigned_at_base(x.y, point)` (but
    /// neither `path_assigned_at_base(x.y.z, point)` nor `path_assigned_at_base(x, point)`).
    #[serde(serialize_with = "ser2")]
    pub path_assigned_at_base: Vec<(T::Path, T::Point)>,

    /// `path_moved_at_base(path, point)` when the `path` was moved at `point`. The
    /// same logic is applied as for `path_assigned_at_base` above.
    #[serde(serialize_with = "ser2")]
    pub path_moved_at_base: Vec<(T::Path, T::Point)>,

    /// `path_accessed_at_base(path, point)` when the `path` was accessed at point
    /// `point`. The same logic as for `path_assigned_at_base` and `path_moved_at_base` applies.
    #[serde(serialize_with = "ser2")]
    pub path_accessed_at_base: Vec<(T::Path, T::Point)>,

    /// These reflect the `'a: 'b` relations that are either declared by the user on function
    /// declarations or which are inferred via implied bounds.
    /// For example: `fn foo<'a, 'b: 'a, 'c>(x: &'c &'a u32)` would have two entries:
    /// - one for the user-supplied subset `'b: 'a`
    /// - and one for the `'a: 'c` implied bound from the `x` parameter,
    /// (note that the transitive relation `'b: 'c` is not necessarily included
    /// explicitly, but rather inferred by polonius).
    #[serde(serialize_with = "ser2")]
    pub known_placeholder_subset: Vec<(T::Origin, T::Origin)>,

    /// `placeholder(origin, loan)` describes a placeholder `origin`, with its associated
    ///  placeholder `loan`.
    #[serde(serialize_with = "ser2")]
    pub placeholder: Vec<(T::Origin, T::Loan)>,
}

impl<T: FactTypes> Default for AllFacts<T> {
    fn default() -> Self {
        AllFacts {
            loan_issued_at: Vec::default(),
            universal_region: Vec::default(),
            cfg_edge: Vec::default(),
            loan_killed_at: Vec::default(),
            subset_base: Vec::default(),
            loan_invalidated_at: Vec::default(),
            var_used_at: Vec::default(),
            var_defined_at: Vec::default(),
            var_dropped_at: Vec::default(),
            use_of_var_derefs_origin: Vec::default(),
            drop_of_var_derefs_origin: Vec::default(),
            child_path: Vec::default(),
            path_is_var: Vec::default(),
            path_assigned_at_base: Vec::default(),
            path_moved_at_base: Vec::default(),
            path_accessed_at_base: Vec::default(),
            known_placeholder_subset: Vec::default(),
            placeholder: Vec::default(),
        }
    }
}

pub trait Atom:
    From<usize> + Into<usize> + Copy + Clone + Debug + Eq + Ord + Hash + 'static
{
    fn index(self) -> usize;
}

pub trait FactTypes: Copy + Clone + Debug {
    type Origin: Atom;
    type Loan: Atom;
    type Point: Atom;
    type Variable: Atom;
    type Path: Atom;
}

use serde::{de::Deserializer, ser::SerializeSeq, Serializer};

impl FactTypes for DeAtom {
    type Origin = DeAtom;
    type Loan = DeAtom;
    type Point = DeAtom;
    type Variable = DeAtom;
    type Path = DeAtom;
}

pub(crate) type SerdeFacts = AllFacts<DeAtom>;

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct DeAtom {
    atom: usize,
}
impl From<usize> for DeAtom {
    fn from(value: usize) -> Self {
        DeAtom { atom: value }
    }
}
impl Into<usize> for DeAtom {
    fn into(self) -> usize {
        self.atom
    }
}
impl Atom for DeAtom {
    fn index(self) -> usize {
        self.atom
    }
}

fn ser1<S>(atom: &[impl Atom], ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = ser.serialize_seq(None)?;
    for a in atom {
        seq.serialize_element(&a.index())?;
    }
    seq.end()
}
fn ser2<S>(atom: &[(impl Atom, impl Atom)], ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = ser.serialize_seq(None)?;
    for (a, b) in atom {
        seq.serialize_element(&(a.index(), b.index()))?;
    }
    seq.end()
}
fn ser3<S>(atom: &[(impl Atom, impl Atom, impl Atom)], ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = ser.serialize_seq(None)?;
    for (a, b, c) in atom {
        seq.serialize_element(&(a.index(), b.index(), c.index()))?;
    }
    seq.end()
}

impl<'de> serde::Deserialize<'de> for SerdeFacts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Helper {
            #[serde(default)]
            loan_issued_at: Vec<[usize; 3]>,
            #[serde(default)]
            universal_region: Vec<usize>,
            #[serde(default)]
            cfg_edge: Vec<[usize; 2]>,
            #[serde(default)]
            loan_killed_at: Vec<[usize; 2]>,
            #[serde(default)]
            subset_base: Vec<[usize; 3]>,
            #[serde(default)]
            loan_invalidated_at: Vec<[usize; 2]>,
            #[serde(default)]
            var_used_at: Vec<[usize; 2]>,
            #[serde(default)]
            var_defined_at: Vec<[usize; 2]>,
            #[serde(default)]
            var_dropped_at: Vec<[usize; 2]>,
            #[serde(default)]
            use_of_var_derefs_origin: Vec<[usize; 2]>,
            #[serde(default)]
            drop_of_var_derefs_origin: Vec<[usize; 2]>,
            #[serde(default)]
            child_path: Vec<[usize; 2]>,
            #[serde(default)]
            path_is_var: Vec<[usize; 2]>,
            #[serde(default)]
            path_assigned_at_base: Vec<[usize; 2]>,
            #[serde(default)]
            path_moved_at_base: Vec<[usize; 2]>,
            #[serde(default)]
            path_accessed_at_base: Vec<[usize; 2]>,
            #[serde(default)]
            known_placeholder_subset: Vec<[usize; 2]>,
            #[serde(default)]
            placeholder: Vec<[usize; 2]>,
        }

        let h = Helper::deserialize(deserializer)?;

        let a = |x| DeAtom::from(x);

        Ok(SerdeFacts {
            loan_issued_at: h
                .loan_issued_at
                .into_iter()
                .map(|[o, l, p]| (a(o), a(l), a(p)))
                .collect(),
            universal_region: h.universal_region.into_iter().map(a).collect(),
            cfg_edge: h
                .cfg_edge
                .into_iter()
                .map(|[p1, p2]| (a(p1), a(p2)))
                .collect(),
            loan_killed_at: h
                .loan_killed_at
                .into_iter()
                .map(|[l, p]| (a(l), a(p)))
                .collect(),
            subset_base: h
                .subset_base
                .into_iter()
                .map(|[o1, o2, p]| (a(o1), a(o2), a(p)))
                .collect(),
            loan_invalidated_at: h
                .loan_invalidated_at
                .into_iter()
                .map(|[p, l]| (a(p), a(l)))
                .collect(),
            var_used_at: h
                .var_used_at
                .into_iter()
                .map(|[v, p]| (a(v), a(p)))
                .collect(),
            var_defined_at: h
                .var_defined_at
                .into_iter()
                .map(|[v, p]| (a(v), a(p)))
                .collect(),
            var_dropped_at: h
                .var_dropped_at
                .into_iter()
                .map(|[v, p]| (a(v), a(p)))
                .collect(),
            use_of_var_derefs_origin: h
                .use_of_var_derefs_origin
                .into_iter()
                .map(|[v, o]| (a(v), a(o)))
                .collect(),
            drop_of_var_derefs_origin: h
                .drop_of_var_derefs_origin
                .into_iter()
                .map(|[v, o]| (a(v), a(o)))
                .collect(),
            child_path: h
                .child_path
                .into_iter()
                .map(|[c, p]| (a(c), a(p)))
                .collect(),
            path_is_var: h
                .path_is_var
                .into_iter()
                .map(|[p, v]| (a(p), a(v)))
                .collect(),
            path_assigned_at_base: h
                .path_assigned_at_base
                .into_iter()
                .map(|[p, pt]| (a(p), a(pt)))
                .collect(),
            path_moved_at_base: h
                .path_moved_at_base
                .into_iter()
                .map(|[p, pt]| (a(p), a(pt)))
                .collect(),
            path_accessed_at_base: h
                .path_accessed_at_base
                .into_iter()
                .map(|[p, pt]| (a(p), a(pt)))
                .collect(),
            known_placeholder_subset: h
                .known_placeholder_subset
                .into_iter()
                .map(|[o1, o2]| (a(o1), a(o2)))
                .collect(),
            placeholder: h
                .placeholder
                .into_iter()
                .map(|[o, l]| (a(o), a(l)))
                .collect(),
        })
    }
}
