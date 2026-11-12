// A way to stop an export that is no longer wanted
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::fmt;
use std::sync::Arc;

/// Asked between pictures and pages whether the work should stop: a newer request has come for an
/// estimate that is being made, so the old one is not worth finishing.
#[derive(Clone, Default)]
pub struct Cancel(Option<Arc<dyn Fn() -> bool + Send + Sync>>);

impl Cancel {
    /// Stops when `wanted` says so.
    pub fn when(wanted: impl Fn() -> bool + Send + Sync + 'static) -> Self {
        Self(Some(Arc::new(wanted)))
    }

    pub fn is_set(&self) -> bool {
        self.0.as_ref().is_some_and(|f| f())
    }
}

impl fmt::Debug for Cancel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if self.0.is_some() {
            "Cancel(..)"
        } else {
            "Cancel"
        })
    }
}
