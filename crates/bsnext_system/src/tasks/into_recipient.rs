use crate::capabilities::Capabilities;
use actix::{Addr, Recipient};
use bsnext_task::invocation::Invocation;

/// ```rust
/// A trait that allows for the transformation of an instance into a `Recipient`
/// capable of receiving `Invocation` messages.
///
/// This is used to facilitate communication with actors or components
/// represented by an `Addr<Capabilities>`, enabling message handling
/// in an abstract and flexible manner.
///
/// # Required Methods
///
/// ## `into_recipient`
///
/// Converts the implementing instance into a `Recipient` that can handle
/// `Invocation` messages.
///
/// ### Arguments
/// - `self`: The instance being transformed into a `Recipient`.
/// - `addr`: A reference to the `Addr` representing the actor/component
///    with a specific set of `Capabilities`.
///
/// ### Returns
/// - A `Recipient<Invocation>` representing the transformed instance
///   that forwards or processes `Invocation` messages.
///
/// # Example
/// ```rust
/// struct MyActor;
///
/// impl IntoRecipient for MyActor {
///     fn into_recipient(self, addr: &Addr<Capabilities>) -> Recipient<Invocation> {
///         // Implementation details for creating the recipient
///     }
/// }
///
/// let addr: Addr<Capabilities> = Addr::new();
/// let my_actor = MyActor;
/// let recipient = my_actor.into_recipient(&addr);
/// ```
/// pub
pub trait IntoRecipient {
    fn into_recipient(self, addr: &Addr<Capabilities>) -> Recipient<Invocation>;
}
