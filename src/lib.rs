//! Construction Sticks for Horizon — slice 1, the plane fill.
//!
//! A construction stick right-clicked on a block fills the plane of the clicked face
//! with copies of the clicked block, bounded by the stick's tier and the player's
//! supply; every placed block costs one item of that block's kind and one durability.
//!
//! [`runtime`] is the pure planner (byte-identical to the pre-migration file, and
//! unit-tested on its own); [`support`] is the glue that reads the world, the
//! inventory and the clicked state, writes the blocks and wears the stick. This file
//! is the one `impl Hooks`: the item part that answers a right click, attached to the
//! five sticks the package's content declares.

pub mod runtime;
pub mod support;

use mod_api::{Hooks, Registrar, RegistrationContext};

/// The mod has no state of its own: every number is in the stick's tier
/// ([`runtime`]) and everything else is derived from the click.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Attach the stick behaviour to the five stick items the content declares.
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        support::register_sticks(registrar);
    }
}
