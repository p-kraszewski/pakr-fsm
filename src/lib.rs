//! # Example
//! ```
//! use pakr_fsm::*;
//! use std::error;
//!
//! #[derive(Copy, Clone, Eq, PartialEq, Debug)]
//! enum MyEv {
//!     E1,
//!     E2,
//! }
//!
//! #[derive(Copy, Clone, Eq, PartialEq, Debug)]
//! enum MyState {
//!     S1,
//!     S2,
//! }
//!
//! impl Default for MyState {
//!     fn default() -> Self { Self::S1 }
//! }
//!
//! struct MyFSM;
//!
//! impl FSM for MyFSM {
//!     type Event = MyEv;
//!     type Response = &'static str;
//!     type State = MyState;
//!
//!     fn new() -> Self { Self {} }
//!
//!     fn trasnsit(
//!         &mut self,
//!         old_state: &Self::State,
//!         ev: &Self::Event,
//!     ) -> (Option<Self::State>, Option<Self::Response>) {
//!         match (old_state, ev) {
//!             (MyState::S1, MyEv::E1) => (None, Some("Quitting")),
//!             (MyState::S1, MyEv::E2) => (Some(MyState::S2), None),
//!             (MyState::S2, MyEv::E1) => (Some(MyState::S1), Some("S2@E1->S1")),
//!             (MyState::S2, MyEv::E2) => (Some(MyState::S2), Some("S2@E2->S2")),
//!         }
//!     }
//!
//!     fn respond(
//!         &mut self,
//!         old_state: &<Self as FSM>::State,
//!         new_state: &Option<<Self as FSM>::State>,
//!         resp: &Self::Response,
//!     ) {
//!         println!("{:?} -> ({:?},{:?}", old_state, new_state, resp);
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn error::Error>> {
//!     let fsm = Reactor::<MyFSM>::new();
//!     fsm.send(MyEv::E2)?;
//!     fsm.send(MyEv::E2)?;
//!     fsm.send(MyEv::E1)?;
//!     fsm.send(MyEv::E1)?;
//!     let ans = fsm.join().unwrap();
//!     println!("Final state result: {:?}",ans);
//!     Ok(())
//! }
//! ```

use std::{sync::mpsc, thread};
use std::any::Any;

/// Trait `FSM` engulfs transition logic and related datatypes.
///
/// It can represent both [Mealy state machine](https://en.wikipedia.org/wiki/Mealy_machine) (output
/// from transit) and  [Moore state machine](https://en.wikipedia.org/wiki/Moore_machine) (output from
/// state)
pub trait FSM {
    /// Events are sent from outer world to influence state of machine
    type Event: Send + Eq + PartialEq + 'static;

    /// Response are optional outcomes of transition
    type Response: Send + 'static;

    /// Current state of the machine. Default should init machine state to entry
    /// one
    type State: Eq + PartialEq + Default;

    /// Creating a new machine
    fn new() -> Self;

    /// Mapping current state & event into a eventual new state and eventual
    /// response.
    ///
    /// - A new state of `None` means machine termination, with response as return value
    /// - A response of `None` means no output given
    fn trasnsit(
        &mut self,
        old_state: &<Self as FSM>::State,
        ev: &<Self as FSM>::Event,
    ) -> (
        Option<<Self as FSM>::State>,
        Option<<Self as FSM>::Response>,
    );

    ///
    fn respond(
        &mut self,
        old_state: &<Self as FSM>::State,
        new_state: &Option<<Self as FSM>::State>,
        resp: &<Self as FSM>::Response,
    );
}

/// Reactor is `FSM` handle to interact and monitor
pub struct Reactor<F: FSM> {
    reactor: thread::JoinHandle<Option<F::Response>>,
    chan: mpsc::Sender<F::Event>,
}

impl<F: FSM> Reactor<F> {
    /// Create new `Reactor`.
    ///
    /// Spawns associated `FSM` in separate thread, itializes it by calling its `new` and setting
    /// state to `default`.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<F::Event>();

        let t = thread::spawn(move || {
            let mut fsm = F::new();
            let mut state = F::State::default();

            while let Ok(ev) = rx.recv() {
                let (new_state, response) = fsm.trasnsit(&state, &ev);
                if let Some(response) = &response {
                    fsm.respond(&state, &new_state, &response);
                }

                match new_state {
                    None => return response,
                    Some(new_state) => state = new_state,
                }
            }
            return None;
        });

        Self {
            reactor: t,
            chan: tx,
        }
    }

    /// Waits for `FSM` to complete.
    ///
    /// Returns response of the last transition
    pub fn join(self) -> thread::Result<Option<<F as FSM>::Response>> { self.reactor.join() }

    /// Send event to `FSM`
    pub fn send(&self, ev: <F as FSM>::Event) -> Result<(), mpsc::SendError<<F as FSM>::Event>> {
        self.chan.send(ev)
    }

    /// Clone `send` endpoint of `FSM` channel to be used in other places.
    pub fn get_sender(&self) -> mpsc::Sender<F::Event> { self.chan.clone() }
}
