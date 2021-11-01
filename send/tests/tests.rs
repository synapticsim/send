#![feature(min_specialization)]

use send::{Actor, Context, Framework, Receiver};

#[derive(Actor)]
struct MyActor {
	value: i16,
	actor: OtherActor,
}

#[derive(Actor)]
struct OtherActor {
	value: i16,
}

struct Increment {
	value: i16,
}

impl Receiver<Increment, MyActor> for MyActor {
	fn receive(&mut self, event: &Increment, _context: Context<Self, MyActor>) { self.value += event.value; }
}

impl Receiver<Increment, MyActor> for OtherActor {
	fn receive(&mut self, event: &Increment, _context: Context<Self, MyActor>) { self.value += event.value; }
}

#[test]
fn events() {
	let actor = MyActor {
		value: 0,
		actor: OtherActor { value: 0 },
	};
	let mut framework = Framework::new(actor);

	let mut message = Increment { value: 1 };

	framework.send(&mut message);
	assert_eq!(framework.get().value, 1);
	assert_eq!(framework.get().actor.value, 1);

	framework.send_to(&mut message, |actor| &mut actor.actor);
	assert_eq!(framework.get().value, 1);
	assert_eq!(framework.get().actor.value, 2);
}
