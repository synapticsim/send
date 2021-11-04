use std::cell::UnsafeCell;
use std::marker::PhantomData;

use crate::{Actor, ActorVisitor, MessageVisitor, Receiver};

/// A context that give you access to the [`Framework`](super::Framework) from inside an [`Actor`].
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
	R: Actor + 'static,
{
	/// Broadcast a message to all the [`Actor`]s in the [`Framework`](super::Framework).
	#[inline(always)]
	pub fn broadcast<T>(&self, _from: &mut S, message: &mut T) {
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
	pub fn send<T, F, A>(&self, _from: &mut S, message: &mut T, getter: F)
	where
		A: Actor + Receiver<T, R>,
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
	pub fn send_sub<T, F, A>(&self, _from: &mut S, message: &mut T, getter: F)
	where
		A: Actor + Receiver<T, R>,
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

	/// Send a message that contains references to fields or sub-fields.
	/// If the selected fields implement [`Actor`], this will panic in debug mode, and will be UB in release mode.
	/// This sends the message to every [`Actor`] in the [`Framework`](super::Framework).
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.
	pub fn send_with<'a, Sel, F, C, M>(&mut self, from: &mut S, selector: Sel, creator: C)
	where
		Sel: FnOnce(&'a R) -> F,
		F: 'a,
		C: FnOnce(F) -> M,
	{
		let fields = selector(unsafe { &*self.root.get() });
		debug_assert!(!F::is_actor(), "Tried to use fields that are Actors themselves");
		self.broadcast(from, &mut creator(fields));
	}

	/// Send a message that contains references to fields or sub-fields.
	/// If the selected fields implement [`Actor`], this will panic in debug mode, and will be UB in release mode.
	/// This sends a message to only a specific [`Actor`].
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.  
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the message to.
	pub fn send_to_with<'a, Sel, F, C, M, G, A>(&mut self, from: &mut S, selector: Sel, creator: C, getter: G)
	where
		Sel: FnOnce(&'a R) -> F,
		F: 'a,
		C: FnOnce(F) -> M,
		G: FnOnce(&mut R) -> &mut A,
	{
		let fields = selector(unsafe { &*self.root.get() });
		debug_assert!(!F::is_actor(), "Tried to use fields that are Actors themselves");
		self.send(from, &mut creator(fields), getter);
	}

	/// Send a message that contains references to fields or sub-fields.
	/// If the selected fields implement [`Actor`], this will panic in debug mode, and will be UB in release mode.
	/// This sends a message to a specific [`Actor`] and its sub-[`Actor`]s.
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.  
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the message to.
	pub fn send_sub_with<'a, Sel, F, C, M, G, A>(&mut self, from: &mut S, selector: Sel, creator: C, getter: G)
	where
		Sel: FnOnce(&'a R) -> F,
		F: 'a,
		C: FnOnce(F) -> M,
		G: FnOnce(&mut R) -> &mut A,
	{
		let fields = selector(unsafe { &*self.root.get() });
		debug_assert!(!F::is_actor(), "Tried to use fields that are Actors themselves");
		self.send_sub(from, &mut creator(fields), getter);
	}
}
