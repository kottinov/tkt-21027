pub mod crd;
pub mod error;
pub mod reconciler;
pub mod resources;

pub use crd::DummySite;
pub use error::{Error, Result};
pub use reconciler::Reconciler;

pub mod prelude {
    pub use crate::crd::DummySite;
    pub use crate::error::{Error, Result};
    pub use crate::reconciler::Reconciler;
}
