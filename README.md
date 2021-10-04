# `send`

A library for efficient local message-passing 
(pretty much [actors](https://en.wikipedia.org/wiki/Actor_model) but without the concurrent part).

## Integrating

Add this to the `[dependencies]` of your `Cargo.toml`:
```toml
[dependencies]
send = { git = "https://github.com/Synaptic-Simulations/send", branch = "main" }
```

`send` requires nightly, and you have to add this to the top of your crate:
```rs
#![feature(min_specialization)]
```

## Using

`send` is a pretty small library because it has just one thing to do: safe, type-safe message passing.

### Actors, Messages, Events

Everything in `send` is an `Actor`. Actors can receive messages and events, send messages,
and contain sub-actors. Pretty simple.

To make something an actor, you simply:
```rs
use send::Actor;

#[derive(Actor)]
pub struct Actor {}
```

What this does is register all sub-actors of this actor to be used for broadcasted messages and events.

Messages are sent between actors, while events are sent externally. 
Messages and events can be any type, without any special traits needing to be implemented. 

### Framework

The `Framework` is the 'root' of all Actors. 
It allows you to send external events to all actors, a specific one, 
or a specific one and all its sub-actors.

Creating a `Framework` is done with `Framework::new()` and the root actor:
```rs
let framework: Framework<MyActor> = Framework::new(MyActor {});
```

### Receiving messages and events

To receive messages on an actor, simply implement the trait `send::Receiver` for your actor:
```rs
impl Receiver<Message, MyActor> for MyActor {
    fn receive(&mut self, message: &Message, context: Context<Self, MyActor>) {
        // Do something with `message`
    }
}
```

To receive event on an actor, simply implement the trait `send::EventReceiver` for your actor:
```rs
impl EventReceiver<Event, MyActor> for MyActor {
    fn receive_event(&mut self, event: &Event, context: Context<Self, MyActor>) {
        // Do something with `event`
    }
}
```

Over here, `Event` and `Message` are the types that you send, 
and `MyActor` is the type of the root actor.

### Sending messages

In `receive` and `receive_event`, `context` allows you to send messages to all actors,
a specific one, or a specific one and its sub-actors. 
These messages are sent and evaluated immediately.

Every method on `Context` requires you to pass `self` as the first parameter, for safety.
