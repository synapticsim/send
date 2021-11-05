#![feature(min_specialization)]
#![warn(clippy::all, clippy::restriction, clippy::pedantic, clippy::nursery, clippy::cargo)]

mod actor;
mod context;

use std::cell::UnsafeCell;

pub use actor::*;
pub use context::*;
// pub use send_derive::receive;

/// The root of everything.
///
/// It handles a root [`Actor`] and all its sub-[`Actor`]s,
/// and facilitates message-passing between them, as well as external events.
pub struct Framework<R> {
	root: UnsafeCell<R>,
}

impl<R> Framework<R>
where
	R: Actor + 'static,
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
		F: FnOnce(&mut R) -> &mut A,
	{
		let mut visitor = MessageVisitor {
			message,
			root: &self.root,
		};
		unsafe { getter(&mut *self.root.get()).accept(&mut visitor) }
	}

	/// Send a message that contains references to fields or sub-fields.
	/// If the selected fields implement [`Actor`], this will panic in debug mode, and will be UB in release mode.
	/// This sends the message to every [`Actor`] in the [`Framework`].
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.
	pub fn send_with<'a, S, F, C, M>(&mut self, selector: S, creator: C)
	where
		S: FnOnce(&'a mut R) -> F,
		F: 'a,
		C: FnOnce(F) -> M,
	{
		// SAFETY: We are verifying that the selected fields do not respond to any messages, and thus, cannot mutate
		// themselves while other actors are reading from them
		let fields = selector(unsafe { &mut *self.root.get() });
		debug_assert!(!F::is_actor(), "Tried to use fields that are Actors themselves");
		self.send(&mut creator(fields));
	}

	/// Send a message that contains references to fields or sub-fields.
	/// If the selected fields implement [`Actor`], this will panic in debug mode, and will be UB in release mode.
	/// This sends a message to only a specific [`Actor`].
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.  
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the message to.
	pub fn send_to_with<'a, S, F, C, M, G, A>(&mut self, selector: S, creator: C, getter: G)
	where
		S: FnOnce(&'a mut R) -> F,
		F: 'a,
		C: FnOnce(F) -> M,
		G: FnOnce(&mut R) -> &mut A,
	{
		let fields = selector(unsafe { &mut *self.root.get() });
		debug_assert!(!F::is_actor(), "Tried to use fields that are Actors themselves");
		self.send_to(&mut creator(fields), getter);
	}

	/// Send a message that contains references to fields or sub-fields.
	/// If the selected fields implement [`Actor`], this will panic in debug mode, and will be UB in release mode.
	/// This sends a message to a specific [`Actor`] and its sub-[`Actor`]s.
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.  
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the message to.
	pub fn send_sub_with<'a, S, F, C, M, G, A>(&mut self, selector: S, creator: C, getter: G)
	where
		S: FnOnce(&'a mut R) -> F,
		F: 'a,
		C: FnOnce(F) -> M,
		G: FnOnce(&mut R) -> &mut A,
	{
		let fields = selector(unsafe { &mut *self.root.get() });
		debug_assert!(!F::is_actor(), "Tried to use fields that are Actors themselves");
		self.send_sub(&mut creator(fields), getter);
	}

	/// Get a reference to the root [`Actor`].
	pub fn get(&self) -> &R { unsafe { &*self.root.get() } }

	/// Get a mutable reference to the root [`Actor`].
	/// This shouldn't be used very often: prefer sending events instead.
	pub fn get_mut(&mut self) -> &mut R { self.root.get_mut() }
}

struct MessageVisitor<'a, M, R> {
	message: &'a mut M,
	root: &'a UnsafeCell<R>,
}

impl<M, R> ActorVisitor<M, R> for MessageVisitor<'_, M, R> {
	#[inline(always)]
	fn visit<A>(&mut self, actor: &mut A)
	where
		A: Actor + Receiver<M, R>,
	{
		let context = Context::new(self.root);
		actor.receive(self.message, context);
	}
}

#[macro_export]
macro_rules! receive {
	($(%$generics:tt)? $message_ty:ty => $on:ty = |&mut $self:ident, $message:pat, $context:pat| $code:block) => {
		$crate::receive! { $message_ty, $on, $self, $message, $context, $code, $($generics)? }
	};

	($message_ty:ty, $on:ty, $self:ident, $message:pat, $context:pat, $code:block, $( ( $($generics:tt)* ) )?) => {
        impl<_RootTy, $($($generics)*)?> $crate::Receiver<$message_ty, _RootTy> for $on {
            fn receive(&mut $self, $message: &mut $message_ty, $context: $crate::Context<$on, _RootTy>) $code
        }
    };
}

#[macro_export]
macro_rules! f {
	($b:block) => {
		$b
	};
}
