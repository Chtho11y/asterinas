// SPDX-License-Identifier: MPL-2.0

pub mod priority;
// TODO: Remove this out-dated module once the `sched_class` module is stable.
mod priority_scheduler;
mod sched_class;
mod stats;

// Export the stats getter functions.
pub use stats::{loadavg, nr_queued_and_running};

// TODO: Use `sched_class::init` instead after the completion of #1676.
pub use self::priority_scheduler::init;
// There may be multiple scheduling policies in the system,
// and subsequent schedulers can be placed under this module.
pub use self::sched_class::SchedAttr;
