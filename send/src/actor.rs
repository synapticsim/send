use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, LinkedList, VecDeque};

pub use send_derive::Actor;
use uom::num::Num;
use uom::si::{Dimension, Quantity, Units};
use uom::Conversion;

use crate::Context;

/// A visitor for [`Actor`]s.
pub trait ActorVisitor<T, R> {
	/// Visit the [`Actor`], doing whatever on it.
	fn visit<A>(&mut self, actor: &mut A)
	where
		A: Actor + Receiver<T, R>;
}

/// An [`Actor`] that can contain sub-[`Actor`]s.
/// If [`accept`] is not a no-op, you *must* un-implement [`NotActor`]:
/// ```
/// # #![feature(negative_impls)]
/// # use send::NotActor;
///
/// # struct MyActor;
///
/// impl !NotActor for MyActor {}
/// ```
///
/// Use the derive macro instead of implementing this by hand.
pub unsafe trait Actor {
	/// Accept an [`ActorVisitor`].
	fn accept<T, R>(&mut self, visitor: &mut impl ActorVisitor<T, R>);
}

/// A trait that allows an [`Actor`] to receive a message sent from another [`Actor`].
pub trait Receiver<T, R>: Sized {
	/// Receives the message.
	fn receive(&mut self, message: &mut T, context: Context<Self, R>);
}

pub auto trait NotActor {}

impl<D: Dimension, U: Units<V>, V: Num + Conversion<V>> NotActor for Quantity<D, U, V> {}

// A dummy implementation for all types.
// Specialization will be used to override this behavior while deriving.
unsafe impl<T> Actor for T {
	#[inline(always)]
	default fn accept<V, R>(&mut self, _: &mut impl ActorVisitor<V, R>) {}
}

// A dummy implementation for all types.
// Specialization will be used to override this behavior for custom types.
impl<M, R, T> Receiver<M, R> for T {
	#[inline(always)]
	default fn receive(&mut self, _: &mut M, _: Context<Self, R>) {}
}

// Implementations for standard library types.

unsafe impl<T> Actor for &T {
	#[inline(always)]
	default fn accept<V, R>(&mut self, visitor: &mut impl ActorVisitor<V, R>) { self.accept(visitor) }
}

unsafe impl<T> Actor for &mut T {
	#[inline(always)]
	default fn accept<V, R>(&mut self, visitor: &mut impl ActorVisitor<V, R>) { self.accept(visitor) }
}

unsafe impl<T> Actor for Option<T> {
	#[inline(always)]
	fn accept<V, R>(&mut self, visitor: &mut impl ActorVisitor<V, R>) {
		if let Some(v) = self.as_mut() {
			v.accept(visitor)
		}
	}
}

unsafe impl<T, E> Actor for Result<T, E> {
	#[inline(always)]
	fn accept<V, R>(&mut self, visitor: &mut impl ActorVisitor<V, R>) {
		match self.as_mut() {
			Ok(v) => v.accept(visitor),
			Err(v) => v.accept(visitor),
		}
	}
}

unsafe impl<T> Actor for Box<T> {
	#[inline(always)]
	fn accept<V, R>(&mut self, visitor: &mut impl ActorVisitor<V, R>) { self.as_mut().accept(visitor); }
}

unsafe impl<T> Actor for [T] {
	#[inline(always)]
	fn accept<V, R>(&mut self, visitor: &mut impl ActorVisitor<V, R>) {
		for v in self {
			v.accept(visitor);
		}
	}
}

unsafe impl<T, const N: usize> Actor for [T; N] {
	#[inline(always)]
	fn accept<V, R>(&mut self, visitor: &mut impl ActorVisitor<V, R>) {
		for v in self {
			v.accept(visitor);
		}
	}
}

unsafe impl<T> Actor for Vec<T> {
	#[inline(always)]
	fn accept<V, R>(&mut self, visitor: &mut impl ActorVisitor<V, R>) {
		for v in self {
			v.accept(visitor);
		}
	}
}

unsafe impl<T> Actor for VecDeque<T> {
	#[inline(always)]
	fn accept<V, R>(&mut self, visitor: &mut impl ActorVisitor<V, R>) {
		for v in self {
			v.accept(visitor);
		}
	}
}

unsafe impl<T> Actor for LinkedList<T> {
	#[inline(always)]
	fn accept<V, R>(&mut self, visitor: &mut impl ActorVisitor<V, R>) {
		for v in self {
			v.accept(visitor);
		}
	}
}

unsafe impl<K, V> Actor for HashMap<K, V> {
	#[inline(always)]
	fn accept<T, R>(&mut self, visitor: &mut impl ActorVisitor<T, R>) {
		for v in self {
			v.1.accept(visitor);
		}
	}
}

unsafe impl<K, V> Actor for BTreeMap<K, V> {
	#[inline(always)]
	fn accept<T, R>(&mut self, visitor: &mut impl ActorVisitor<T, R>) {
		for v in self {
			v.1.accept(visitor);
		}
	}
}

unsafe impl<T> Actor for RefCell<T> {
	#[inline(always)]
	fn accept<V, R>(&mut self, visitor: &mut impl ActorVisitor<V, R>) { self.get_mut().accept(visitor); }
}

unsafe impl<A> Actor for (A,) {
	#[inline(always)]
	fn accept<T, R>(&mut self, visitor: &mut impl ActorVisitor<T, R>) { self.0.accept(visitor); }
}

unsafe impl<A, B> Actor for (A, B) {
	#[inline(always)]
	fn accept<T, R>(&mut self, visitor: &mut impl ActorVisitor<T, R>) {
		self.0.accept(visitor);
		self.1.accept(visitor);
	}
}

unsafe impl<A, B, C> Actor for (A, B, C) {
	#[inline(always)]
	fn accept<T, R>(&mut self, visitor: &mut impl ActorVisitor<T, R>) {
		self.0.accept(visitor);
		self.1.accept(visitor);
		self.2.accept(visitor);
	}
}

unsafe impl<A, B, C, D> Actor for (A, B, C, D) {
	#[inline(always)]
	fn accept<T, R>(&mut self, visitor: &mut impl ActorVisitor<T, R>) {
		self.0.accept(visitor);
		self.1.accept(visitor);
		self.2.accept(visitor);
		self.3.accept(visitor);
	}
}

unsafe impl<A, B, C, D, E> Actor for (A, B, C, D, E) {
	#[inline(always)]
	fn accept<T, R>(&mut self, visitor: &mut impl ActorVisitor<T, R>) {
		self.0.accept(visitor);
		self.1.accept(visitor);
		self.2.accept(visitor);
		self.3.accept(visitor);
		self.4.accept(visitor);
	}
}

unsafe impl<A, B, C, D, E, F> Actor for (A, B, C, D, E, F) {
	#[inline(always)]
	fn accept<T, R>(&mut self, visitor: &mut impl ActorVisitor<T, R>) {
		self.0.accept(visitor);
		self.1.accept(visitor);
		self.2.accept(visitor);
		self.3.accept(visitor);
		self.4.accept(visitor);
		self.5.accept(visitor);
	}
}

unsafe impl<A, B, C, D, E, F, G> Actor for (A, B, C, D, E, F, G) {
	#[inline(always)]
	fn accept<T, R>(&mut self, visitor: &mut impl ActorVisitor<T, R>) {
		self.0.accept(visitor);
		self.1.accept(visitor);
		self.2.accept(visitor);
		self.3.accept(visitor);
		self.4.accept(visitor);
		self.5.accept(visitor);
		self.6.accept(visitor);
	}
}

unsafe impl<A, B, C, D, E, F, G, H> Actor for (A, B, C, D, E, F, G, H) {
	#[inline(always)]
	fn accept<T, R>(&mut self, visitor: &mut impl ActorVisitor<T, R>) {
		self.0.accept(visitor);
		self.1.accept(visitor);
		self.2.accept(visitor);
		self.3.accept(visitor);
		self.4.accept(visitor);
		self.5.accept(visitor);
		self.6.accept(visitor);
		self.7.accept(visitor);
	}
}
