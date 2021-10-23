use std::cell::UnsafeCell;
use std::marker::PhantomData;

use crate::{Actor, ActorVisitor, EventReceiver, Receiver};

/// A context that give you access to the [`Framework`] from inside an [`Actor`].
pub struct Context<'a, S, R> {
	root: &'a UnsafeCell<R>,
	phantom: PhantomData<S>,
}

impl<'a, S, R> Context<'a, S, R> {
	pub fn new(root: &'a UnsafeCell<R>) -> Self {
		Self {
			root,
			phantom: PhantomData,
		}
	}
}

impl<S, R> Context<'_, S, R>
where
	R: Actor,
{
	/// Broadcast a message to all the [`Actor`]s in the [`Framework`].
	#[inline(always)]
	pub fn broadcast<T>(&self, _from: &mut S, message: &T) {
		// SAFETY:
		// This is safe because `from` was the only `Actor` that had a mutable reference taken to it.
		// Since we now have a mutable reference to `from`, we can mutate the `Framework`.
		let mut visitor = MessageVisitor {
			message,
			root: self.root,
		};
		unsafe {
			(*self.root.get()).accept(&mut visitor);
		}
	}

	/// Send a message to only a specific [`Actor`].
	///
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the event to.
	#[inline(always)]
	pub fn send<T, F, A>(&self, _from: &mut S, message: &T, getter: F)
	where
		A: Actor + Receiver<T, R> + EventReceiver<T, R>,
		F: FnOnce(&mut R) -> &mut A,
	{
		// SAFETY: Above ^^
		let mut visitor = MessageVisitor {
			message,
			root: self.root,
		};
		unsafe { visitor.visit(getter(&mut *self.root.get())) }
	}

	/// Send a message to a specific [`Actor`] and its sub-[`Actor`]s.
	///
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the event to.
	#[inline(always)]
	pub fn send_sub<T, F, A>(&self, _from: &mut S, message: &T, getter: F)
	where
		A: Actor + Receiver<T, R> + EventReceiver<T, R>,
		F: FnOnce(&mut R) -> &mut A,
	{
		// SAFETY: Above ^^
		let mut visitor = MessageVisitor {
			message,
			root: self.root,
		};
		unsafe {
			getter(&mut *self.root.get()).accept(&mut visitor);
		}
	}
}

struct MessageVisitor<'a, M, R> {
	message: &'a M,
	root: &'a UnsafeCell<R>,
}

impl<M, R> ActorVisitor<M, R> for MessageVisitor<'_, M, R> {
	#[inline(always)]
	fn visit<A>(&mut self, actor: &mut A)
	where
		A: Actor + Receiver<M, R> + EventReceiver<M, R>,
	{
		let context = Context::new(self.root);
		actor.receive(&self.message, context);
	}
}
