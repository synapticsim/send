#![feature(min_specialization)]
#![warn(clippy::all, clippy::restriction, clippy::pedantic, clippy::nursery, clippy::cargo)]

mod actor;
mod context;

use std::cell::UnsafeCell;

pub use actor::*;
pub use context::*;

/// The root of everything.
///
/// It handles a root [`Actor`] and all its sub-[`Actor`]s,
/// and facilitates message-passing between them, as well as external events.
pub struct Framework<R> {
	root: UnsafeCell<R>,
}

impl<R> Framework<R>
where
	R: Actor,
{
	/// Create a [`Framework`] handling a root [`Actor`].
	pub fn new(root: R) -> Self {
		Self {
			root: UnsafeCell::new(root),
		}
	}

	/// Send a message to every [`Actor`] in the [`Framework`].
	pub fn send<M>(&mut self, message: &mut M) {
		let mut visitor = MessageVisitor {
			message,
			root: &self.root,
		};
		unsafe {
			(*self.root.get()).accept(&mut visitor);
		}
	}

	/// Send a message to only a specific [`Actor`].
	///
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the message to.
	pub fn send_to<M, F, A>(&mut self, message: &mut M, getter: F)
	where
		A: Actor + Receiver<M, R>,
		F: FnOnce(&mut R) -> &mut A,
	{
		let mut visitor = MessageVisitor {
			message,
			root: &self.root,
		};
		unsafe {
			visitor.visit(getter(&mut *self.root.get()));
		}
	}

	/// Send a message to a specific [`Actor`] and its sub-[`Actor`]s.
	///
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the message to.
	pub fn send_sub<M, F, A>(&mut self, message: &mut M, getter: F)
	where
		A: Actor + Receiver<M, R>,
		F: FnOnce(&mut R) -> &mut A,
	{
		let mut visitor = MessageVisitor {
			message,
			root: &self.root,
		};
		unsafe { getter(&mut *self.root.get()).accept(&mut visitor) }
	}

	/// Get a reference to the root [`Actor`].
	pub fn get(&self) -> &R { unsafe { &*self.root.get() } }

	/// Get a mutable reference to the root [`Actor`].
	/// This shouldn't be used very often: prefer sending events instead.
	pub fn get_mut(&mut self) -> &mut R { self.root.get_mut() }
}

struct MessageVisitor<'a, M, R> {
	message: &'a M,
	root: &'a UnsafeCell<R>,
}

impl<M, R> ActorVisitor<M, R> for MessageVisitor<'_, M, R> {
	#[inline(always)]
	fn visit<A>(&mut self, actor: &mut A)
	where
		A: Actor + Receiver<M, R>,
	{
		let context = Context::new(self.root);
		actor.receive(&self.message, context);
	}
}
