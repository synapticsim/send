#![feature(min_specialization)]

use send::{Actor, Context, EventReceiver, Framework, Receiver};

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

impl EventReceiver<Increment, MyActor> for MyActor {
	fn receive_event(&mut self, event: &mut Increment, context: Context<Self, MyActor>) {
		self.value += event.value;
		context.broadcast(self, &Increment { value: 1 });
	}
}

impl Receiver<Increment, MyActor> for MyActor {
	fn receive(&mut self, event: &Increment, _context: Context<Self, MyActor>) { self.value += event.value; }
}

impl EventReceiver<Increment, MyActor> for OtherActor {
	fn receive_event(&mut self, event: &mut Increment, _context: Context<Self, MyActor>) { self.value += event.value; }
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

	let mut event = Increment { value: 1 };

	framework.send_event(&mut event);
	assert_eq!(framework.get().value, 2);
	assert_eq!(framework.get().actor.value, 2);

	framework.send_event_to(&mut event, |actor| &mut actor.actor);
	assert_eq!(framework.get().value, 2);
	assert_eq!(framework.get().actor.value, 3);
}
